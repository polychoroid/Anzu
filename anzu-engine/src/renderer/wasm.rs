use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use winit::{
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta},
    keyboard::PhysicalKey,
    window::Window,
};

use super::core::{BatchVertex, DrawBatch, RenderCore, RenderError, RotationUniform};
use crate::ecs::{EntityId, World};
use crate::input::InputEvent;
use crate::renderer::RenderRomPackage;
use crate::simulation::{InteractionEventKind, PhysicsConfig, Scheduler, SimulationModel};

const FIXED_STEP_SECONDS: f32 = 1.0 / 60.0;
const MAX_FIXED_STEPS_PER_FRAME: u32 = 8;
const INPUT_LOG_INTERVAL_FRAMES: u64 = 120;

struct GamepadPollStats {
    visible_gamepads: usize,
    slot_count: u32,
    document_has_focus: bool,
}

fn log_info(message: &str) {
    web_sys::console::log_1(&JsValue::from_str(message));
}

fn log_warn(message: &str) {
    web_sys::console::warn_1(&JsValue::from_str(message));
}

impl RenderError {
    fn into_js_value(self) -> JsValue {
        JsValue::from_str(&self.to_string())
    }
}

pub struct State {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    world: World,
    anchor_entity: EntityId,
    scheduler: Scheduler,
    frame_index: u64,
    fixed_tick_index: u64,
    next_scheduler_log_frame: u64,
    fixed_time_accumulator_seconds: f32,
    fixed_step_clamp_count: u64,
    game_over: bool,
    simulation_paused: bool,
    pending_step_ticks: u32,
    logged_gamepad_devices: BTreeSet<u32>,
    gamepad_button_states: BTreeMap<(u32, usize), bool>,
    gamepad_button_values: BTreeMap<(u32, usize), f32>,
    gamepad_axis_values: BTreeMap<(u32, usize), f32>,
    simulation: Box<dyn SimulationModel>,
    render_core: RenderCore,
}

impl State {
    fn simulation_counters(&self) -> (f32, f32, f32, f32, f32) {
        let mut linear_momentum_x = 0.0f32;
        let mut linear_momentum_y = 0.0f32;
        let mut angular_momentum = 0.0f32;
        let mut kinetic_energy_linear = 0.0f32;
        let mut kinetic_energy_angular = 0.0f32;

        for (entity_id, body) in self.world.rigid_bodies_iter() {
            let Some(transform) = self.world.transform(entity_id) else {
                continue;
            };

            linear_momentum_x += body.mass * transform.velocity_x;
            linear_momentum_y += body.mass * transform.velocity_y;
            angular_momentum += body.moment_of_inertia * transform.angular_velocity;
            kinetic_energy_linear += 0.5
                * body.mass
                * (transform.velocity_x * transform.velocity_x
                    + transform.velocity_y * transform.velocity_y);
            kinetic_energy_angular += 0.5
                * body.moment_of_inertia
                * transform.angular_velocity
                * transform.angular_velocity;
        }

        (
            linear_momentum_x,
            linear_momentum_y,
            angular_momentum,
            kinetic_energy_linear,
            kinetic_energy_angular,
        )
    }

    fn build_batch_vertices(&self) -> (Vec<BatchVertex>, Vec<DrawBatch>) {
        // First pass: collect world-space positions of all bullet entities.
        // Bullets are identified by having a Lifecycle component (only they have TTL).
        let mut bullet_positions: Vec<[f32; 2]> = Vec::new();
        self.world.for_each_render_mesh(|entity_id, _mesh, _material_id| {
            if self.world.lifecycle(entity_id).is_some() {
                if let Some(t) = self.world.transform(entity_id) {
                    bullet_positions.push([t.position_x, t.position_y]);
                }
            }
        });

        // Returns the world-space position of the nearest bullet, or a far-off
        // sentinel ([10.0, 10.0]) when no bullets exist so light_strength → 0.
        let nearest_bullet = |px: f32, py: f32| -> [f32; 2] {
            bullet_positions
                .iter()
                .copied()
                .min_by(|a, b| {
                    let da = (a[0] - px).powi(2) + (a[1] - py).powi(2);
                    let db = (b[0] - px).powi(2) + (b[1] - py).powi(2);
                    da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap_or([10.0, 10.0])
        };

        let mut batch = Vec::new();
        let mut draw_batches = Vec::new();

        self.world
            .for_each_render_mesh(|entity_id, mesh, material_id| {
            let Some(transform) = self.world.transform(entity_id) else {
                return;
            };

            let Ok(vertices) = bytemuck::try_cast_slice::<u8, BatchVertex>(mesh.vertex_bytes)
            else {
                log_warn(&format!(
                    "[RENDER] skipped mesh for entity={} due to incompatible vertex payload",
                    entity_id
                ));
                return;
            };

            let c = transform.rotation_rad.cos();
            let s = transform.rotation_rad.sin();
            let batch_start = batch.len() as u32;

            batch.extend(vertices.iter().map(|vertex| {
                let x = vertex.position[0] * transform.uniform_scale;
                let y = vertex.position[1] * transform.uniform_scale;

                let world_x = c * x - s * y + transform.position_x;
                let world_y = s * x + c * y + transform.position_y;

                BatchVertex {
                    position: [world_x, world_y],
                    color: vertex.color,
                    barycentric: vertex.barycentric,
                    light_pos: nearest_bullet(world_x, world_y),
                    edge_mask: vertex.edge_mask,
                }
            }));

            let batch_end = batch.len() as u32;
            draw_batches.push(DrawBatch {
                material_id,
                vertex_offset: batch_start,
                vertex_count: batch_end.saturating_sub(batch_start),
            });
        });

        (batch, draw_batches)
    }

    pub async fn new(
        window: Arc<Window>,
        rom: Box<dyn RenderRomPackage>,
        physics_config: PhysicsConfig,
    ) -> Result<Self, JsValue> {
        log_info("[RENDER] init: creating browser GPU instance");
        let size = window.inner_size();

        let mut instance_descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        instance_descriptor.backends = wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL;

        let instance = wgpu::util::new_instance_with_webgpu_detection(instance_descriptor).await;

        let surface = instance
            .create_surface(window.clone())
            .map_err(|error| RenderError::SurfaceCreation(error.to_string()).into_js_value())?;
        log_info("[RENDER] init: surface created");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|error| RenderError::AdapterRequest(error.to_string()).into_js_value())?;
        let adapter_info = adapter.get_info();
        log_info(&format!(
            "[RENDER] init: adapter acquired backend={:?} device_type={:?} driver='{}'",
            adapter_info.backend, adapter_info.device_type, adapter_info.driver
        ));

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|error| RenderError::DeviceRequest(error.to_string()).into_js_value())?;
        log_info("[RENDER] init: device and queue acquired");

        let surface_caps = surface.get_capabilities(&adapter);
        log_info(&format!(
            "[RENDER] surface capabilities: formats={} present_modes={} alpha_modes={}",
            surface_caps.formats.len(),
            surface_caps.present_modes.len(),
            surface_caps.alpha_modes.len()
        ));

        if surface_caps.formats.is_empty() {
            return Err(
                RenderError::UnsupportedSurface("surface reports no supported formats")
                    .into_js_value(),
            );
        }
        if surface_caps.present_modes.is_empty() {
            return Err(
                RenderError::UnsupportedSurface("surface reports no present modes").into_js_value(),
            );
        }
        if surface_caps.alpha_modes.is_empty() {
            return Err(
                RenderError::UnsupportedSurface("surface reports no alpha modes").into_js_value(),
            );
        }

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
            .or_else(|| surface_caps.formats.first().copied())
            .ok_or_else(|| {
                RenderError::UnsupportedSurface("surface format selection failed").into_js_value()
            })?;

        let present_mode = surface_caps.present_modes.first().copied().ok_or_else(|| {
            RenderError::UnsupportedSurface("surface present mode selection failed").into_js_value()
        })?;

        let alpha_mode = surface_caps.alpha_modes.first().copied().ok_or_else(|| {
            RenderError::UnsupportedSurface("surface alpha mode selection failed").into_js_value()
        })?;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);
        log_info(&format!(
            "[RENDER] surface configured: format={:?} size={}x{} present_mode={:?} alpha_mode={:?}",
            config.format, config.width, config.height, config.present_mode, config.alpha_mode
        ));

        log_info(&format!("[ROM] bootstrap start: rom_id='{}'", rom.rom_id()));
        log_info(&format!(
            "[SIM] physics config bounds=({:.2},{:.2})..({:.2},{:.2}) restitution={:.2} friction={:.2}",
            physics_config.world_min_x,
            physics_config.world_min_y,
            physics_config.world_max_x,
            physics_config.world_max_y,
            physics_config.default_restitution,
            physics_config.default_friction
        ));
        let mut world = World::new();
        let anchor_entity = rom.bootstrap_world(&mut world);
        log_info(&format!(
            "[ROM] world bootstrapped: anchor_entity={}",
            anchor_entity
        ));
        let mut simulation = rom.create_simulation(&world, anchor_entity);
        simulation.update(0.0);
        log_info("[ROM] simulation created and initial update completed");
        let render_data = rom.render_data();
        log_info(&format!(
            "[RENDER] render payload received: vertex_count={}",
            render_data.vertex_count
        ));
        let render_core = RenderCore::new(device, queue, config, simulation.as_ref(), render_data);
        log_info("[RENDER] render core initialized");

        let get_gamepads_available = web_sys::window()
            .and_then(|browser_window| {
                js_sys::Reflect::has(
                    &browser_window.navigator().into(),
                    &JsValue::from_str("getGamepads"),
                )
                .ok()
            })
            .unwrap_or(false);
        log_info(&format!(
            "[INPUT][GAMEPAD] API probe getGamepads_available={}",
            get_gamepads_available
        ));

        Ok(Self {
            window,
            surface,
            world,
            anchor_entity,
            scheduler: Scheduler::new(physics_config),
            frame_index: 0,
            fixed_tick_index: 0,
            next_scheduler_log_frame: 0,
            fixed_time_accumulator_seconds: 0.0,
            fixed_step_clamp_count: 0,
            game_over: false,
            simulation_paused: false,
            pending_step_ticks: 0,
            logged_gamepad_devices: BTreeSet::new(),
            gamepad_button_states: BTreeMap::new(),
            gamepad_button_values: BTreeMap::new(),
            gamepad_axis_values: BTreeMap::new(),
            simulation,
            render_core,
        })
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.render_core.resize(&self.surface, width, height);
    }

    pub fn handle_key_event(&mut self, event: &KeyEvent) {
        let PhysicalKey::Code(key_code) = event.physical_key else {
            return;
        };

        let is_pressed = event.state == ElementState::Pressed;
        if event.repeat && is_pressed {
            return;
        }

        log_info(&format!(
            "[INPUT][KEYBOARD] control={:?} pressed={} repeat={}",
            key_code, is_pressed, event.repeat
        ));

        self.simulation
            .handle_input_event(input_event_from_key_code(
                key_code,
                is_pressed,
                event.repeat,
            ));
    }

    pub fn handle_mouse_button_event(&mut self, button: MouseButton, state: ElementState) {
        let is_pressed = state == ElementState::Pressed;
        log_info(&format!(
            "[INPUT][MOUSE] button={:?} pressed={}",
            button, is_pressed
        ));
        self.simulation
            .handle_input_event(input_event_from_mouse_button(button, is_pressed));
    }

    pub fn handle_mouse_wheel_event(&mut self, delta: MouseScrollDelta) {
        let value = match delta {
            MouseScrollDelta::LineDelta(_, y) => y,
            MouseScrollDelta::PixelDelta(position) => position.y as f32,
        };

        self.simulation
            .handle_input_event(input_event_from_mouse_wheel(value));
    }

    pub fn handle_cursor_moved_event(&mut self, x: f32, y: f32) {
        self.simulation
            .handle_input_event(input_event_from_cursor_position("x", x));
        self.simulation
            .handle_input_event(input_event_from_cursor_position("y", y));
    }

    fn run_fixed_tick(&mut self) {
        if self.game_over {
            return;
        }

        let should_tick = if self.simulation_paused {
            if self.pending_step_ticks > 0 {
                self.pending_step_ticks = self.pending_step_ticks.saturating_sub(1);
                true
            } else {
                false
            }
        } else {
            true
        };

        if !should_tick {
            return;
        }

        self.simulation
            .sync_anchor_from_world(&self.world, self.anchor_entity);
        self.simulation.update(FIXED_STEP_SECONDS);
        self.simulation
            .write_anchor_to_world(&mut self.world, self.anchor_entity);

        let report = self
            .scheduler
            .update_world(&mut self.world, FIXED_STEP_SECONDS);

        self.simulation
            .sync_anchor_from_world(&self.world, self.anchor_entity);

        self.simulation
            .reconcile_world(&mut self.world, FIXED_STEP_SECONDS, &report);

        if report.despawned_entities.contains(&self.anchor_entity) {
            self.game_over = true;
            log_info(&format!(
                "[GAME] game over: anchor entity {} despawned at fixed_tick={}",
                self.anchor_entity, self.fixed_tick_index
            ));
        }

        self.fixed_tick_index = self.fixed_tick_index.saturating_add(1);

        let has_activity =
            !report.interaction_events.is_empty() || !report.despawned_entities.is_empty();
        if has_activity && self.fixed_tick_index >= self.next_scheduler_log_frame {
            let mut collisions = 0u32;
            let mut proximity_enter = 0u32;
            let mut proximity_exit = 0u32;

            for event in &report.interaction_events {
                match event.kind {
                    InteractionEventKind::Collision => collisions = collisions.saturating_add(1),
                    InteractionEventKind::ProximityEnter => {
                        proximity_enter = proximity_enter.saturating_add(1)
                    }
                    InteractionEventKind::ProximityExit => {
                        proximity_exit = proximity_exit.saturating_add(1)
                    }
                }
            }

            let (p_x, p_y, l_z, e_linear, e_angular) = self.simulation_counters();
            log_info(&format!(
                "[SIM] fixed_tick={} pairs_checked={} collisions={} proximity_enter={} proximity_exit={} despawned={} momentum=({:.4},{:.4}) angular_momentum={:.4} energy_linear={:.4} energy_angular={:.4}",
                self.fixed_tick_index,
                report.interaction_pairs_checked,
                collisions,
                proximity_enter,
                proximity_exit,
                report.despawned_entities.len(),
                p_x,
                p_y,
                l_z,
                e_linear,
                e_angular
            ));

            if !report.despawned_entities.is_empty() {
                log_info(&format!(
                    "[SIM] despawned entities: {:?}",
                    report.despawned_entities
                ));
            }

            self.next_scheduler_log_frame = self.fixed_tick_index.saturating_add(60);
        }
    }

    fn poll_gamepads(&mut self) -> GamepadPollStats {
        let Some(window) = web_sys::window() else {
            return GamepadPollStats {
                visible_gamepads: 0,
                slot_count: 0,
                document_has_focus: false,
            };
        };

        let document_has_focus = window
            .document()
            .and_then(|document| document.has_focus().ok())
            .unwrap_or(true);

        let Ok(gamepads) = window.navigator().get_gamepads() else {
            log_warn("[INPUT][GAMEPAD] navigator.getGamepads() failed");
            return GamepadPollStats {
                visible_gamepads: 0,
                slot_count: 0,
                document_has_focus,
            };
        };

        let mut seen_button_keys = BTreeMap::new();
        let mut seen_axis_keys = BTreeMap::new();
        let mut visible_gamepads = 0usize;
        let slot_count = gamepads.length();

        for gamepad_value in gamepads.iter() {
            if gamepad_value.is_null() || gamepad_value.is_undefined() {
                continue;
            }

            let Ok(gamepad) = gamepad_value.dyn_into::<web_sys::Gamepad>() else {
                continue;
            };

            visible_gamepads = visible_gamepads.saturating_add(1);
            let device_index = gamepad.index() as u32;

            if self.logged_gamepad_devices.insert(device_index) {
                log_info(&format!(
                    "[INPUT][GAMEPAD] detected index={} id='{}' buttons={} axes={}",
                    device_index,
                    gamepad.id(),
                    gamepad.buttons().length(),
                    gamepad.axes().length()
                ));
            }

            let buttons = gamepad.buttons();
            for (button_index, button_value) in buttons.iter().enumerate() {
                let Ok(button) = button_value.dyn_into::<web_sys::GamepadButton>() else {
                    continue;
                };

                let key = (device_index, button_index);
                seen_button_keys.insert(key, true);

                let is_pressed = button.pressed();
                let value = button.value() as f32;
                let previous_pressed = self
                    .gamepad_button_states
                    .get(&key)
                    .copied()
                    .unwrap_or(false);
                let previous_value = self.gamepad_button_values.get(&key).copied().unwrap_or(0.0);

                if previous_pressed != is_pressed || (previous_value - value).abs() >= 0.01 {
                    log_info(&format!(
                        "[INPUT][GAMEPAD] dispatch button index={} control={} value={:.3} pressed={}",
                        device_index,
                        gamepad_button_control(button_index),
                        value,
                        is_pressed
                    ));
                    self.simulation.handle_input_event(InputEvent {
                        source: "gamepad".to_owned(),
                        control: gamepad_button_control(button_index),
                        value,
                        device_index: Some(device_index),
                        is_pressed,
                        is_repeat: false,
                    });
                }

                self.gamepad_button_states.insert(key, is_pressed);
                self.gamepad_button_values.insert(key, value);
            }

            let axes = gamepad.axes();
            for (axis_index, axis_value) in axes.iter().enumerate() {
                let Some(mut value) = axis_value.as_f64().map(|v| v as f32) else {
                    continue;
                };

                value = value.clamp(-1.0, 1.0);
                let key = (device_index, axis_index);
                seen_axis_keys.insert(key, true);

                let previous = self.gamepad_axis_values.get(&key).copied().unwrap_or(0.0);
                if (previous - value).abs() >= 0.01 {
                    log_info(&format!(
                        "[INPUT][GAMEPAD] dispatch axis index={} control={} value={:.3} pressed={}",
                        device_index,
                        gamepad_axis_control(axis_index),
                        value,
                        value.abs() >= 0.5
                    ));
                    self.simulation.handle_input_event(InputEvent {
                        source: "gamepad".to_owned(),
                        control: gamepad_axis_control(axis_index),
                        value,
                        device_index: Some(device_index),
                        is_pressed: value.abs() >= 0.5,
                        is_repeat: false,
                    });

                    if let Some((control, normalized_value)) =
                        gamepad_axis_semantic_alias(axis_index, value)
                    {
                        log_info(&format!(
                            "[INPUT][GAMEPAD] dispatch axis alias index={} control={} value={:.3} pressed={}",
                            device_index,
                            control,
                            normalized_value,
                            normalized_value >= 0.5,
                        ));
                        self.simulation.handle_input_event(InputEvent {
                            source: "gamepad".to_owned(),
                            control: control.to_owned(),
                            value: normalized_value,
                            device_index: Some(device_index),
                            is_pressed: normalized_value >= 0.5,
                            is_repeat: false,
                        });
                    }
                }

                self.gamepad_axis_values.insert(key, value);
            }
        }

        self.gamepad_button_states
            .retain(|key, _| seen_button_keys.contains_key(key));
        self.gamepad_button_values
            .retain(|key, _| seen_button_keys.contains_key(key));
        self.gamepad_axis_values
            .retain(|key, _| seen_axis_keys.contains_key(key));
        self.logged_gamepad_devices.retain(|device_index| {
            seen_button_keys
                .keys()
                .any(|(seen_index, _)| seen_index == device_index)
                || seen_axis_keys
                    .keys()
                    .any(|(seen_index, _)| seen_index == device_index)
        });

        GamepadPollStats {
            visible_gamepads,
            slot_count,
            document_has_focus,
        }
    }

    pub fn update(&mut self, delta_seconds: f32) {
        self.frame_index = self.frame_index.saturating_add(1);
        let gamepad_stats = self.poll_gamepads();
        let clamped_delta_seconds = delta_seconds.clamp(0.0, 0.25);
        self.fixed_time_accumulator_seconds += clamped_delta_seconds;

        let mut ticks_this_frame = 0u32;
        while self.fixed_time_accumulator_seconds >= FIXED_STEP_SECONDS
            && ticks_this_frame < MAX_FIXED_STEPS_PER_FRAME
        {
            self.fixed_time_accumulator_seconds -= FIXED_STEP_SECONDS;
            self.run_fixed_tick();
            ticks_this_frame = ticks_this_frame.saturating_add(1);
        }

        if self.fixed_time_accumulator_seconds >= FIXED_STEP_SECONDS {
            self.fixed_step_clamp_count = self.fixed_step_clamp_count.saturating_add(1);
            self.fixed_time_accumulator_seconds = FIXED_STEP_SECONDS * 0.5;
        }

        if self.frame_index % 120 == 0 {
            log_info(&format!(
                "[SIM] fixed-step summary frame={} ticks_this_frame={} accumulator={:.5} clamped_frames={} game_over={} paused={} pending_steps={}",
                self.frame_index,
                ticks_this_frame,
                self.fixed_time_accumulator_seconds,
                self.fixed_step_clamp_count,
                self.game_over,
                self.simulation_paused,
                self.pending_step_ticks
            ));
        }

        if self.frame_index % INPUT_LOG_INTERVAL_FRAMES == 0 {
            log_info(&format!(
                "[INPUT] heartbeat frame={} visible_gamepads={} gamepad_slots={} document_has_focus={} tracked_gamepads={} tracked_buttons={} tracked_axes={}",
                self.frame_index,
                gamepad_stats.visible_gamepads,
                gamepad_stats.slot_count,
                gamepad_stats.document_has_focus,
                self.logged_gamepad_devices.len(),
                self.gamepad_button_states.len(),
                self.gamepad_axis_values.len()
            ));

            if !gamepad_stats.document_has_focus {
                log_warn(
                    "[INPUT][GAMEPAD] document is not focused; gamepad exposure is often blocked until focus is regained",
                );
            }

            if gamepad_stats.slot_count > 0 && gamepad_stats.visible_gamepads == 0 {
                log_warn(
                    "[INPUT][GAMEPAD] browser reports gamepad slots but all entries are null (permission policy, gesture exposure, or browser/device support issue)",
                );
            }

            if gamepad_stats.visible_gamepads == 0 {
                log_warn(
                    "[INPUT][GAMEPAD] no visible gamepads from browser API; this can mean no device, no gamepad user gesture yet, or a stale wasm bundle",
                );
            }
        }
    }

    pub fn render(&mut self) -> Result<(), JsValue> {
        let (batch_vertices, draw_batches) = self.build_batch_vertices();
        let uniform = RotationUniform::identity();
        self.render_core
            .render(
                &self.surface,
                uniform,
                bytemuck::cast_slice(&batch_vertices),
                &draw_batches,
            )
            .map_err(RenderError::into_js_value)
    }
}

fn input_event_from_key_code(
    key_code: winit::keyboard::KeyCode,
    is_pressed: bool,
    is_repeat: bool,
) -> InputEvent {
    InputEvent {
        source: "keyboard".to_owned(),
        control: format!("{key_code:?}"),
        value: if is_pressed { 1.0 } else { 0.0 },
        device_index: None,
        is_pressed,
        is_repeat,
    }
}

fn input_event_from_mouse_button(button: MouseButton, is_pressed: bool) -> InputEvent {
    let control = match button {
        MouseButton::Left => "primary_button".to_owned(),
        MouseButton::Right => "secondary_button".to_owned(),
        MouseButton::Middle => "middle_button".to_owned(),
        MouseButton::Back => "back_button".to_owned(),
        MouseButton::Forward => "forward_button".to_owned(),
        MouseButton::Other(value) => format!("button_{value}"),
    };

    InputEvent {
        source: "mouse".to_owned(),
        control,
        value: if is_pressed { 1.0 } else { 0.0 },
        device_index: None,
        is_pressed,
        is_repeat: false,
    }
}

fn input_event_from_mouse_wheel(delta_y: f32) -> InputEvent {
    InputEvent {
        source: "mouse".to_owned(),
        control: "wheel_y".to_owned(),
        value: delta_y,
        device_index: None,
        is_pressed: false,
        is_repeat: false,
    }
}

fn input_event_from_cursor_position(axis: &str, value: f32) -> InputEvent {
    InputEvent {
        source: "mouse".to_owned(),
        control: format!("cursor_{axis}"),
        value,
        device_index: None,
        is_pressed: false,
        is_repeat: false,
    }
}

fn gamepad_button_control(button_index: usize) -> String {
    match button_index {
        0 => "south_button".to_owned(),
        1 => "east_button".to_owned(),
        2 => "west_button".to_owned(),
        3 => "north_button".to_owned(),
        4 => "left_shoulder".to_owned(),
        5 => "right_shoulder".to_owned(),
        6 => "left_trigger".to_owned(),
        7 => "right_trigger".to_owned(),
        8 => "select_button".to_owned(),
        9 => "start_button".to_owned(),
        10 => "left_stick_button".to_owned(),
        11 => "right_stick_button".to_owned(),
        12 => "dpad_up".to_owned(),
        13 => "dpad_down".to_owned(),
        14 => "dpad_left".to_owned(),
        15 => "dpad_right".to_owned(),
        16 => "home_button".to_owned(),
        _ => format!("button_{button_index}"),
    }
}

fn gamepad_axis_control(axis_index: usize) -> String {
    match axis_index {
        0 => "left_stick_x".to_owned(),
        1 => "left_stick_y".to_owned(),
        2 => "right_stick_x".to_owned(),
        3 => "right_stick_y".to_owned(),
        _ => format!("axis_{axis_index}"),
    }
}

fn gamepad_axis_semantic_alias(axis_index: usize, value: f32) -> Option<(&'static str, f32)> {
    match axis_index {
        4 => Some(("left_trigger", normalize_gamepad_trigger_axis(value))),
        5 => Some(("right_trigger", normalize_gamepad_trigger_axis(value))),
        _ => None,
    }
}

fn normalize_gamepad_trigger_axis(value: f32) -> f32 {
    ((value.clamp(-1.0, 1.0) + 1.0) * 0.5).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{
        gamepad_axis_control, gamepad_axis_semantic_alias, normalize_gamepad_trigger_axis,
    };

    #[test]
    fn trigger_axes_keep_raw_axis_names() {
        assert_eq!(gamepad_axis_control(4), "axis_4");
        assert_eq!(gamepad_axis_control(5), "axis_5");
    }

    #[test]
    fn firefox_trigger_axes_emit_semantic_aliases() {
        assert_eq!(
            gamepad_axis_semantic_alias(4, 1.0),
            Some(("left_trigger", 1.0))
        );
        assert_eq!(
            gamepad_axis_semantic_alias(5, 1.0),
            Some(("right_trigger", 1.0))
        );
        assert_eq!(gamepad_axis_semantic_alias(3, 0.5), None);
    }

    #[test]
    fn trigger_axis_normalization_maps_firefox_range_to_button_range() {
        assert!((normalize_gamepad_trigger_axis(-1.0) - 0.0).abs() < f32::EPSILON);
        assert!((normalize_gamepad_trigger_axis(0.0) - 0.5).abs() < f32::EPSILON);
        assert!((normalize_gamepad_trigger_axis(1.0) - 1.0).abs() < f32::EPSILON);
    }
}
