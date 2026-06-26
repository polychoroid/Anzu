use std::collections::{BTreeMap, BTreeSet};
use std::f32::consts::TAU;

use winit::keyboard::KeyCode;

use crate::ecs::{
    CollisionBounds, EntityId, Lifecycle, Mesh, MeshAssetId, PolygonCollider, RigidBody, Transform,
    World,
};
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
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
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
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3];

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
    },
    Vertex {
        position: [-0.3, 0.5],
        color: [0.2, 1.0, 0.2],
    },
    Vertex {
        position: [-0.3, -0.5],
        color: [0.2, 0.4, 1.0],
    },
];

pub const SQUARE_VERTICES: [Vertex; 6] = [
    Vertex {
        position: [-0.35, -0.35],
        color: [1.0, 0.8, 0.2],
    },
    Vertex {
        position: [0.35, -0.35],
        color: [1.0, 0.8, 0.2],
    },
    Vertex {
        position: [0.35, 0.35],
        color: [1.0, 0.8, 0.2],
    },
    Vertex {
        position: [-0.35, -0.35],
        color: [1.0, 0.8, 0.2],
    },
    Vertex {
        position: [0.35, 0.35],
        color: [1.0, 0.8, 0.2],
    },
    Vertex {
        position: [-0.35, 0.35],
        color: [1.0, 0.8, 0.2],
    },
];

pub const DIAMOND_VERTICES: [Vertex; 6] = [
    Vertex {
        position: [0.0, 0.45],
        color: [0.8, 0.3, 1.0],
    },
    Vertex {
        position: [0.3, 0.0],
        color: [0.8, 0.3, 1.0],
    },
    Vertex {
        position: [0.0, -0.45],
        color: [0.8, 0.3, 1.0],
    },
    Vertex {
        position: [0.0, 0.45],
        color: [0.8, 0.3, 1.0],
    },
    Vertex {
        position: [0.0, -0.45],
        color: [0.8, 0.3, 1.0],
    },
    Vertex {
        position: [-0.3, 0.0],
        color: [0.8, 0.3, 1.0],
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
    (0.88, 0.75, -0.26, -0.04, 0.55),
    (0.90, -0.30, -0.23, 0.10, -0.62),
    (-0.90, 0.50, 0.24, -0.06, 0.42),
    (-0.92, -0.78, 0.27, 0.06, -0.71),
    (0.30, 0.92, -0.04, -0.24, 0.48),
    (-0.35, 0.90, 0.10, -0.23, -0.53),
    (0.52, -0.90, -0.08, 0.25, 0.67),
    (-0.62, -0.92, 0.11, 0.27, -0.58),
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
    pub scale: f32,
}

impl Default for TriangleManSpec {
    fn default() -> Self {
        Self {
            thrust_accel_units_per_sec2: 0.15,
            reverse_thrust_scale: 0.15,
            angular_accel_rad_per_sec2: 9.0,
            linear_damping_per_sec: 0.08,
            angular_damping_per_sec: 0.22,
            fire_cooldown_seconds: 0.12,
            bullet_speed_units_per_sec: 0.9,
            bullet_lifetime_seconds: 2.0,
            asteroid_spawn_interval_seconds: 3.0,
            asteroid_spawn_min_interval_seconds: 0.45,
            asteroid_spawn_accel_per_second: 0.99,
            scale: 0.1,
        }
    }
}

#[derive(Default)]
struct InputManager {
    thrust_forward: bool,
    thrust_reverse: bool,
    turn_left: bool,
    turn_right: bool,
    fire_key_down: bool,
    pending_fire: bool,
}

#[derive(Clone, Copy, Default)]
struct InputFrame {
    thrust: i8,
    turn: i8,
    fire: bool,
}

impl InputManager {
    fn handle_key_event(&mut self, key_code: KeyCode, is_pressed: bool, is_repeat: bool) {
        if is_repeat && is_pressed {
            return;
        }

        match key_code {
            KeyCode::KeyW => self.thrust_forward = is_pressed,
            KeyCode::KeyS => self.thrust_reverse = is_pressed,
            KeyCode::KeyA => self.turn_left = is_pressed,
            KeyCode::KeyD => self.turn_right = is_pressed,
            KeyCode::Space => {
                if is_pressed && !self.fire_key_down {
                    self.pending_fire = true;
                }
                self.fire_key_down = is_pressed;
            }
            _ => {}
        }
    }

    fn snapshot_frame(&mut self) -> InputFrame {
        let frame = InputFrame {
            thrust: axis_value(self.thrust_reverse, self.thrust_forward),
            turn: axis_value(self.turn_left, self.turn_right),
            fire: self.pending_fire,
        };

        self.pending_fire = false;
        frame
    }
}

fn axis_value(negative: bool, positive: bool) -> i8 {
    match (negative, positive) {
        (true, false) => -1,
        (false, true) => 1,
        _ => 0,
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
                vertex_count: TRIANGLE_VERTICES.len() as u32,
                index_bytes: &[],
                index_count: 0,
            },
        );
        world.register_mesh_asset(
            MESH_SQUARE,
            Mesh {
                vertex_bytes: bytemuck::cast_slice(&SQUARE_VERTICES),
                vertex_count: SQUARE_VERTICES.len() as u32,
                index_bytes: &[],
                index_count: 0,
            },
        );
        world.register_mesh_asset(
            MESH_DIAMOND,
            Mesh {
                vertex_bytes: bytemuck::cast_slice(&DIAMOND_VERTICES),
                vertex_count: DIAMOND_VERTICES.len() as u32,
                index_bytes: &[],
                index_count: 0,
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
    input_manager: InputManager,
    fire_cooldown_seconds: f32,
    elapsed_seconds: f32,
    asteroid_spawn_timer_seconds: f32,
    asteroid_spawn_index: usize,
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
            input_manager: InputManager::default(),
            fire_cooldown_seconds: 0.0,
            elapsed_seconds: 0.0,
            asteroid_spawn_timer_seconds: spec.asteroid_spawn_interval_seconds,
            asteroid_spawn_index: 1,
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
    ) -> EntityId {
        let preset = ASTEROID_SPAWN_PRESETS[spawn_index % ASTEROID_SPAWN_PRESETS.len()];
        let entity_id = world.spawn();
        let radius = max_radius(&SQUARE_COLLIDER, scale);
        let mass = 2.2;
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
                z_depth: 0.0,
            },
        );
        world.set_mesh_instance(entity_id, MESH_SQUARE);
        world.set_collision_bounds(
            entity_id,
            CollisionBounds {
                radius,
                proximity_radius: radius * 1.35,
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
                z_depth: 0.0,
            },
        );
        world.set_mesh_instance(entity_id, MESH_DIAMOND);
        world.set_collision_bounds(
            entity_id,
            CollisionBounds {
                radius,
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
}

impl SimulationModel for TriangleManSimulation {
    fn handle_key_event(&mut self, key_code: KeyCode, is_pressed: bool, is_repeat: bool) {
        self.input_manager
            .handle_key_event(key_code, is_pressed, is_repeat);
    }

    fn update(&mut self, delta_seconds: f32) {
        if delta_seconds <= 0.0 {
            return;
        }

        self.elapsed_seconds += delta_seconds;

        let frame = self.input_manager.snapshot_frame();
        let thrust_accel = if frame.thrust > 0 {
            self.spec.thrust_accel_units_per_sec2
        } else if frame.thrust < 0 {
            -self.spec.thrust_accel_units_per_sec2 * self.spec.reverse_thrust_scale
        } else {
            0.0
        };

        let heading_x = self.angle_rad.cos();
        let heading_y = self.angle_rad.sin();
        self.velocity_x += heading_x * thrust_accel * delta_seconds;
        self.velocity_y += heading_y * thrust_accel * delta_seconds;

        let angular_accel = -(frame.turn as f32) * self.spec.angular_accel_rad_per_sec2;
        self.angular_velocity += angular_accel * delta_seconds;

        let linear_drag = (1.0 - self.spec.linear_damping_per_sec * delta_seconds).clamp(0.0, 1.0);
        self.velocity_x *= linear_drag;
        self.velocity_y *= linear_drag;
        let angular_drag =
            (1.0 - self.spec.angular_damping_per_sec * delta_seconds).clamp(0.0, 1.0);
        self.angular_velocity *= angular_drag;

        self.position_x += self.velocity_x * delta_seconds;
        self.position_y += self.velocity_y * delta_seconds;
        self.angle_rad = (self.angle_rad + self.angular_velocity * delta_seconds).rem_euclid(TAU);

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
            self.spawn_asteroid_entity(world, 0, 0.28);
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

        let dynamic_interval = (self.spec.asteroid_spawn_interval_seconds
            - self.elapsed_seconds * self.spec.asteroid_spawn_accel_per_second)
            .max(self.spec.asteroid_spawn_min_interval_seconds);

        while self.asteroid_spawn_timer_seconds <= 0.0 {
            self.spawn_asteroid_entity(world, self.asteroid_spawn_index, 0.24);
            self.asteroid_spawn_index = self.asteroid_spawn_index.saturating_add(1);
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
            z_depth: 0.0,
        },
    );
    world.set_mesh_instance(entity_id, MESH_TRIANGLE);
    world.set_collision_bounds(
        entity_id,
        CollisionBounds {
            radius,
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
    use crate::ecs::{EntityId, Transform};
    use crate::simulation::SimulationModel;

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
}
