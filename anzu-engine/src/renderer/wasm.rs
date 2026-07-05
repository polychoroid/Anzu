use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use winit::{
    event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta},
    keyboard::PhysicalKey,
    window::Window,
};

use super::control_plane::{
    ControlCommand, ControlPlaneAction, ControlPlaneState, command_label, is_reserved_browser_key,
};
use super::core::{BatchVertex, DrawBatch, RenderCore, RenderError, RotationUniform};
use crate::ecs::{EntityId, MaterialId, World};
use crate::input::InputEvent;
use crate::renderer::RenderRomPackage;
use crate::simulation::{InteractionEventKind, PhysicsConfig, Scheduler, SimulationModel};

const FIXED_STEP_SECONDS: f32 = 1.0 / 60.0;
const MAX_FIXED_STEPS_PER_FRAME: u32 = 8;
const INPUT_LOG_INTERVAL_FRAMES: u64 = 120;
const WEBGL_RENDER_SCALE: f32 = 0.60;
const ENABLE_RUNTIME_INFO_LOGS: bool = true;
const ENABLE_RUNTIME_WARN_LOGS: bool = true;
const MAX_CONTROL_PLANE_NOTIFICATIONS: usize = 8;
const OVERLAY_MATERIAL_ID: MaterialId = MaterialId(u16::MAX);

struct GamepadPollStats {
    visible_gamepads: usize,
    slot_count: u32,
    document_has_focus: bool,
    document_visible: bool,
    secure_context: bool,
    gamepad_api_available: bool,
}

#[derive(Clone, Copy, Debug)]
struct InputDiagnosticsSnapshot {
    visible_gamepads: usize,
    slot_count: u32,
    document_has_focus: bool,
    document_visible: bool,
    secure_context: bool,
    gamepad_api_available: bool,
}

impl InputDiagnosticsSnapshot {
    fn from_poll_stats(stats: GamepadPollStats) -> Self {
        Self {
            visible_gamepads: stats.visible_gamepads,
            slot_count: stats.slot_count,
            document_has_focus: stats.document_has_focus,
            document_visible: stats.document_visible,
            secure_context: stats.secure_context,
            gamepad_api_available: stats.gamepad_api_available,
        }
    }
}

impl Default for InputDiagnosticsSnapshot {
    fn default() -> Self {
        Self {
            visible_gamepads: 0,
            slot_count: 0,
            document_has_focus: true,
            document_visible: true,
            secure_context: false,
            gamepad_api_available: false,
        }
    }
}

fn diagnostics_bool_text(value: bool) -> &'static str {
    if value { "YES" } else { "NO" }
}

fn overlay_glyph_5x5(character: char) -> [u8; 5] {
    match character.to_ascii_uppercase() {
        'A' => [0b01110, 0b10001, 0b11111, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b11110, 0b10001, 0b11110],
        'C' => [0b01111, 0b10000, 0b10000, 0b10000, 0b01111],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' => [0b11111, 0b10000, 0b11110, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b11110, 0b10000, 0b10000],
        'G' => [0b01111, 0b10000, 0b10011, 0b10001, 0b01110],
        'H' => [0b10001, 0b10001, 0b11111, 0b10001, 0b10001],
        'I' => [0b11111, 0b00100, 0b00100, 0b00100, 0b11111],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10001, 0b10001],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b11110, 0b10000, 0b10000],
        'R' => [0b11110, 0b10001, 0b11110, 0b10010, 0b10001],
        'S' => [0b01111, 0b10000, 0b01110, 0b00001, 0b11110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'X' => [0b10001, 0b01010, 0b00100, 0b01010, 0b10001],
        'Y' => [0b10001, 0b01010, 0b00100, 0b00100, 0b00100],
        '0' => [0b01110, 0b10011, 0b10101, 0b11001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00010, 0b00100, 0b11111],
        '3' => [0b11110, 0b00001, 0b00110, 0b00001, 0b11110],
        '4' => [0b00010, 0b00110, 0b01010, 0b11111, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b11110],
        '6' => [0b01110, 0b10000, 0b11110, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000],
        '8' => [0b01110, 0b10001, 0b01110, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b01111, 0b00001, 0b01110],
        ':' => [0b00000, 0b00100, 0b00000, 0b00100, 0b00000],
        ' ' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
        _ => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
    }
}

fn log_info(message: &str) {
    if ENABLE_RUNTIME_INFO_LOGS {
        web_sys::console::log_1(&JsValue::from_str(message));
    }
}

fn log_warn(message: &str) {
    if ENABLE_RUNTIME_WARN_LOGS {
        web_sys::console::warn_1(&JsValue::from_str(message));
    }
}

impl RenderError {
    fn into_js_value(self) -> JsValue {
        JsValue::from_str(&self.to_string())
    }
}

pub struct State {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    rom: Box<dyn RenderRomPackage>,
    webgl_compat: bool,
    render_scale: f32,
    physics_config: PhysicsConfig,
    world: World,
    anchor_entity: EntityId,
    scheduler: Scheduler,
    frame_index: u64,
    fixed_tick_index: u64,
    next_scheduler_log_frame: u64,
    fixed_time_accumulator_seconds: f32,
    fixed_step_clamp_count: u64,
    smoothed_fps: f32,
    game_over: bool,
    control_plane: ControlPlaneState,
    input_diagnostics: InputDiagnosticsSnapshot,
    logged_gamepad_devices: BTreeSet<u32>,
    gamepad_button_states: BTreeMap<(u32, usize), bool>,
    gamepad_button_values: BTreeMap<(u32, usize), f32>,
    gamepad_axis_values: BTreeMap<(u32, usize), f32>,
    simulation: Box<dyn SimulationModel>,
    render_core: RenderCore,
}

impl State {
    fn scaled_extent(width: u32, height: u32, scale: f32) -> (u32, u32) {
        let scaled_width = ((width as f32) * scale).round() as u32;
        let scaled_height = ((height as f32) * scale).round() as u32;
        (scaled_width.max(1), scaled_height.max(1))
    }

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
        self.world
            .for_each_render_mesh(|entity_id, _mesh, _material_id| {
                if self.world.lifecycle(entity_id).is_some() {
                    if let Some(t) = self.world.transform(entity_id) {
                        bullet_positions.push([t.position_x, t.position_y]);
                    }
                }
            });

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

        let nearest_bullet_light = |px: f32, py: f32| -> f32 {
            let bullet = nearest_bullet(px, py);
            let light_dist = ((bullet[0] - px).powi(2) + (bullet[1] - py).powi(2)).sqrt();
            let light_radius = 0.35f32;
            let t = (light_dist / light_radius).clamp(0.0, 1.0);
            let smooth = t * t * (3.0 - 2.0 * t);
            (1.0 - smooth).powf(1.2)
        };

        let mut vertices_by_material: BTreeMap<MaterialId, Vec<BatchVertex>> = BTreeMap::new();

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
                let entity_light_pos = if self.webgl_compat {
                    [
                        nearest_bullet_light(transform.position_x, transform.position_y),
                        0.0,
                    ]
                } else {
                    nearest_bullet(transform.position_x, transform.position_y)
                };

                let material_vertices = vertices_by_material.entry(material_id).or_default();

                material_vertices.extend(vertices.iter().map(|vertex| {
                    let x = vertex.position[0] * transform.uniform_scale;
                    let y = vertex.position[1] * transform.uniform_scale;

                    let world_x = c * x - s * y + transform.position_x;
                    let world_y = s * x + c * y + transform.position_y;

                    BatchVertex {
                        position: [world_x, world_y],
                        color: vertex.color,
                        barycentric: vertex.barycentric,
                        light_pos: entity_light_pos,
                        edge_mask: vertex.edge_mask,
                    }
                }));
            });

        self.append_overlay_vertices(&mut vertices_by_material);

        let mut batch = Vec::new();
        let mut draw_batches = Vec::new();
        for (material_id, vertices) in vertices_by_material {
            let batch_start = batch.len() as u32;
            batch.extend(vertices);
            let batch_end = batch.len() as u32;
            draw_batches.push(DrawBatch {
                material_id,
                vertex_offset: batch_start,
                vertex_count: batch_end.saturating_sub(batch_start),
            });
        }

        (batch, draw_batches)
    }

    fn overlay_vertex_light_payload(&self, x: f32, y: f32) -> [f32; 2] {
        if self.webgl_compat {
            // WebGL shader path interprets light_pos.x as scalar light strength.
            [0.95, 0.0]
        } else {
            [x, y]
        }
    }

    fn push_overlay_rect(
        &self,
        vertices: &mut Vec<BatchVertex>,
        min_x: f32,
        min_y: f32,
        max_x: f32,
        max_y: f32,
        color: [f32; 3],
    ) {
        let edge_mask = [0.0, 0.0, 0.0];
        let top_left_light = self.overlay_vertex_light_payload(min_x, max_y);
        let bottom_left_light = self.overlay_vertex_light_payload(min_x, min_y);
        let bottom_right_light = self.overlay_vertex_light_payload(max_x, min_y);
        let top_right_light = self.overlay_vertex_light_payload(max_x, max_y);
        vertices.extend_from_slice(&[
            BatchVertex {
                position: [min_x, max_y],
                color,
                barycentric: [1.0, 0.0, 0.0],
                // Overlay quads are rendered with the same shader as gameplay meshes.
                // Setting per-vertex light_pos to the panel position makes interpolation
                // track world_pos, yielding stable alpha/fill for UI geometry.
                light_pos: top_left_light,
                edge_mask,
            },
            BatchVertex {
                position: [min_x, min_y],
                color,
                barycentric: [0.0, 1.0, 0.0],
                light_pos: bottom_left_light,
                edge_mask,
            },
            BatchVertex {
                position: [max_x, min_y],
                color,
                barycentric: [0.0, 0.0, 1.0],
                light_pos: bottom_right_light,
                edge_mask,
            },
            BatchVertex {
                position: [min_x, max_y],
                color,
                barycentric: [1.0, 0.0, 0.0],
                light_pos: top_left_light,
                edge_mask,
            },
            BatchVertex {
                position: [max_x, min_y],
                color,
                barycentric: [0.0, 1.0, 0.0],
                light_pos: bottom_right_light,
                edge_mask,
            },
            BatchVertex {
                position: [max_x, max_y],
                color,
                barycentric: [0.0, 0.0, 1.0],
                light_pos: top_right_light,
                edge_mask,
            },
        ]);
    }

    fn append_overlay_distortion(&self, vertices: &mut Vec<BatchVertex>, time_seconds: f32) {
        const BAND_COUNT: u32 = 18;

        // Layer thin animated strips across the scrim to suggest refractive flow.
        // The offsets are intentionally modest to preserve UI readability.
        for band in 0..BAND_COUNT {
            let t0 = band as f32 / BAND_COUNT as f32;
            let t1 = (band + 1) as f32 / BAND_COUNT as f32;
            let min_y = -1.0 + t0 * 2.0;
            let max_y = -1.0 + t1 * 2.0 + 0.004;

            let phase = time_seconds * 1.6 + t0 * 7.5;
            let flow_offset = phase.sin() * 0.028 + (phase * 0.65).cos() * 0.012;
            let tone = 0.06 + 0.04 * (phase * 0.9).sin().abs();

            self.push_overlay_rect(
                vertices,
                -1.05 + flow_offset,
                min_y,
                1.05 + flow_offset,
                max_y,
                [tone, tone + 0.02, tone + 0.06],
            );
        }

        let ridge_count = 6;
        for ridge in 0..ridge_count {
            let ridge_t = ridge as f32 / ridge_count as f32;
            let drift = (time_seconds * 0.55 + ridge_t * 5.0).sin() * 0.20;
            let center_x = -0.85 + ridge_t * 1.7 + drift;
            let ridge_width = 0.035 + 0.01 * (time_seconds * 1.2 + ridge_t * 3.2).cos().abs();
            let shimmer = 0.12 + 0.05 * (time_seconds * 1.8 + ridge_t * 4.4).sin().abs();

            self.push_overlay_rect(
                vertices,
                center_x - ridge_width,
                -1.0,
                center_x + ridge_width,
                1.0,
                [shimmer, shimmer + 0.02, shimmer + 0.07],
            );
        }

        // Softly restore legibility after distortion bands.
        self.push_overlay_rect(vertices, -1.0, -1.0, 1.0, 1.0, [0.05, 0.09, 0.14]);
    }

    fn append_overlay_vertices(
        &self,
        vertices_by_material: &mut BTreeMap<MaterialId, Vec<BatchVertex>>,
    ) {
        let overlays = self.control_plane.overlay_visibility();
        let pinned_fps_hud = self.control_plane.is_fps_hud_pinned();
        if !overlays.pause_menu
            && !overlays.control
            && !overlays.performance
            && !overlays.notifications
            && !pinned_fps_hud
        {
            return;
        }

        let overlay_vertices = vertices_by_material.entry(OVERLAY_MATERIAL_ID).or_default();
        // Keep the control diagnostics panel readable by reserving full-screen
        // distortion for the performance tab only.
        let tab_overlay_active = overlays.performance;

        if tab_overlay_active {
            let overlay_time_seconds = self.frame_index as f32 / 75.0;
            self.append_overlay_distortion(overlay_vertices, overlay_time_seconds);
        }

        if overlays.pause_menu {
            self.push_overlay_rect(
                overlay_vertices,
                -0.48,
                -0.32,
                0.48,
                0.32,
                [0.10, 0.12, 0.20],
            );
            self.push_overlay_rect(
                overlay_vertices,
                -0.44,
                -0.28,
                0.44,
                0.28,
                [0.18, 0.22, 0.36],
            );
        }

        if overlays.control {
            // Outer border
            self.push_overlay_rect(
                overlay_vertices,
                -0.98,
                0.52,
                -0.42,
                0.96,
                [0.38, 0.76, 0.96],
            );
            // Inner panel
            self.push_overlay_rect(
                overlay_vertices,
                -0.94,
                0.56,
                -0.46,
                0.92,
                [0.10, 0.18, 0.30],
            );

            self.append_control_diagnostic_indicators(overlay_vertices);
        }

        if overlays.performance {
            // Outer border
            self.push_overlay_rect(overlay_vertices, 0.48, 0.74, 0.98, 0.96, [0.40, 0.84, 0.94]);
            // Inner panel
            self.push_overlay_rect(overlay_vertices, 0.52, 0.78, 0.94, 0.92, [0.10, 0.17, 0.22]);

            let fps_fill = (self.smoothed_fps / 120.0).clamp(0.0, 1.0);
            let bar_min_x = 0.56;
            let bar_max_x = bar_min_x + 0.36 * fps_fill;
            let bar_color = if self.smoothed_fps >= 55.0 {
                [0.24, 0.90, 0.36]
            } else if self.smoothed_fps >= 30.0 {
                [0.94, 0.76, 0.18]
            } else {
                [0.90, 0.28, 0.26]
            };

            if bar_max_x > bar_min_x {
                self.push_overlay_rect(
                    overlay_vertices,
                    bar_min_x,
                    0.80,
                    bar_max_x,
                    0.88,
                    bar_color,
                );
            }
        }

        if pinned_fps_hud && !overlays.performance {
            self.append_pinned_fps_text(overlay_vertices);
        }

        if overlays.notifications {
            let notifications = self.control_plane.notifications();
            let panel_min_x = 0.36;
            let panel_max_x = 0.98;
            let panel_min_y = -0.96;
            let panel_max_y = -0.52;
            // Outer border
            self.push_overlay_rect(
                overlay_vertices,
                panel_min_x,
                panel_min_y,
                panel_max_x,
                panel_max_y,
                [0.80, 0.50, 0.30],
            );
            // Inner panel
            self.push_overlay_rect(
                overlay_vertices,
                panel_min_x + 0.04,
                panel_min_y + 0.04,
                panel_max_x - 0.04,
                panel_max_y - 0.04,
                [0.16, 0.10, 0.08],
            );

            for (index, message) in notifications.iter().rev().take(4).enumerate() {
                let top = panel_max_y - 0.05 - index as f32 * 0.09;
                let bottom = top - 0.055;
                let width_scale = ((message.len() % 24) as f32 / 24.0).clamp(0.25, 1.0);
                let strip_max_x =
                    panel_min_x + 0.08 + (panel_max_x - panel_min_x - 0.10) * width_scale;
                let strip_color = match index {
                    0 => [0.98, 0.72, 0.34],
                    1 => [0.88, 0.60, 0.30],
                    2 => [0.76, 0.52, 0.28],
                    _ => [0.64, 0.44, 0.26],
                };
                self.push_overlay_rect(
                    overlay_vertices,
                    panel_min_x + 0.04,
                    bottom,
                    strip_max_x,
                    top,
                    strip_color,
                );
            }
        }
    }

    fn append_control_diagnostic_indicators(&self, vertices: &mut Vec<BatchVertex>) {
        let diagnostics = self.input_diagnostics;
        let panel_min_x = -0.935;
        let panel_max_x = -0.465;

        // Dedicated diagnostics card to separate text from scene/scrim noise.
        // We layer the card fill to increase effective opacity on both WebGPU and WebGL.
        self.push_overlay_rect(
            vertices,
            panel_min_x,
            0.59,
            panel_max_x,
            0.91,
            [0.05, 0.10, 0.16],
        );
        self.push_overlay_rect(
            vertices,
            panel_min_x,
            0.59,
            panel_max_x,
            0.91,
            [0.05, 0.10, 0.16],
        );
        self.push_overlay_rect(
            vertices,
            panel_min_x,
            0.59,
            panel_max_x,
            0.91,
            [0.05, 0.10, 0.16],
        );

        let text_color = [0.74, 0.92, 0.96];
        let text_origin_x = -0.92;
        let mut text_origin_y = 0.885;
        let text_scale = 0.0046;
        let text_step = 0.036;

        self.append_overlay_text_line(
            vertices,
            "INPUT DIAG",
            text_origin_x,
            text_origin_y,
            text_scale,
            [0.86, 0.97, 0.98],
        );

        text_origin_y -= text_step;
        self.append_overlay_text_line(
            vertices,
            &format!(
                "FCS: {}",
                diagnostics_bool_text(diagnostics.document_has_focus)
            ),
            text_origin_x,
            text_origin_y,
            text_scale,
            text_color,
        );

        text_origin_y -= text_step;
        self.append_overlay_text_line(
            vertices,
            &format!(
                "VIS: {}",
                diagnostics_bool_text(diagnostics.document_visible)
            ),
            text_origin_x,
            text_origin_y,
            text_scale,
            text_color,
        );

        text_origin_y -= text_step;
        self.append_overlay_text_line(
            vertices,
            &format!(
                "API: {}",
                diagnostics_bool_text(diagnostics.gamepad_api_available)
            ),
            text_origin_x,
            text_origin_y,
            text_scale,
            text_color,
        );

        text_origin_y -= text_step;
        self.append_overlay_text_line(
            vertices,
            &format!("SEC: {}", diagnostics_bool_text(diagnostics.secure_context)),
            text_origin_x,
            text_origin_y,
            text_scale,
            text_color,
        );

        text_origin_y -= text_step;
        self.append_overlay_text_line(
            vertices,
            &format!("SLT: {}", diagnostics.slot_count),
            text_origin_x,
            text_origin_y,
            text_scale,
            text_color,
        );

        text_origin_y -= text_step;
        self.append_overlay_text_line(
            vertices,
            &format!("PAD: {}", diagnostics.visible_gamepads),
            text_origin_x,
            text_origin_y,
            text_scale,
            text_color,
        );

        text_origin_y -= text_step;
        self.append_overlay_text_line(
            vertices,
            &format!(
                "HUD MODE: {}",
                self.control_plane
                    .hud_telemetry_mode()
                    .label()
                    .to_ascii_uppercase()
            ),
            text_origin_x,
            text_origin_y,
            text_scale,
            text_color,
        );

        text_origin_y -= text_step;
        self.append_overlay_text_line(
            vertices,
            "TOGGLE: F CYCLE",
            text_origin_x,
            text_origin_y,
            text_scale,
            [0.88, 0.86, 0.62],
        );
    }

    fn append_overlay_text_line(
        &self,
        vertices: &mut Vec<BatchVertex>,
        text: &str,
        start_x: f32,
        top_y: f32,
        glyph_size: f32,
        color: [f32; 3],
    ) {
        let mut cursor_x = start_x;

        for character in text.chars() {
            let glyph_rows = overlay_glyph_5x5(character);

            for (row_index, row_bits) in glyph_rows.into_iter().enumerate() {
                for column_index in 0..5 {
                    let bit_mask = 1 << (4 - column_index);
                    if row_bits & bit_mask == 0 {
                        continue;
                    }

                    let min_x = cursor_x + column_index as f32 * glyph_size;
                    let max_x = min_x + glyph_size * 0.86;
                    let row_top = top_y - row_index as f32 * glyph_size;
                    let min_y = row_top - glyph_size * 0.86;

                    self.push_overlay_rect(vertices, min_x, min_y, max_x, row_top, color);
                }
            }

            cursor_x += glyph_size * 6.0;
        }
    }

    fn append_pinned_fps_text(&self, vertices: &mut Vec<BatchVertex>) {
        let hud_min_x = 0.72;
        let hud_max_x = 0.985;
        let hud_min_y = 0.90;
        let hud_max_y = 0.985;

        self.push_overlay_rect(
            vertices,
            hud_min_x,
            hud_min_y,
            hud_max_x,
            hud_max_y,
            [0.10, 0.16, 0.22],
        );
        self.push_overlay_rect(
            vertices,
            hud_min_x,
            hud_min_y,
            hud_max_x,
            hud_max_y,
            [0.10, 0.16, 0.22],
        );

        let fps_text = format!("FPS: {}", self.smoothed_fps.round() as i32);
        self.append_overlay_text_line(
            vertices,
            &fps_text,
            hud_min_x + 0.015,
            hud_max_y - 0.02,
            0.005,
            [0.86, 0.96, 0.98],
        );
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
        let webgl_compat = adapter_info.backend == wgpu::Backend::Gl;
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

        let render_scale = if webgl_compat {
            WEBGL_RENDER_SCALE
        } else {
            1.0
        };
        let (surface_width, surface_height) =
            Self::scaled_extent(size.width.max(1), size.height.max(1), render_scale);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: surface_width,
            height: surface_height,
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);
        log_info(&format!(
            "[RENDER] surface configured: format={:?} size={}x{} present_mode={:?} alpha_mode={:?} scale={:.2}",
            config.format,
            config.width,
            config.height,
            config.present_mode,
            config.alpha_mode,
            render_scale
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
        let render_data = rom.render_data(webgl_compat);
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
            rom,
            webgl_compat,
            render_scale,
            physics_config,
            world,
            anchor_entity,
            scheduler: Scheduler::new(physics_config),
            frame_index: 0,
            fixed_tick_index: 0,
            next_scheduler_log_frame: 0,
            fixed_time_accumulator_seconds: 0.0,
            fixed_step_clamp_count: 0,
            smoothed_fps: 60.0,
            game_over: false,
            control_plane: ControlPlaneState::new(MAX_CONTROL_PLANE_NOTIFICATIONS),
            input_diagnostics: InputDiagnosticsSnapshot::default(),
            logged_gamepad_devices: BTreeSet::new(),
            gamepad_button_states: BTreeMap::new(),
            gamepad_button_values: BTreeMap::new(),
            gamepad_axis_values: BTreeMap::new(),
            simulation,
            render_core,
        })
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn reset_game(&mut self) {
        log_info("[GAME] reset requested: rebuilding world and simulation state");

        self.world = World::new();
        self.anchor_entity = self.rom.bootstrap_world(&mut self.world);
        self.simulation = self.rom.create_simulation(&self.world, self.anchor_entity);
        self.simulation.update(0.0);

        self.scheduler = Scheduler::new(self.physics_config);
        self.frame_index = 0;
        self.fixed_tick_index = 0;
        self.next_scheduler_log_frame = 0;
        self.fixed_time_accumulator_seconds = 0.0;
        self.fixed_step_clamp_count = 0;
        self.smoothed_fps = 60.0;
        self.game_over = false;
        self.control_plane.on_game_reset_complete();

        self.logged_gamepad_devices.clear();
        self.gamepad_button_states.clear();
        self.gamepad_button_values.clear();
        self.gamepad_axis_values.clear();

        log_info(&format!(
            "[GAME] reset complete: new anchor entity {}",
            self.anchor_entity
        ));
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        let (scaled_width, scaled_height) = Self::scaled_extent(width, height, self.render_scale);
        self.render_core
            .resize(&self.surface, scaled_width, scaled_height);
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

        if is_reserved_browser_key(key_code) {
            log_info(&format!(
                "[INPUT][KEYBOARD] reserved browser key={:?}, passing through",
                key_code
            ));
            return;
        }

        if let Some((command, action)) =
            self.control_plane
                .process_key_event(key_code, is_pressed, event.repeat)
        {
            log_info(&format!(
                "[CONTROL] command={} key={:?}",
                command_label(command),
                key_code
            ));
            if action == ControlPlaneAction::ResetRequested {
                self.reset_game();
            }
            return;
        }

        log_info(&format!(
            "[INPUT][KEYBOARD] key {:?} not bound to control, feeding to simulation",
            key_code
        ));

        if !self.control_plane.allows_gameplay_input() {
            log_info(&format!(
                "[INPUT][KEYBOARD] key {:?} consumed by overlay context",
                key_code
            ));
            return;
        }

        self.simulation
            .handle_input_event(input_event_from_key_code(
                key_code,
                is_pressed,
                event.repeat,
            ));
    }

    pub fn set_control_binding(&mut self, command_name: &str, key_name: &str) -> bool {
        self.control_plane
            .set_control_binding(command_name, key_name)
    }

    pub fn handle_mouse_button_event(&mut self, button: MouseButton, state: ElementState) {
        let is_pressed = state == ElementState::Pressed;
        log_info(&format!(
            "[INPUT][MOUSE] button={:?} pressed={}",
            button, is_pressed
        ));

        if !self.control_plane.allows_gameplay_input() {
            log_info("[INPUT][MOUSE] button input consumed by overlay context");
            return;
        }

        self.simulation
            .handle_input_event(input_event_from_mouse_button(button, is_pressed));
    }

    pub fn handle_mouse_wheel_event(&mut self, delta: MouseScrollDelta) {
        let value = match delta {
            MouseScrollDelta::LineDelta(_, y) => y,
            MouseScrollDelta::PixelDelta(position) => position.y as f32,
        };

        if !self.control_plane.allows_gameplay_input() {
            log_info("[INPUT][MOUSE] wheel input consumed by overlay context");
            return;
        }

        self.simulation
            .handle_input_event(input_event_from_mouse_wheel(value));
    }

    pub fn handle_cursor_moved_event(&mut self, x: f32, y: f32) {
        if !self.control_plane.allows_gameplay_input() {
            return;
        }

        self.simulation
            .handle_input_event(input_event_from_cursor_position("x", x));
        self.simulation
            .handle_input_event(input_event_from_cursor_position("y", y));
    }

    fn run_fixed_tick(&mut self) {
        if self.game_over {
            return;
        }

        let should_tick = self.control_plane.should_run_fixed_tick();

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
                "[SIM] fixed_tick={} bodies={} candidates={} pairs_checked={} narrowphase_checks={} collisions={} proximity_enter={} proximity_exit={} despawned={} stage_ns(sync={}, compose={}, broadphase={}, narrowphase={}, resolve={}, reconcile={}, publish={}) momentum=({:.4},{:.4}) angular_momentum={:.4} energy_linear={:.4} energy_angular={:.4}",
                self.fixed_tick_index,
                report.body_count,
                report.candidate_pair_count,
                report.interaction_pairs_checked,
                report.narrowphase_checks,
                collisions,
                proximity_enter,
                proximity_exit,
                report.despawned_entities.len(),
                report.sync_ns,
                report.compose_ns,
                report.broadphase_ns,
                report.narrowphase_ns,
                report.resolve_ns,
                report.reconcile_ns,
                report.publish_ns,
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

            self.next_scheduler_log_frame = self.fixed_tick_index.saturating_add(600);
        }
    }

    fn poll_gamepads(&mut self) -> GamepadPollStats {
        let Some(window) = web_sys::window() else {
            return GamepadPollStats {
                visible_gamepads: 0,
                slot_count: 0,
                document_has_focus: false,
                document_visible: false,
                secure_context: false,
                gamepad_api_available: false,
            };
        };

        let gamepad_api_available = js_sys::Reflect::has(
            &window.navigator().into(),
            &JsValue::from_str("getGamepads"),
        )
        .unwrap_or(false);

        let document_has_focus = window
            .document()
            .and_then(|document| document.has_focus().ok())
            .unwrap_or(true);

        let document_visible = window
            .document()
            .map(|document| document.visibility_state() == web_sys::VisibilityState::Visible)
            .unwrap_or(true);

        let secure_context = window.is_secure_context();

        let Ok(gamepads) = window.navigator().get_gamepads() else {
            log_warn("[INPUT][GAMEPAD] navigator.getGamepads() failed");
            return GamepadPollStats {
                visible_gamepads: 0,
                slot_count: 0,
                document_has_focus,
                document_visible,
                secure_context,
                gamepad_api_available,
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
                self.control_plane.on_controller_connected(device_index);
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
                let control = gamepad_button_control(button_index);
                let mapped_control_command = gamepad_button_control_command(button_index);

                if previous_pressed != is_pressed || (previous_value - value).abs() >= 0.01 {
                    log_info(&format!(
                        "[INPUT][GAMEPAD] dispatch button index={} control={} value={:.3} pressed={}",
                        device_index, control, value, is_pressed
                    ));

                    if let Some(command) = mapped_control_command {
                        if is_pressed && !previous_pressed {
                            let action = self.control_plane.apply_command(command);
                            log_info(&format!(
                                "[CONTROL][GAMEPAD] command={} button={} index={}",
                                command_label(command),
                                control,
                                device_index
                            ));

                            if action == ControlPlaneAction::ResetRequested {
                                self.reset_game();
                            }
                        }
                    } else {
                        if !self.control_plane.allows_gameplay_input() {
                            continue;
                        }

                        self.simulation.handle_input_event(InputEvent {
                            source: "gamepad".to_owned(),
                            control,
                            value,
                            device_index: Some(device_index),
                            is_pressed,
                            is_repeat: false,
                        });
                    }
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
                    if self.control_plane.allows_gameplay_input() {
                        self.simulation.handle_input_event(InputEvent {
                            source: "gamepad".to_owned(),
                            control: gamepad_axis_control(axis_index),
                            value,
                            device_index: Some(device_index),
                            is_pressed: value.abs() >= 0.5,
                            is_repeat: false,
                        });
                    }

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
                        if self.control_plane.allows_gameplay_input() {
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
        let disconnected_indices: Vec<u32> = self
            .logged_gamepad_devices
            .iter()
            .copied()
            .filter(|device_index| {
                !seen_button_keys
                    .keys()
                    .any(|(seen_index, _)| seen_index == device_index)
                    && !seen_axis_keys
                        .keys()
                        .any(|(seen_index, _)| seen_index == device_index)
            })
            .collect();

        self.logged_gamepad_devices.retain(|device_index| {
            seen_button_keys
                .keys()
                .any(|(seen_index, _)| seen_index == device_index)
                || seen_axis_keys
                    .keys()
                    .any(|(seen_index, _)| seen_index == device_index)
        });

        for device_index in disconnected_indices {
            self.control_plane.on_controller_disconnected(device_index);
        }

        GamepadPollStats {
            visible_gamepads,
            slot_count,
            document_has_focus,
            document_visible,
            secure_context,
            gamepad_api_available,
        }
    }

    pub fn update(&mut self, delta_seconds: f32) {
        self.frame_index = self.frame_index.saturating_add(1);
        let gamepad_stats = self.poll_gamepads();
        self.input_diagnostics = InputDiagnosticsSnapshot::from_poll_stats(gamepad_stats);

        if delta_seconds > 0.0 {
            let instant_fps = (1.0 / delta_seconds).clamp(0.0, 240.0);
            self.smoothed_fps = self.smoothed_fps * 0.90 + instant_fps * 0.10;
        }

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
            let overlays = self.control_plane.overlay_visibility();
            log_info(&format!(
                "[SIM] fixed-step summary frame={} ticks_this_frame={} accumulator={:.5} clamped_frames={} game_over={} paused={} pending_steps={} overlays(pause={},control={},perf={},notify={})",
                self.frame_index,
                ticks_this_frame,
                self.fixed_time_accumulator_seconds,
                self.fixed_step_clamp_count,
                self.game_over,
                self.control_plane.is_runtime_paused(),
                self.control_plane.pending_step_ticks(),
                overlays.pause_menu,
                overlays.control,
                overlays.performance,
                overlays.notifications
            ));
        }

        if self.frame_index % INPUT_LOG_INTERVAL_FRAMES == 0 {
            log_info(&format!(
                "[INPUT] heartbeat frame={} visible_gamepads={} gamepad_slots={} document_has_focus={} tracked_gamepads={} tracked_buttons={} tracked_axes={}",
                self.frame_index,
                self.input_diagnostics.visible_gamepads,
                self.input_diagnostics.slot_count,
                self.input_diagnostics.document_has_focus,
                self.logged_gamepad_devices.len(),
                self.gamepad_button_states.len(),
                self.gamepad_axis_values.len()
            ));

            if !self.input_diagnostics.document_has_focus {
                log_warn(
                    "[INPUT][GAMEPAD] document is not focused; gamepad exposure is often blocked until focus is regained",
                );
            }

            if self.input_diagnostics.slot_count > 0 && self.input_diagnostics.visible_gamepads == 0
            {
                log_warn(
                    "[INPUT][GAMEPAD] browser reports gamepad slots but all entries are null (permission policy, gesture exposure, or browser/device support issue)",
                );
            }

            if self.input_diagnostics.visible_gamepads == 0 {
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

fn gamepad_button_control_command(button_index: usize) -> Option<ControlCommand> {
    match button_index {
        8 => Some(ControlCommand::ToggleOverlay),
        9 => Some(ControlCommand::TogglePause),
        4 => Some(ControlCommand::ToggleControl),
        5 => Some(ControlCommand::TogglePerformance),
        _ => None,
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

fn visible_gamepad_ratio(visible_gamepads: usize, slot_count: u32) -> f32 {
    if slot_count == 0 {
        return 0.0;
    }

    (visible_gamepads as f32 / slot_count as f32).clamp(0.0, 1.0)
}

fn normalize_count_for_overlay(value: usize, max_value: usize) -> f32 {
    if max_value == 0 {
        return 0.0;
    }

    (value as f32 / max_value as f32).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{
        gamepad_axis_control, gamepad_axis_semantic_alias, gamepad_button_control_command,
        normalize_count_for_overlay, normalize_gamepad_trigger_axis, overlay_glyph_5x5,
        visible_gamepad_ratio,
    };
    use crate::renderer::control_plane::ControlCommand;

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

    #[test]
    fn control_plane_buttons_map_to_expected_commands() {
        assert_eq!(
            gamepad_button_control_command(4),
            Some(ControlCommand::ToggleControl)
        );
        assert_eq!(
            gamepad_button_control_command(5),
            Some(ControlCommand::TogglePerformance)
        );
        assert_eq!(
            gamepad_button_control_command(8),
            Some(ControlCommand::ToggleOverlay)
        );
        assert_eq!(
            gamepad_button_control_command(9),
            Some(ControlCommand::TogglePause)
        );
        assert_eq!(gamepad_button_control_command(0), None);
    }

    #[test]
    fn visible_gamepad_ratio_returns_zero_when_no_slots_visible() {
        assert!((visible_gamepad_ratio(0, 0) - 0.0).abs() < f32::EPSILON);
        assert!((visible_gamepad_ratio(2, 0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn visible_gamepad_ratio_is_clamped() {
        assert!((visible_gamepad_ratio(1, 2) - 0.5).abs() < f32::EPSILON);
        assert!((visible_gamepad_ratio(5, 2) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn normalize_count_for_overlay_is_clamped() {
        assert!((normalize_count_for_overlay(0, 8) - 0.0).abs() < f32::EPSILON);
        assert!((normalize_count_for_overlay(4, 8) - 0.5).abs() < f32::EPSILON);
        assert!((normalize_count_for_overlay(16, 8) - 1.0).abs() < f32::EPSILON);
        assert!((normalize_count_for_overlay(1, 0) - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn overlay_font_returns_non_empty_for_known_glyph() {
        let glyph = overlay_glyph_5x5('A');
        assert!(glyph.iter().any(|row| *row != 0));
    }

    #[test]
    fn overlay_font_supports_f_character_for_fps_toggle_hint() {
        let glyph = overlay_glyph_5x5('F');
        assert!(glyph.iter().any(|row| *row != 0));
    }
}
