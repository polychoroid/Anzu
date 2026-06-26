use std::sync::Arc;

use wasm_bindgen::prelude::*;

use winit::{
    event::{ElementState, KeyEvent},
    keyboard::PhysicalKey,
    window::Window,
};

use super::core::{BatchVertex, RenderCore, RenderError, RotationUniform};
use crate::ecs::{EntityId, World};
use crate::renderer::RenderRomPackage;
use crate::simulation::{InteractionEventKind, PhysicsConfig, Scheduler, SimulationModel};

const FIXED_STEP_SECONDS: f32 = 1.0 / 60.0;
const MAX_FIXED_STEPS_PER_FRAME: u32 = 8;

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

    fn build_batch_vertices(&self) -> Vec<BatchVertex> {
        let mut batch = Vec::new();

        self.world.for_each_render_mesh(|entity_id, mesh| {
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

            batch.extend(vertices.iter().map(|vertex| {
                let x = vertex.position[0] * transform.uniform_scale;
                let y = vertex.position[1] * transform.uniform_scale;

                let rotated_x = c * x - s * y;
                let rotated_y = s * x + c * y;

                BatchVertex {
                    position: [
                        rotated_x + transform.position_x,
                        rotated_y + transform.position_y,
                    ],
                    color: vertex.color,
                }
            }));
        });

        batch
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

        match key_code {
            winit::keyboard::KeyCode::Backquote => {
                if is_pressed {
                    self.simulation_paused = !self.simulation_paused;
                    log_info(&format!(
                        "[SIM] pause toggled: paused={} fixed_tick={}",
                        self.simulation_paused, self.fixed_tick_index
                    ));
                }
            }
            winit::keyboard::KeyCode::Period => {
                if is_pressed {
                    self.pending_step_ticks = self.pending_step_ticks.saturating_add(1);
                }
            }
            _ => {
                self.simulation
                    .handle_key_event(key_code, is_pressed, event.repeat);
            }
        }
    }

    fn sync_anchor_from_simulation(&mut self) {
        let transform = self.simulation.transform_2d();
        if let Some(entity_transform) = self.world.transform_mut(self.anchor_entity) {
            entity_transform.position_x = transform.position_x;
            entity_transform.position_y = transform.position_y;
            entity_transform.rotation_rad = transform.rotation_rad;
            entity_transform.uniform_scale = transform.uniform_scale;
        }
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

        let report = self
            .scheduler
            .update_world(&mut self.world, FIXED_STEP_SECONDS);

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

        self.simulation.update(FIXED_STEP_SECONDS);
        self.sync_anchor_from_simulation();
    }

    pub fn update(&mut self, delta_seconds: f32) {
        self.frame_index = self.frame_index.saturating_add(1);
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
    }

    pub fn render(&mut self) -> Result<(), JsValue> {
        let batch_vertices = self.build_batch_vertices();
        let uniform = RotationUniform::identity();
        self.render_core
            .render(
                &self.surface,
                uniform,
                bytemuck::cast_slice(&batch_vertices),
                batch_vertices.len() as u32,
            )
            .map_err(RenderError::into_js_value)
    }
}
