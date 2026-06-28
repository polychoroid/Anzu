use std::collections::{BTreeMap, BTreeSet};

use crate::ecs::{
    CollisionBounds, EntityId, Lifecycle, Mesh, MeshAssetId, PolygonCollider, RigidBody, Transform,
    World,
};
use crate::input::InputEvent;
use crate::renderer::{RenderRomPackage, RomRenderData};
use crate::rom::RomPackage;
use crate::simulation::{
    InteractionEventKind, SchedulerFrameReport, SimulationModel, SimulationTransform2D,
};

pub const TRIANGLE_SHADER: &str = r#"
struct RotationUniform {
    angle: f32,
    scale: f32,
    translation: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> u_rotation: RotationUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
    @location(2) barycentric: vec3<f32>,
    @location(3) light_pos: vec2<f32>,
    @location(4) edge_mask: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) barycentric: vec3<f32>,
    @location(2) world_pos: vec2<f32>,
    @location(3) light_pos: vec2<f32>,
    @location(4) edge_mask: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let c = cos(u_rotation.angle);
    let s = sin(u_rotation.angle);
    let rotated = vec2<f32>(
        c * in.position.x - s * in.position.y,
        s * in.position.x + c * in.position.y,
    ) * u_rotation.scale;
    let translated = rotated + u_rotation.translation;

    var out: VertexOutput;
    out.position = vec4<f32>(translated, 0.0, 1.0);
    out.color = in.color;
    out.barycentric = in.barycentric;
    out.world_pos = translated;
    out.light_pos = in.light_pos;
    out.edge_mask = in.edge_mask;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Edge glow from barycentric coordinates with per-edge masking.
    // This allows us to hide internal triangulation diagonals.
    let EDGE_WIDTH = 0.15;
    let edge_x = (1.0 - smoothstep(0.0, EDGE_WIDTH, in.barycentric.x)) * in.edge_mask.x;
    let edge_y = (1.0 - smoothstep(0.0, EDGE_WIDTH, in.barycentric.y)) * in.edge_mask.y;
    let edge_z = (1.0 - smoothstep(0.0, EDGE_WIDTH, in.barycentric.z)) * in.edge_mask.z;
    let base_alpha = max(edge_x, max(edge_y, edge_z));

    // Bullet light: illuminates nearby geometry based on world-space distance
    let light_dist = distance(in.world_pos, in.light_pos);
    let LIGHT_RADIUS = 0.35;
    let light_strength = 1.0 - smoothstep(0.0, LIGHT_RADIUS, light_dist);

    let glow_brightness = 1.3;
    let lit_color = clamp(in.color * glow_brightness + light_strength * 0.6, vec3<f32>(0.0), vec3<f32>(2.0));
    let alpha = clamp(base_alpha + light_strength * 0.4, 0.0, 1.0);

    return vec4<f32>(lit_color, alpha);
}
"#;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
    pub barycentric: [f32; 3],
    pub light_pos: [f32; 2],
    pub edge_mask: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x3,
        2 => Float32x3,
        3 => Float32x2,
        4 => Float32x3
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub const TRIANGLE_VERTICES: [Vertex; 3] = [
    Vertex {
        position: [0.6, 0.0],
        color: [1.0, 0.2, 0.2],
        barycentric: [1.0, 0.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    Vertex {
        position: [-0.3, 0.5],
        color: [0.2, 1.0, 0.2],
        barycentric: [0.0, 1.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    Vertex {
        position: [-0.3, -0.5],
        color: [0.2, 0.4, 1.0],
        barycentric: [0.0, 0.0, 1.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
];

pub const SQUARE_VERTICES: [Vertex; 6] = [
    Vertex {
        position: [-0.35, -0.35],
        color: [1.0, 0.8, 0.2],
        barycentric: [1.0, 0.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.35, -0.35],
        color: [1.0, 0.8, 0.2],
        barycentric: [0.0, 1.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.35, 0.35],
        color: [1.0, 0.8, 0.2],
        barycentric: [0.0, 0.0, 1.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.35, -0.35],
        color: [1.0, 0.8, 0.2],
        barycentric: [1.0, 0.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.35, 0.35],
        color: [1.0, 0.8, 0.2],
        barycentric: [0.0, 1.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 0.0],
    },
    Vertex {
        position: [-0.35, 0.35],
        color: [1.0, 0.8, 0.2],
        barycentric: [0.0, 0.0, 1.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 0.0],
    },
];

pub const DIAMOND_VERTICES: [Vertex; 6] = [
    Vertex {
        position: [0.0, 0.45],
        color: [0.8, 0.3, 1.0],
        barycentric: [1.0, 0.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.3, 0.0],
        color: [0.8, 0.3, 1.0],
        barycentric: [0.0, 1.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.0, -0.45],
        color: [0.8, 0.3, 1.0],
        barycentric: [0.0, 0.0, 1.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.0, 0.45],
        color: [0.8, 0.3, 1.0],
        barycentric: [1.0, 0.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.0, -0.45],
        color: [0.8, 0.3, 1.0],
        barycentric: [0.0, 0.0, 1.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.3, 0.0],
        color: [0.8, 0.3, 1.0],
        barycentric: [0.0, 1.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 0.0, 1.0],
    },
];

pub const TRIANGLE_COLLIDER: [[f32; 2]; 3] = [[0.6, 0.0], [-0.3, 0.5], [-0.3, -0.5]];
pub const SQUARE_COLLIDER: [[f32; 2]; 4] =
    [[-0.35, -0.35], [0.35, -0.35], [0.35, 0.35], [-0.35, 0.35]];
pub const DIAMOND_COLLIDER: [[f32; 2]; 4] = [[0.0, 0.45], [0.3, 0.0], [0.0, -0.45], [-0.3, 0.0]];

const MESH_TRIANGLE: MeshAssetId = MeshAssetId(1);
const MESH_SQUARE: MeshAssetId = MeshAssetId(2);
const MESH_DIAMOND: MeshAssetId = MeshAssetId(3);

const ASTEROID_SPAWN_PRESETS: [(f32, f32, f32, f32, f32); 8] = [
    // All spawn from right edge, moving left with vertical jitter
    // (x_pos, y_pos, vel_x, vel_y, angular_vel)
    (0.95, -0.90, -0.45, -0.12, 0.35),
    (0.95, -0.60, -0.48, -0.08, -0.42),
    (0.95, -0.30, -0.42, -0.04, 0.38),
    (0.95, 0.00, -0.50, 0.00, -0.36),
    (0.95, 0.30, -0.43, 0.04, 0.30),
    (0.95, 0.60, -0.47, 0.08, -0.40),
    (0.95, 0.90, -0.44, 0.12, 0.32),
    (0.95, 0.15, -0.46, 0.06, -0.28),
];

#[derive(Clone, Copy)]
pub struct TriangleManSpec {
    pub thrust_accel_units_per_sec2: f32,
    pub reverse_thrust_scale: f32,
    pub angular_accel_rad_per_sec2: f32,
    pub linear_damping_per_sec: f32,
    pub angular_damping_per_sec: f32,
    pub fire_cooldown_seconds: f32,
    pub bullet_speed_units_per_sec: f32,
    pub bullet_lifetime_seconds: f32,
    pub asteroid_spawn_interval_seconds: f32,
    pub asteroid_spawn_min_interval_seconds: f32,
    pub asteroid_spawn_accel_per_second: f32,
    pub asteroid_spawn_max_burst: u32,
    pub asteroid_target_count: u32,
    pub asteroid_target_time_seconds: f32,
    pub asteroid_mass_scale_at_target: f32,
    pub scale: f32,
}

impl Default for TriangleManSpec {
    fn default() -> Self {
        Self {
            thrust_accel_units_per_sec2: 0.35,
            reverse_thrust_scale: 0.25,
            angular_accel_rad_per_sec2: 6.28,
            linear_damping_per_sec: 0.15,
            angular_damping_per_sec: 0.65,
            fire_cooldown_seconds: 0.12,
            bullet_speed_units_per_sec: 0.9,
            bullet_lifetime_seconds: 2.0,
            asteroid_spawn_interval_seconds: 1.0,
            asteroid_spawn_min_interval_seconds: 0.45,
            asteroid_spawn_accel_per_second: 2.0,
            asteroid_spawn_max_burst: 25,
            asteroid_target_count: 1000,
            asteroid_target_time_seconds: 300.0,
            asteroid_mass_scale_at_target: 8.0,
            scale: 0.1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TriangleManAction {
    ThrustForward,
    ThrustReverse,
    TurnLeft,
    TurnRight,
    Fire,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TriangleManAxisAction {
    Thrust,
    Turn,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TriangleManBindingKind {
    Digital(TriangleManAction),
    Analog {
        axis: TriangleManAxisAction,
        invert: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TriangleManInputBinding {
    source: &'static str,
    control: &'static str,
    kind: TriangleManBindingKind,
}

struct TriangleManInputProfile {
    bindings: &'static [TriangleManInputBinding],
}

const TRIANGLE_MAN_INPUT_BINDINGS: [TriangleManInputBinding; 11] = [
    TriangleManInputBinding {
        source: "keyboard",
        control: "KeyW",
        kind: TriangleManBindingKind::Digital(TriangleManAction::ThrustForward),
    },
    TriangleManInputBinding {
        source: "keyboard",
        control: "KeyS",
        kind: TriangleManBindingKind::Digital(TriangleManAction::ThrustReverse),
    },
    TriangleManInputBinding {
        source: "keyboard",
        control: "KeyA",
        kind: TriangleManBindingKind::Digital(TriangleManAction::TurnLeft),
    },
    TriangleManInputBinding {
        source: "keyboard",
        control: "KeyD",
        kind: TriangleManBindingKind::Digital(TriangleManAction::TurnRight),
    },
    TriangleManInputBinding {
        source: "keyboard",
        control: "Space",
        kind: TriangleManBindingKind::Digital(TriangleManAction::Fire),
    },
    TriangleManInputBinding {
        source: "gamepad",
        control: "south_button",
        kind: TriangleManBindingKind::Digital(TriangleManAction::Fire),
    },
    TriangleManInputBinding {
        source: "gamepad",
        control: "left_stick_x",
        kind: TriangleManBindingKind::Analog {
            axis: TriangleManAxisAction::Turn,
            invert: false,
        },
    },
    TriangleManInputBinding {
        source: "gamepad",
        control: "left_stick_y",
        kind: TriangleManBindingKind::Analog {
            axis: TriangleManAxisAction::Thrust,
            invert: true,
        },
    },
    TriangleManInputBinding {
        source: "gamepad",
        control: "left_trigger",
        kind: TriangleManBindingKind::Analog {
            axis: TriangleManAxisAction::Thrust,
            invert: true,
        },
    },
    TriangleManInputBinding {
        source: "gamepad",
        control: "right_trigger",
        kind: TriangleManBindingKind::Analog {
            axis: TriangleManAxisAction::Thrust,
            invert: false,
        },
    },
    TriangleManInputBinding {
        source: "mouse",
        control: "primary_button",
        kind: TriangleManBindingKind::Digital(TriangleManAction::Fire),
    },
];

const TRIANGLE_MAN_INPUT_PROFILE: TriangleManInputProfile = TriangleManInputProfile {
    bindings: &TRIANGLE_MAN_INPUT_BINDINGS,
};

#[derive(Clone, Copy, Default)]
struct TriangleManInputFrame {
    thrust: f32,
    turn: f32,
    fire: bool,
}

struct TriangleManInputManager {
    input_profile: &'static TriangleManInputProfile,
    thrust_forward: bool,
    thrust_reverse: bool,
    turn_left: bool,
    turn_right: bool,
    analog_values: BTreeMap<(&'static str, &'static str), f32>,
    fire_key_down: bool,
    pending_fire: bool,
}

fn log_input_debug(message: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(message));
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        eprintln!("{}", message);
    }
}

const TRIGGER_PRESS_THRESHOLD: f32 = 0.55;
const TRIGGER_RELEASE_THRESHOLD: f32 = 0.45;

impl Default for TriangleManInputManager {
    fn default() -> Self {
        Self {
            input_profile: &TRIANGLE_MAN_INPUT_PROFILE,
            thrust_forward: false,
            thrust_reverse: false,
            turn_left: false,
            turn_right: false,
            analog_values: BTreeMap::new(),
            fire_key_down: false,
            pending_fire: false,
        }
    }
}

impl TriangleManInputManager {
    fn handle_input_event(&mut self, event: &InputEvent) {
        if event.is_repeat && event.is_pressed {
            return;
        }

        for binding in self.input_profile.bindings {
            if binding.source != event.source || binding.control != event.control {
                continue;
            }

            if event.source == "gamepad" {
                log_input_debug(&format!(
                    "[INPUT][TRIANGLE_MAN] matched control={} value={:.3} pressed={} repeat={}",
                    event.control, event.value, event.is_pressed, event.is_repeat
                ));
            }

            match binding.kind {
                TriangleManBindingKind::Digital(action) => match action {
                    TriangleManAction::ThrustForward => self.thrust_forward = event.is_pressed,
                    TriangleManAction::ThrustReverse => self.thrust_reverse = event.is_pressed,
                    TriangleManAction::TurnLeft => self.turn_left = event.is_pressed,
                    TriangleManAction::TurnRight => self.turn_right = event.is_pressed,
                    TriangleManAction::Fire => {
                        if event.is_pressed && !self.fire_key_down {
                            self.pending_fire = true;
                        }
                        self.fire_key_down = event.is_pressed;
                    }
                },
                TriangleManBindingKind::Analog { axis, invert } => {
                    let mut value = event.value.clamp(-1.0, 1.0);
                    if invert {
                        value = -value;
                    }
                    value = apply_analog_filter(
                        binding.control,
                        value,
                        self.analog_values
                            .get(&(binding.source, binding.control))
                            .copied(),
                    );

                    if event.source == "gamepad" {
                        log_input_debug(&format!(
                            "[INPUT][TRIANGLE_MAN] filtered analog control={} stored_value={:.3}",
                            binding.control, value
                        ));
                    }

                    self.analog_values.insert(
                        (binding.source, binding.control),
                        analog_axis_value(axis, value),
                    );
                }
            }
        }
    }

    fn snapshot_frame(&mut self) -> TriangleManInputFrame {
        let digital_thrust = axis_value(self.thrust_reverse, self.thrust_forward) as f32;
        let digital_turn = axis_value(self.turn_left, self.turn_right) as f32;

        let mut analog_thrust = 0.0f32;
        let mut analog_turn = 0.0f32;

        for binding in self.input_profile.bindings {
            let TriangleManBindingKind::Analog { axis, .. } = binding.kind else {
                continue;
            };

            let Some(value) = self
                .analog_values
                .get(&(binding.source, binding.control))
                .copied()
            else {
                continue;
            };

            match axis {
                TriangleManAxisAction::Thrust => analog_thrust += value,
                TriangleManAxisAction::Turn => analog_turn += value,
            }
        }

        analog_thrust = analog_thrust.clamp(-1.0, 1.0);
        analog_turn = analog_turn.clamp(-1.0, 1.0);

        let thrust = if analog_thrust.abs() > digital_thrust.abs() {
            analog_thrust
        } else {
            digital_thrust
        };

        let turn = if analog_turn.abs() > digital_turn.abs() {
            analog_turn
        } else {
            digital_turn
        };

        let frame = TriangleManInputFrame {
            thrust,
            turn,
            fire: self.pending_fire,
        };

        self.pending_fire = false;
        frame
    }
}

fn analog_axis_value(_axis: TriangleManAxisAction, value: f32) -> f32 {
    value
}

fn apply_analog_filter(control: &str, value: f32, previous: Option<f32>) -> f32 {
    match control {
        "left_trigger" | "right_trigger" => {
            apply_trigger_hysteresis(value, previous.unwrap_or(0.0))
        }
        _ => apply_deadzone(value, 0.2),
    }
}

fn apply_trigger_hysteresis(value: f32, previous: f32) -> f32 {
    let magnitude = value.abs();
    let previous_active = previous.abs() >= TRIGGER_RELEASE_THRESHOLD;

    if previous_active {
        if magnitude < TRIGGER_RELEASE_THRESHOLD {
            0.0
        } else {
            value
        }
    } else if magnitude < TRIGGER_PRESS_THRESHOLD {
        0.0
    } else {
        value
    }
}

fn axis_value(negative: bool, positive: bool) -> i8 {
    match (negative, positive) {
        (true, false) => -1,
        (false, true) => 1,
        _ => 0,
    }
}

fn apply_deadzone(value: f32, deadzone: f32) -> f32 {
    if value.abs() < deadzone {
        0.0
    } else {
        value
    }
}

trait GameplayRole {
    fn despawn_on_collision_with(&self, _other: &dyn GameplayRole) -> bool {
        false
    }

    fn is_bullet(&self) -> bool {
        false
    }
}

struct PlayerRole;
struct AsteroidRole;
struct BulletRole;

impl GameplayRole for PlayerRole {}
impl GameplayRole for AsteroidRole {}

impl GameplayRole for BulletRole {
    fn despawn_on_collision_with(&self, other: &dyn GameplayRole) -> bool {
        !other.is_bullet()
    }

    fn is_bullet(&self) -> bool {
        true
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RoleKind {
    Player,
    Asteroid,
    Bullet,
}

impl RoleKind {
    fn create_role(self) -> Box<dyn GameplayRole> {
        match self {
            RoleKind::Player => Box::new(PlayerRole),
            RoleKind::Asteroid => Box::new(AsteroidRole),
            RoleKind::Bullet => Box::new(BulletRole),
        }
    }
}

#[derive(Default)]
pub struct TriangleManRom;

impl RomPackage for TriangleManRom {
    fn rom_id(&self) -> &'static str {
        "anzu.triangle_man"
    }

    fn bootstrap_world(&self, world: &mut World) -> EntityId {
        world.register_mesh_asset(
            MESH_TRIANGLE,
            Mesh {
                vertex_bytes: bytemuck::cast_slice(&TRIANGLE_VERTICES),
            },
        );
        world.register_mesh_asset(
            MESH_SQUARE,
            Mesh {
                vertex_bytes: bytemuck::cast_slice(&SQUARE_VERTICES),
            },
        );
        world.register_mesh_asset(
            MESH_DIAMOND,
            Mesh {
                vertex_bytes: bytemuck::cast_slice(&DIAMOND_VERTICES),
            },
        );

        let spec = TriangleManSpec::default();
        spawn_player_entity(world, spec.scale, -0.7, -0.7)
    }

    fn create_simulation(
        &self,
        world: &World,
        anchor_entity: EntityId,
    ) -> Box<dyn SimulationModel> {
        let initial = world.transform(anchor_entity).copied().unwrap_or_default();
        Box::new(TriangleManSimulation::new(
            anchor_entity,
            TriangleManSpec::default(),
            initial,
        ))
    }
}

impl RenderRomPackage for TriangleManRom {
    fn render_data(&self) -> RomRenderData {
        RomRenderData {
            shader_source_wgsl: TRIANGLE_SHADER,
            vertex_layout: Vertex::desc(),
            vertex_bytes: bytemuck::cast_slice(&TRIANGLE_VERTICES),
            vertex_count: TRIANGLE_VERTICES.len() as u32,
        }
    }
}

pub struct TriangleManSimulation {
    anchor_entity: EntityId,
    spec: TriangleManSpec,
    position_x: f32,
    position_y: f32,
    angle_rad: f32,
    velocity_x: f32,
    velocity_y: f32,
    angular_velocity: f32,
    input_manager: TriangleManInputManager,
    fire_cooldown_seconds: f32,
    elapsed_seconds: f32,
    asteroid_spawn_timer_seconds: f32,
    asteroid_spawn_index: usize,
    asteroid_spawned_total: u32,
    baseline_asteroid_spawned: bool,
    fire_requested: bool,
    roles: BTreeMap<EntityId, RoleKind>,
}

impl TriangleManSimulation {
    pub fn new(anchor_entity: EntityId, spec: TriangleManSpec, initial: Transform) -> Self {
        let mut roles = BTreeMap::new();
        roles.insert(anchor_entity, RoleKind::Player);

        Self {
            anchor_entity,
            spec,
            position_x: initial.position_x,
            position_y: initial.position_y,
            angle_rad: initial.rotation_rad,
            velocity_x: initial.velocity_x,
            velocity_y: initial.velocity_y,
            angular_velocity: initial.angular_velocity,
            input_manager: TriangleManInputManager::default(),
            fire_cooldown_seconds: 0.0,
            elapsed_seconds: 0.0,
            asteroid_spawn_timer_seconds: spec.asteroid_spawn_interval_seconds,
            asteroid_spawn_index: 1,
            asteroid_spawned_total: 0,
            baseline_asteroid_spawned: false,
            fire_requested: false,
            roles,
        }
    }

    fn spawn_asteroid_entity(
        &mut self,
        world: &mut World,
        spawn_index: usize,
        scale: f32,
        mass_scale: f32,
    ) -> EntityId {
        let preset = ASTEROID_SPAWN_PRESETS[spawn_index % ASTEROID_SPAWN_PRESETS.len()];
        let entity_id = world.spawn();
        let radius = max_radius(&SQUARE_COLLIDER, scale);
        let mass = 2.2 * mass_scale.max(1.0);
        let moment_of_inertia = 0.5 * mass * radius * radius;

        world.set_transform(
            entity_id,
            Transform {
                position_x: preset.0,
                position_y: preset.1,
                rotation_rad: 0.0,
                uniform_scale: scale,
                velocity_x: preset.2,
                velocity_y: preset.3,
                angular_velocity: preset.4,
            },
        );
        world.set_mesh_instance(entity_id, MESH_SQUARE);
        world.set_collision_bounds(
            entity_id,
            CollisionBounds {
                proximity_radius: radius * 1.15,
            },
        );
        world.set_polygon_collider(
            entity_id,
            PolygonCollider {
                local_vertices: &SQUARE_COLLIDER,
            },
        );
        world.set_rigid_body(
            entity_id,
            RigidBody {
                mass,
                inverse_mass: 1.0 / mass,
                restitution: 0.9,
                friction: 0.06,
                moment_of_inertia,
                inverse_moment_of_inertia: 1.0 / moment_of_inertia,
            },
        );

        self.roles.insert(entity_id, RoleKind::Asteroid);
        self.asteroid_spawned_total = self.asteroid_spawned_total.saturating_add(1);
        entity_id
    }

    fn spawn_bullet_entity(
        &mut self,
        world: &mut World,
        origin_x: f32,
        origin_y: f32,
        angle_rad: f32,
        inherited_velocity_x: f32,
        inherited_velocity_y: f32,
    ) -> EntityId {
        let entity_id = world.spawn();
        let scale = 0.06;
        let radius = max_radius(&DIAMOND_COLLIDER, scale);
        let heading_x = angle_rad.cos();
        let heading_y = angle_rad.sin();
        let spawn_offset = 0.12;

        let mass = 1.00;
        let moment_of_inertia = 0.5 * mass * radius * radius;
        world.set_transform(
            entity_id,
            Transform {
                position_x: origin_x + heading_x * spawn_offset,
                position_y: origin_y + heading_y * spawn_offset,
                rotation_rad: angle_rad,
                uniform_scale: scale,
                velocity_x: inherited_velocity_x + heading_x * self.spec.bullet_speed_units_per_sec,
                velocity_y: inherited_velocity_y + heading_y * self.spec.bullet_speed_units_per_sec,
                angular_velocity: 0.0,
            },
        );
        world.set_mesh_instance(entity_id, MESH_DIAMOND);
        world.set_collision_bounds(
            entity_id,
            CollisionBounds {
                proximity_radius: radius * 1.2,
            },
        );
        world.set_polygon_collider(
            entity_id,
            PolygonCollider {
                local_vertices: &DIAMOND_COLLIDER,
            },
        );
        world.set_rigid_body(
            entity_id,
            RigidBody {
                mass,
                inverse_mass: 1.0 / mass,
                restitution: 0.2,
                friction: 0.01,
                moment_of_inertia,
                inverse_moment_of_inertia: 1.0 / moment_of_inertia,
            },
        );
        world.set_lifecycle(
            entity_id,
            Lifecycle {
                ttl_seconds: self.spec.bullet_lifetime_seconds,
            },
        );

        self.roles.insert(entity_id, RoleKind::Bullet);
        entity_id
    }

    fn role_for_entity(&self, entity_id: EntityId) -> Option<Box<dyn GameplayRole>> {
        self.roles
            .get(&entity_id)
            .copied()
            .map(RoleKind::create_role)
    }

    fn asteroid_spawn_interval(&self) -> f32 {
        let speedup =
            1.0 + self.elapsed_seconds.max(0.0) * self.spec.asteroid_spawn_accel_per_second;
        (self.spec.asteroid_spawn_interval_seconds / speedup)
            .max(self.spec.asteroid_spawn_min_interval_seconds)
    }

    fn asteroid_target_spawn_total(&self) -> u32 {
        let target_total = self.spec.asteroid_target_count.max(1);
        let target_time = self.spec.asteroid_target_time_seconds;

        if target_time <= 0.0 {
            return target_total;
        }

        let progress = (self.elapsed_seconds.max(0.0) / target_time).clamp(0.0, 1.0);
        let baseline = 1u32;
        let additional_target = target_total.saturating_sub(baseline);
        baseline + ((additional_target as f32) * progress).floor() as u32
    }

    fn asteroid_spawn_burst_count(&self) -> u32 {
        let missing = self
            .asteroid_target_spawn_total()
            .saturating_sub(self.asteroid_spawned_total);
        missing.clamp(1, self.spec.asteroid_spawn_max_burst.max(1))
    }

    fn asteroid_mass_scale(&self) -> f32 {
        let target_time = self.spec.asteroid_target_time_seconds;
        if target_time <= 0.0 {
            return self.spec.asteroid_mass_scale_at_target.max(1.0);
        }

        let slope = (self.spec.asteroid_mass_scale_at_target.max(1.0) - 1.0) / target_time;
        (1.0 + self.elapsed_seconds.max(0.0) * slope).max(1.0)
    }
}

impl SimulationModel for TriangleManSimulation {
    fn handle_input_event(&mut self, event: InputEvent) {
        self.input_manager.handle_input_event(&event);
    }

    fn sync_anchor_from_world(&mut self, world: &World, anchor_entity: EntityId) {
        if let Some(transform) = world.transform(anchor_entity) {
            self.position_x = transform.position_x;
            self.position_y = transform.position_y;
            self.angle_rad = transform.rotation_rad;
            self.velocity_x = transform.velocity_x;
            self.velocity_y = transform.velocity_y;
            self.angular_velocity = transform.angular_velocity;
        }
    }

    fn write_anchor_to_world(&self, world: &mut World, anchor_entity: EntityId) {
        if let Some(transform) = world.transform_mut(anchor_entity) {
            transform.position_x = self.position_x;
            transform.position_y = self.position_y;
            transform.rotation_rad = self.angle_rad;
            transform.uniform_scale = self.spec.scale;
            transform.velocity_x = self.velocity_x;
            transform.velocity_y = self.velocity_y;
            transform.angular_velocity = self.angular_velocity;
        }
    }

    fn update(&mut self, delta_seconds: f32) {
        if delta_seconds <= 0.0 {
            return;
        }

        self.elapsed_seconds += delta_seconds;

        let frame = self.input_manager.snapshot_frame();
        let thrust_accel = if frame.thrust > 0.0 {
            self.spec.thrust_accel_units_per_sec2 * frame.thrust
        } else if frame.thrust < 0.0 {
            self.spec.thrust_accel_units_per_sec2 * frame.thrust * self.spec.reverse_thrust_scale
        } else {
            0.0
        };

        let heading_x = self.angle_rad.cos();
        let heading_y = self.angle_rad.sin();
        self.velocity_x += heading_x * thrust_accel * delta_seconds;
        self.velocity_y += heading_y * thrust_accel * delta_seconds;

        let angular_accel = -frame.turn * self.spec.angular_accel_rad_per_sec2;
        self.angular_velocity += angular_accel * delta_seconds;

        let linear_drag = (1.0 - self.spec.linear_damping_per_sec * delta_seconds).clamp(0.0, 1.0);
        self.velocity_x *= linear_drag;
        self.velocity_y *= linear_drag;
        let angular_drag =
            (1.0 - self.spec.angular_damping_per_sec * delta_seconds).clamp(0.0, 1.0);
        self.angular_velocity *= angular_drag;

        self.fire_cooldown_seconds = (self.fire_cooldown_seconds - delta_seconds).max(0.0);
        self.asteroid_spawn_timer_seconds -= delta_seconds;

        if frame.fire && self.fire_cooldown_seconds <= 0.0 {
            self.fire_requested = true;
            self.fire_cooldown_seconds = self.spec.fire_cooldown_seconds;
        }
    }

    fn reconcile_world(
        &mut self,
        world: &mut World,
        _delta_seconds: f32,
        frame_report: &SchedulerFrameReport,
    ) {
        self.roles
            .retain(|entity_id, _| world.transform(*entity_id).is_some());

        if !self.baseline_asteroid_spawned {
            let mass_scale = self.asteroid_mass_scale();
            self.spawn_asteroid_entity(world, 0, 0.14, mass_scale);
            self.baseline_asteroid_spawned = true;
        }

        let mut bullets_to_despawn = BTreeSet::new();
        for event in &frame_report.interaction_events {
            if event.kind != InteractionEventKind::Collision {
                continue;
            }

            let Some(role_a) = self.role_for_entity(event.entity_a) else {
                continue;
            };
            let Some(role_b) = self.role_for_entity(event.entity_b) else {
                continue;
            };

            if role_a.despawn_on_collision_with(role_b.as_ref()) {
                bullets_to_despawn.insert(event.entity_a);
            }
            if role_b.despawn_on_collision_with(role_a.as_ref()) {
                bullets_to_despawn.insert(event.entity_b);
            }
        }

        for entity_id in bullets_to_despawn {
            world.despawn(entity_id);
            self.roles.remove(&entity_id);
        }

        let dynamic_interval = self.asteroid_spawn_interval();
        let spawn_burst = self.asteroid_spawn_burst_count();
        let mass_scale = self.asteroid_mass_scale();

        while self.asteroid_spawn_timer_seconds <= 0.0 {
            for _ in 0..spawn_burst {
                self.spawn_asteroid_entity(world, self.asteroid_spawn_index, 0.12, mass_scale);
                self.asteroid_spawn_index = self.asteroid_spawn_index.saturating_add(1);
            }
            self.asteroid_spawn_timer_seconds += dynamic_interval;
        }

        if self.fire_requested {
            self.spawn_bullet_entity(
                world,
                self.position_x,
                self.position_y,
                self.angle_rad,
                self.velocity_x,
                self.velocity_y,
            );
            self.fire_requested = false;
        }

        if world.transform(self.anchor_entity).is_none() {
            self.roles.remove(&self.anchor_entity);
        }
    }

    fn transform_2d(&self) -> SimulationTransform2D {
        SimulationTransform2D {
            position_x: self.position_x,
            position_y: self.position_y,
            rotation_rad: self.angle_rad,
            uniform_scale: self.spec.scale,
        }
    }
}

fn max_radius(collider: &'static [[f32; 2]], scale: f32) -> f32 {
    collider
        .iter()
        .map(|vertex| (vertex[0] * vertex[0] + vertex[1] * vertex[1]).sqrt())
        .fold(0.0f32, f32::max)
        * scale
}

fn spawn_player_entity(
    world: &mut World,
    scale: f32,
    position_x: f32,
    position_y: f32,
) -> EntityId {
    let entity_id = world.spawn();
    let radius = max_radius(&TRIANGLE_COLLIDER, scale);
    let mass = 1.0;
    let moment_of_inertia = 0.5 * mass * radius * radius;

    world.set_transform(
        entity_id,
        Transform {
            position_x,
            position_y,
            rotation_rad: 0.0,
            uniform_scale: scale,
            velocity_x: 0.0,
            velocity_y: 0.0,
            angular_velocity: 0.0,
        },
    );
    world.set_mesh_instance(entity_id, MESH_TRIANGLE);
    world.set_collision_bounds(
        entity_id,
        CollisionBounds {
            proximity_radius: radius * 1.6,
        },
    );
    world.set_polygon_collider(
        entity_id,
        PolygonCollider {
            local_vertices: &TRIANGLE_COLLIDER,
        },
    );
    world.set_rigid_body(
        entity_id,
        RigidBody {
            mass,
            inverse_mass: 1.0 / mass,
            restitution: 0.5,
            friction: 0.12,
            moment_of_inertia,
            inverse_moment_of_inertia: 1.0 / moment_of_inertia,
        },
    );

    entity_id
}

#[cfg(test)]
mod tests {
    use super::{TriangleManSimulation, TriangleManSpec};
    use crate::ecs::{EntityId, Transform, World};
    use crate::input::InputEvent;
    use crate::simulation::{SchedulerFrameReport, SimulationModel};

    fn input_event(source: &str, control: &str, is_pressed: bool) -> InputEvent {
        input_event_with_value(
            source,
            control,
            is_pressed,
            if is_pressed { 1.0 } else { 0.0 },
        )
    }

    fn input_event_with_value(
        source: &str,
        control: &str,
        is_pressed: bool,
        value: f32,
    ) -> InputEvent {
        InputEvent {
            source: source.to_owned(),
            control: control.to_owned(),
            value,
            device_index: None,
            is_pressed,
            is_repeat: false,
        }
    }

    #[test]
    fn same_input_sequence_stays_deterministic_across_sessions() {
        let anchor: EntityId = 42;
        let initial = Transform {
            position_x: -0.9,
            position_y: -0.9,
            rotation_rad: 0.0,
            uniform_scale: 0.1,
            ..Default::default()
        };

        let mut session_a = TriangleManSimulation::new(anchor, TriangleManSpec::default(), initial);
        let mut session_b = TriangleManSimulation::new(anchor, TriangleManSpec::default(), initial);

        let dt = 1.0f32 / 60.0;
        for _step in 0..120 {
            session_a.update(dt);
            session_b.update(dt);
        }

        let a = session_a.transform_2d();
        let b = session_b.transform_2d();

        assert!((a.position_x - b.position_x).abs() < f32::EPSILON);
        assert!((a.position_y - b.position_y).abs() < f32::EPSILON);
        assert!((a.rotation_rad - b.rotation_rad).abs() < f32::EPSILON);
        assert!((a.uniform_scale - b.uniform_scale).abs() < f32::EPSILON);
    }

    #[test]
    fn second_press_after_release_is_processed() {
        let anchor: EntityId = 7;
        let initial = Transform {
            position_x: 0.0,
            position_y: 0.0,
            rotation_rad: 0.0,
            uniform_scale: 0.1,
            ..Default::default()
        };

        let mut with_second_press =
            TriangleManSimulation::new(anchor, TriangleManSpec::default(), initial);
        let mut without_second_press =
            TriangleManSimulation::new(anchor, TriangleManSpec::default(), initial);

        let dt = 1.0f32 / 60.0;

        with_second_press.handle_input_event(input_event("keyboard", "KeyW", true));
        without_second_press.handle_input_event(input_event("keyboard", "KeyW", true));
        with_second_press.update(dt);
        without_second_press.update(dt);

        with_second_press.handle_input_event(input_event("keyboard", "KeyW", false));
        without_second_press.handle_input_event(input_event("keyboard", "KeyW", false));
        with_second_press.update(dt);
        without_second_press.update(dt);

        with_second_press.handle_input_event(input_event("keyboard", "KeyW", true));
        with_second_press.update(dt);
        without_second_press.update(dt);

        let a = with_second_press.transform_2d();
        let b = without_second_press.transform_2d();
        assert!(
            (a.position_x - b.position_x).abs() > 1e-6
                || (a.position_y - b.position_y).abs() > 1e-6,
            "expected second press to change movement, but it was ignored"
        );
    }

    #[test]
    fn mouse_primary_binding_triggers_fire_request() {
        let anchor: EntityId = 11;
        let initial = Transform {
            position_x: 0.0,
            position_y: 0.0,
            rotation_rad: 0.0,
            uniform_scale: 0.1,
            ..Default::default()
        };

        let mut from_mouse =
            TriangleManSimulation::new(anchor, TriangleManSpec::default(), initial);
        let mut no_fire_control =
            TriangleManSimulation::new(anchor, TriangleManSpec::default(), initial);

        let mut world_mouse = World::new();
        let mut world_control = World::new();
        let report = SchedulerFrameReport::default();
        let dt = 1.0f32 / 60.0;

        from_mouse.handle_input_event(input_event("mouse", "primary_button", true));
        from_mouse.update(dt);
        no_fire_control.update(dt);

        from_mouse.reconcile_world(&mut world_mouse, dt, &report);
        no_fire_control.reconcile_world(&mut world_control, dt, &report);

        let mouse_entities = world_mouse.transforms().count();
        let control_entities = world_control.transforms().count();

        assert!(
            mouse_entities > control_entities,
            "expected mouse fire binding to spawn an extra entity via fire action"
        );
    }

    #[test]
    fn right_trigger_analog_value_applies_forward_thrust() {
        let anchor: EntityId = 17;
        let initial = Transform {
            position_x: 0.0,
            position_y: 0.0,
            rotation_rad: 0.0,
            uniform_scale: 0.1,
            ..Default::default()
        };

        let mut with_trigger =
            TriangleManSimulation::new(anchor, TriangleManSpec::default(), initial);
        let mut without_trigger =
            TriangleManSimulation::new(anchor, TriangleManSpec::default(), initial);
        let dt = 1.0f32 / 60.0;

        with_trigger.handle_input_event(InputEvent {
            source: "gamepad".to_owned(),
            control: "right_trigger".to_owned(),
            value: 0.75,
            device_index: Some(0),
            is_pressed: true,
            is_repeat: false,
        });

        with_trigger.update(dt);
        without_trigger.update(dt);

        let with_transform = with_trigger.transform_2d();
        let without_transform = without_trigger.transform_2d();
        assert!(with_transform.position_x > without_transform.position_x);
    }

    #[test]
    fn left_trigger_analog_value_applies_reverse_thrust() {
        let anchor: EntityId = 19;
        let initial = Transform {
            position_x: 0.0,
            position_y: 0.0,
            rotation_rad: 0.0,
            uniform_scale: 0.1,
            ..Default::default()
        };

        let mut with_trigger =
            TriangleManSimulation::new(anchor, TriangleManSpec::default(), initial);
        let mut without_trigger =
            TriangleManSimulation::new(anchor, TriangleManSpec::default(), initial);
        let dt = 1.0f32 / 60.0;

        with_trigger.handle_input_event(InputEvent {
            source: "gamepad".to_owned(),
            control: "left_trigger".to_owned(),
            value: 0.75,
            device_index: Some(0),
            is_pressed: true,
            is_repeat: false,
        });

        with_trigger.update(dt);
        without_trigger.update(dt);

        let with_transform = with_trigger.transform_2d();
        let without_transform = without_trigger.transform_2d();
        assert!(with_transform.position_x < without_transform.position_x);
    }

    #[test]
    fn firefox_right_trigger_axis_value_applies_forward_thrust() {
        let anchor: EntityId = 29;
        let initial = Transform {
            position_x: 0.0,
            position_y: 0.0,
            rotation_rad: 0.0,
            uniform_scale: 0.1,
            ..Default::default()
        };

        let mut with_trigger =
            TriangleManSimulation::new(anchor, TriangleManSpec::default(), initial);
        let mut without_trigger =
            TriangleManSimulation::new(anchor, TriangleManSpec::default(), initial);
        let dt = 1.0f32 / 60.0;

        with_trigger.handle_input_event(input_event_with_value(
            "gamepad",
            "right_trigger",
            false,
            0.0,
        ));
        with_trigger.handle_input_event(input_event_with_value(
            "gamepad",
            "right_trigger",
            true,
            1.0,
        ));

        with_trigger.update(dt);
        without_trigger.update(dt);

        let with_transform = with_trigger.transform_2d();
        let without_transform = without_trigger.transform_2d();
        assert!(with_transform.position_x > without_transform.position_x);
    }

    #[test]
    fn firefox_left_trigger_axis_value_applies_reverse_thrust() {
        let anchor: EntityId = 31;
        let initial = Transform {
            position_x: 0.0,
            position_y: 0.0,
            rotation_rad: 0.0,
            uniform_scale: 0.1,
            ..Default::default()
        };

        let mut with_trigger =
            TriangleManSimulation::new(anchor, TriangleManSpec::default(), initial);
        let mut without_trigger =
            TriangleManSimulation::new(anchor, TriangleManSpec::default(), initial);
        let dt = 1.0f32 / 60.0;

        with_trigger.handle_input_event(input_event_with_value(
            "gamepad",
            "left_trigger",
            false,
            0.0,
        ));
        with_trigger.handle_input_event(input_event_with_value(
            "gamepad",
            "left_trigger",
            true,
            1.0,
        ));

        with_trigger.update(dt);
        without_trigger.update(dt);

        let with_transform = with_trigger.transform_2d();
        let without_transform = without_trigger.transform_2d();
        assert!(with_transform.position_x < without_transform.position_x);
    }

    #[test]
    fn trigger_hysteresis_requires_press_threshold_and_holds_until_release_threshold() {
        let anchor: EntityId = 23;
        let initial = Transform {
            position_x: 0.0,
            position_y: 0.0,
            rotation_rad: 0.0,
            uniform_scale: 0.1,
            ..Default::default()
        };

        let dt = 1.0f32 / 60.0;
        let mut below_press =
            TriangleManSimulation::new(anchor, TriangleManSpec::default(), initial);
        let mut held_between_thresholds =
            TriangleManSimulation::new(anchor, TriangleManSpec::default(), initial);
        let mut control = TriangleManSimulation::new(anchor, TriangleManSpec::default(), initial);

        below_press.handle_input_event(input_event_with_value(
            "gamepad",
            "right_trigger",
            false,
            0.50,
        ));
        below_press.update(dt);

        held_between_thresholds.handle_input_event(input_event_with_value(
            "gamepad",
            "right_trigger",
            true,
            0.60,
        ));
        held_between_thresholds.update(dt);
        held_between_thresholds.handle_input_event(input_event_with_value(
            "gamepad",
            "right_trigger",
            false,
            0.50,
        ));
        held_between_thresholds.update(dt);

        control.update(dt);
        control.update(dt);

        let below_press_transform = below_press.transform_2d();
        let held_transform = held_between_thresholds.transform_2d();
        let control_transform = control.transform_2d();

        assert!(
            (below_press_transform.position_x - control_transform.position_x).abs() < 1e-6,
            "trigger input below press threshold should be ignored"
        );
        assert!(
            held_transform.position_x > control_transform.position_x,
            "trigger should remain active between press and release thresholds"
        );
    }

    #[test]
    fn gamepad_left_stick_y_controls_forward_thrust() {
        let anchor: EntityId = 12;
        let initial = Transform {
            position_x: 0.0,
            position_y: 0.0,
            rotation_rad: 0.0,
            uniform_scale: 0.1,
            ..Default::default()
        };

        let mut with_stick =
            TriangleManSimulation::new(anchor, TriangleManSpec::default(), initial);
        let mut without_stick =
            TriangleManSimulation::new(anchor, TriangleManSpec::default(), initial);

        let dt = 1.0f32 / 60.0;

        with_stick.handle_input_event(input_event_with_value(
            "gamepad",
            "left_stick_y",
            true,
            -1.0,
        ));

        with_stick.update(dt);
        without_stick.update(dt);

        let a = with_stick.transform_2d();
        let b = without_stick.transform_2d();

        assert!(
            (a.position_x - b.position_x).abs() > 1e-6
                || (a.position_y - b.position_y).abs() > 1e-6,
            "expected gamepad axis mapping to affect motion"
        );
    }
}
