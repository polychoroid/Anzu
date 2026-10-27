use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::f32::consts::PI;

use crate::ecs::{
    CollisionBounds, EntityId, Lifecycle, MaterialId, Mesh, MeshAssetId, PolygonCollider,
    RigidBody, Transform, World,
};
use crate::input::{InputBinding, InputContextStack, InputControlMap, InputEvent};
use crate::platformer_common::{
    PLATFORMER_GROUND_VERTICES, PLATFORMER_PLAYER_VERTICES, PLATFORMER_SHADER,
    PLATFORMER_SHADER_WEBGL, PlatformerLocomotionState, PlatformerTuning, PlatformerVertex,
    platformer_ground_contact_y, register_platformer_assets, spawn_platformer_ground,
    spawn_platformer_player,
};
use crate::renderer::{MaterialBlendMode, MaterialDefinition, RenderRomPackage, RomRenderData};
use crate::rom::RomPackage;
use crate::simulation::{
    InputContextRequest, SchedulerFrameReport, SimulationModel, SimulationTransform2D,
};

const MESH_AIM_INDICATOR: MeshAssetId = MeshAssetId(13);
const MATERIAL_AIM_INDICATOR: MaterialId = MaterialId(13);
const MATERIAL_PLATFORM: MaterialId = MaterialId(14);
const MATERIAL_ASTEROID: MaterialId = MaterialId(15);
const MATERIAL_BULLET: MaterialId = MaterialId(16);
const MATERIAL_GOAL: MaterialId = MaterialId(17);
const AIM_INDICATOR_SCALE: f32 = 0.09;
const AIM_INDICATOR_OFFSET: f32 = 0.11;
const AIM_TURN_RATE_RAD_PER_SEC: f32 = PI * 8.0;
const AIM_INPUT_DEADZONE: f32 = 0.25;
const ASTEROID_GRAVITY_UNITS_PER_SEC2: f32 = 1.15;
const ASTEROID_BULLET_SPEED_UNITS_PER_SEC: f32 = 0.55;
const ASTEROID_SPAWN_INTERVAL_SECONDS: f32 = 1.35;
const ASTEROID_SPAWN_MIN_INTERVAL_SECONDS: f32 = 0.7;
const ASTEROID_MIN_FRAGMENT_SCALE: f32 = 0.05;
const PLAYER_BASE_FIRE_COOLDOWN_SECONDS: f32 = 0.16;

const MESH_PLATFORM: MeshAssetId = MeshAssetId(14);
const MESH_ASTEROID: MeshAssetId = MeshAssetId(15);
const MESH_BULLET: MeshAssetId = MeshAssetId(16);
const MESH_GOAL: MeshAssetId = MeshAssetId(17);

const WORLD_MIN_FALL_Y: f32 = -1.08;
const PLATFORM_HALF_THICKNESS: f32 = 0.045;
const PLATFORM_HALF_WIDTH_UNIT: f32 = 0.5;
const PLAYER_BOTTOM_OFFSET: f32 = 0.42;
const PLAYER_COLLISION_HALF_WIDTH_UNIT: f32 = 0.36;
const PLAYER_COLLISION_TOP_OFFSET_UNIT: f32 = 0.56;
const PLAYER_PLATFORM_SNAP_EPSILON: f32 = 0.015;
const BULLET_TTL_SECONDS: f32 = 1.0;
const BULLET_SCALE: f32 = 0.045;
const ASTEROID_BASE_SCALE: f32 = 0.13;
const ASTEROID_FRAGMENT_SCALE_FACTOR: f32 = 0.62;
const ASTEROID_FRAGMENT_SPEED_UNIT: f32 = 0.17;
const ASTEROID_BASE_HEALTH: f32 = 2.0;
const ASTEROID_FRAGMENT_HEALTH: f32 = 1.0;

const PLAYER_COLLIDER_VERTICES: [[f32; 2]; 4] = [
    [-PLAYER_COLLISION_HALF_WIDTH_UNIT, -PLAYER_BOTTOM_OFFSET],
    [PLAYER_COLLISION_HALF_WIDTH_UNIT, -PLAYER_BOTTOM_OFFSET],
    [
        PLAYER_COLLISION_HALF_WIDTH_UNIT,
        PLAYER_COLLISION_TOP_OFFSET_UNIT,
    ],
    [
        -PLAYER_COLLISION_HALF_WIDTH_UNIT,
        PLAYER_COLLISION_TOP_OFFSET_UNIT,
    ],
];
const PLATFORM_COLLIDER_VERTICES: [[f32; 2]; 4] = [
    [-PLATFORM_HALF_WIDTH_UNIT, -PLATFORM_HALF_THICKNESS],
    [PLATFORM_HALF_WIDTH_UNIT, -PLATFORM_HALF_THICKNESS],
    [PLATFORM_HALF_WIDTH_UNIT, PLATFORM_HALF_THICKNESS],
    [-PLATFORM_HALF_WIDTH_UNIT, PLATFORM_HALF_THICKNESS],
];
const GOAL_COLLIDER_VERTICES: [[f32; 2]; 4] =
    [[-0.10, -0.30], [0.10, -0.30], [0.10, 0.30], [-0.10, 0.30]];
const ASTEROID_COLLIDER_VERTICES: [[f32; 2]; 4] =
    [[0.0, 0.40], [0.30, 0.0], [0.0, -0.40], [-0.30, 0.0]];
const BULLET_COLLIDER_VERTICES: [[f32; 2]; 4] =
    [[0.0, 0.14], [0.10, 0.0], [0.0, -0.14], [-0.10, 0.0]];

const TRIANGLE_MAN_2_GAMEPLAY_CONTEXT: &str = "triangle_man_2.gameplay";
const TRIANGLE_MAN_2_MENU_CONTEXT: &str = "triangle_man_2.menu";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RoleKind {
    Player,
    Platform,
    Goal,
    Asteroid,
    Bullet,
    AimIndicator,
}

#[derive(Clone, Copy)]
struct PlatformSpec {
    center_x: f32,
    center_y: f32,
    half_width_scale: f32,
}

const TRIANGLE_MAN_2_LEVEL_PLATFORMS: [PlatformSpec; 5] = [
    PlatformSpec {
        center_x: -0.58,
        center_y: -0.42,
        half_width_scale: 0.92,
    },
    PlatformSpec {
        center_x: 0.44,
        center_y: -0.18,
        half_width_scale: 0.76,
    },
    PlatformSpec {
        center_x: -0.16,
        center_y: 0.06,
        half_width_scale: 0.92,
    },
    PlatformSpec {
        center_x: 0.50,
        center_y: 0.30,
        half_width_scale: 0.72,
    },
    PlatformSpec {
        center_x: -0.20,
        center_y: 0.54,
        half_width_scale: 0.80,
    },
];

const TRIANGLE_MAN_2_GOAL_POSITION: (f32, f32, f32) = (0.68, 0.78, 0.18);
const TRIANGLE_MAN_2_START_X: f32 = -0.58;
const TRIANGLE_MAN_2_INITIAL_ASTEROIDS: [(f32, f32, f32, f32, f32); 3] = [
    (0.62, 0.92, -0.18, -0.36, 1.0),
    (-0.68, 1.00, 0.15, -0.42, -1.2),
    (0.24, 1.12, -0.05, -0.34, 0.8),
];

pub const AIM_INDICATOR_VERTICES: [PlatformerVertex; 6] = [
    PlatformerVertex {
        position: [-0.22, -0.03],
        color: [1.0, 0.96, 0.34],
        barycentric: [1.0, 0.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [0.16, -0.03],
        color: [1.0, 0.96, 0.34],
        barycentric: [0.0, 1.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [0.16, 0.03],
        color: [1.0, 0.96, 0.34],
        barycentric: [0.0, 0.0, 1.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [-0.22, -0.03],
        color: [1.0, 0.96, 0.34],
        barycentric: [1.0, 0.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [0.16, 0.03],
        color: [1.0, 0.96, 0.34],
        barycentric: [0.0, 1.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [0.30, 0.0],
        color: [1.0, 0.96, 0.34],
        barycentric: [0.0, 0.0, 1.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
];

pub const PLATFORM_VERTICES: [PlatformerVertex; 6] = [
    PlatformerVertex {
        position: [-0.50, -0.045],
        color: [0.20, 0.28, 0.42],
        barycentric: [1.0, 0.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [0.50, -0.045],
        color: [0.22, 0.32, 0.46],
        barycentric: [0.0, 1.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [0.50, 0.045],
        color: [0.24, 0.36, 0.52],
        barycentric: [0.0, 0.0, 1.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [-0.50, -0.045],
        color: [0.20, 0.28, 0.42],
        barycentric: [1.0, 0.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [0.50, 0.045],
        color: [0.24, 0.36, 0.52],
        barycentric: [0.0, 1.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [-0.50, 0.045],
        color: [0.22, 0.32, 0.46],
        barycentric: [0.0, 0.0, 1.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
];

pub const ASTEROID_VERTICES: [PlatformerVertex; 6] = [
    PlatformerVertex {
        position: [0.0, 0.40],
        color: [0.68, 0.64, 0.48],
        barycentric: [1.0, 0.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 0.0, 1.0],
    },
    PlatformerVertex {
        position: [0.30, 0.0],
        color: [0.58, 0.56, 0.40],
        barycentric: [0.0, 1.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 0.0, 1.0],
    },
    PlatformerVertex {
        position: [0.0, -0.40],
        color: [0.48, 0.46, 0.34],
        barycentric: [0.0, 0.0, 1.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 0.0, 1.0],
    },
    PlatformerVertex {
        position: [0.0, 0.40],
        color: [0.68, 0.64, 0.48],
        barycentric: [1.0, 0.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 0.0, 1.0],
    },
    PlatformerVertex {
        position: [0.0, -0.40],
        color: [0.48, 0.46, 0.34],
        barycentric: [0.0, 0.0, 1.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 0.0, 1.0],
    },
    PlatformerVertex {
        position: [-0.30, 0.0],
        color: [0.58, 0.56, 0.40],
        barycentric: [0.0, 1.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 0.0, 1.0],
    },
];

pub const BULLET_VERTICES: [PlatformerVertex; 6] = [
    PlatformerVertex {
        position: [0.0, 0.14],
        color: [1.0, 0.96, 0.58],
        barycentric: [1.0, 0.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [0.10, 0.0],
        color: [1.0, 0.90, 0.42],
        barycentric: [0.0, 1.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [0.0, -0.14],
        color: [1.0, 0.82, 0.34],
        barycentric: [0.0, 0.0, 1.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [0.0, 0.14],
        color: [1.0, 0.96, 0.58],
        barycentric: [1.0, 0.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [0.0, -0.14],
        color: [1.0, 0.82, 0.34],
        barycentric: [0.0, 0.0, 1.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [-0.10, 0.0],
        color: [1.0, 0.90, 0.42],
        barycentric: [0.0, 1.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
];

pub const GOAL_VERTICES: [PlatformerVertex; 6] = [
    PlatformerVertex {
        position: [-0.08, -0.28],
        color: [0.95, 0.85, 0.32],
        barycentric: [1.0, 0.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [0.08, -0.28],
        color: [0.95, 0.85, 0.32],
        barycentric: [0.0, 1.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [0.08, 0.30],
        color: [1.0, 0.96, 0.48],
        barycentric: [0.0, 0.0, 1.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [-0.08, -0.28],
        color: [0.95, 0.85, 0.32],
        barycentric: [1.0, 0.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [0.08, 0.30],
        color: [1.0, 0.96, 0.48],
        barycentric: [0.0, 1.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [-0.08, 0.30],
        color: [1.0, 0.92, 0.36],
        barycentric: [0.0, 0.0, 1.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
];

const TRIANGLE_MAN_2_MATERIALS: [MaterialDefinition; 7] = [
    MaterialDefinition {
        material_id: crate::platformer_common::PLATFORMER_GROUND_MATERIAL,
        blend_mode: MaterialBlendMode::Alpha,
        base_color_tint: [1.0, 1.0, 1.0],
        emissive_strength: 0.0,
        metallic: 0.0,
        roughness: 0.8,
        specular_strength: 0.1,
    },
    MaterialDefinition {
        material_id: crate::platformer_common::PLATFORMER_PLAYER_MATERIAL,
        blend_mode: MaterialBlendMode::Alpha,
        base_color_tint: [1.0, 1.0, 1.0],
        emissive_strength: 0.0,
        metallic: 0.0,
        roughness: 0.7,
        specular_strength: 0.1,
    },
    MaterialDefinition {
        material_id: MATERIAL_AIM_INDICATOR,
        blend_mode: MaterialBlendMode::Additive,
        base_color_tint: [1.0, 0.98, 0.55],
        emissive_strength: 0.95,
        metallic: 0.0,
        roughness: 0.2,
        specular_strength: 0.1,
    },
    MaterialDefinition {
        material_id: MATERIAL_PLATFORM,
        blend_mode: MaterialBlendMode::Alpha,
        base_color_tint: [0.78, 0.82, 0.92],
        emissive_strength: 0.03,
        metallic: 0.0,
        roughness: 0.78,
        specular_strength: 0.06,
    },
    MaterialDefinition {
        material_id: MATERIAL_ASTEROID,
        blend_mode: MaterialBlendMode::Alpha,
        base_color_tint: [0.68, 0.64, 0.52],
        emissive_strength: 0.07,
        metallic: 0.25,
        roughness: 0.88,
        specular_strength: 0.08,
    },
    MaterialDefinition {
        material_id: MATERIAL_BULLET,
        blend_mode: MaterialBlendMode::Additive,
        base_color_tint: [1.0, 0.92, 0.42],
        emissive_strength: 0.9,
        metallic: 0.0,
        roughness: 0.15,
        specular_strength: 0.3,
    },
    MaterialDefinition {
        material_id: MATERIAL_GOAL,
        blend_mode: MaterialBlendMode::Additive,
        base_color_tint: [1.0, 0.95, 0.28],
        emissive_strength: 0.85,
        metallic: 0.0,
        roughness: 0.2,
        specular_strength: 0.15,
    },
];

#[derive(Default)]
pub struct TriangleMan2Rom;

impl RomPackage for TriangleMan2Rom {
    fn rom_id(&self) -> &'static str {
        "anzu.triangle_man_2"
    }

    fn bootstrap_world(&self, world: &mut World) -> EntityId {
        register_triangle_man_2_assets(world);
        spawn_platformer_ground(world);

        let start_scale = PlatformerTuning::default().standing_scale;
        let start_position_x = -0.35;
        let start_position_y = platformer_ground_contact_y(start_scale);
        spawn_platformer_player(world, start_position_x, start_position_y, start_scale)
    }

    fn create_simulation(
        &self,
        world: &World,
        anchor_entity: EntityId,
    ) -> Box<dyn SimulationModel> {
        let initial = world.transform(anchor_entity).copied().unwrap_or_default();
        Box::new(TriangleMan2Simulation::new(anchor_entity, initial))
    }
}

impl RenderRomPackage for TriangleMan2Rom {
    fn render_data(&self, webgl_compat: bool) -> RomRenderData {
        triangle_man_2_render_data(webgl_compat)
    }
}

pub struct TriangleMan2Simulation {
    anchor_entity: EntityId,
    locomotion: PlatformerLocomotionState,
    input: TriangleMan2InputManager,
    aim_angle_rad: f32,
    fire_requested: bool,
    aim_indicator_entity: Option<EntityId>,
    roles: BTreeMap<EntityId, RoleKind>,
    asteroid_health_points: BTreeMap<EntityId, f32>,
    level_initialized: bool,
    fire_cooldown_seconds: f32,
    asteroid_spawn_timer_seconds: f32,
    asteroid_spawned_total: u32,
    game_finished: bool,
    pending_context_requests: VecDeque<InputContextRequest>,
}

impl TriangleMan2Simulation {
    pub fn new(anchor_entity: EntityId, initial: Transform) -> Self {
        Self {
            anchor_entity,
            locomotion: PlatformerLocomotionState::new(initial, PlatformerTuning::default()),
            input: TriangleMan2InputManager::default(),
            aim_angle_rad: 0.0,
            fire_requested: false,
            aim_indicator_entity: None,
            roles: BTreeMap::new(),
            asteroid_health_points: BTreeMap::new(),
            level_initialized: false,
            fire_cooldown_seconds: 0.0,
            asteroid_spawn_timer_seconds: ASTEROID_SPAWN_INTERVAL_SECONDS,
            asteroid_spawned_total: 0,
            game_finished: false,
            pending_context_requests: VecDeque::new(),
        }
    }

    fn role_for_entity(&self, entity_id: EntityId) -> Option<RoleKind> {
        self.roles.get(&entity_id).copied()
    }

    fn initialize_level_entities(&mut self, world: &mut World) {
        if self.level_initialized {
            return;
        }

        let standing_scale = PlatformerTuning::default().standing_scale;
        let start_x = TRIANGLE_MAN_2_START_X;
        let start_y = platformer_ground_contact_y(standing_scale);
        self.locomotion.position_x = start_x;
        self.locomotion.position_y = start_y;
        self.locomotion.uniform_scale = standing_scale;
        self.locomotion.velocity_x = 0.0;
        self.locomotion.velocity_y = 0.0;
        self.locomotion.on_ground = false;

        world.set_transform(
            self.anchor_entity,
            Transform {
                position_x: start_x,
                position_y: start_y,
                rotation_rad: 0.0,
                uniform_scale: standing_scale,
                velocity_x: 0.0,
                velocity_y: 0.0,
                angular_velocity: 0.0,
            },
        );

        world.set_collision_bounds(
            self.anchor_entity,
            CollisionBounds {
                proximity_radius: 0.12,
            },
        );
        world.set_polygon_collider(
            self.anchor_entity,
            PolygonCollider {
                local_vertices: &PLAYER_COLLIDER_VERTICES,
            },
        );
        world.set_rigid_body(
            self.anchor_entity,
            RigidBody {
                mass: 1.0,
                inverse_mass: 0.0,
                restitution: 0.0,
                friction: 0.0,
                moment_of_inertia: 1.0,
                inverse_moment_of_inertia: 0.0,
                is_sensor: true,
            },
        );
        self.roles.insert(self.anchor_entity, RoleKind::Player);

        for spec in TRIANGLE_MAN_2_LEVEL_PLATFORMS {
            let platform_entity = world.spawn();
            world.set_transform(
                platform_entity,
                Transform {
                    position_x: spec.center_x,
                    position_y: spec.center_y,
                    rotation_rad: 0.0,
                    uniform_scale: spec.half_width_scale,
                    velocity_x: 0.0,
                    velocity_y: 0.0,
                    angular_velocity: 0.0,
                },
            );
            world.set_mesh_instance_with_material(
                platform_entity,
                MESH_PLATFORM,
                MATERIAL_PLATFORM,
            );
            world.set_collision_bounds(
                platform_entity,
                CollisionBounds {
                    proximity_radius: spec.half_width_scale,
                },
            );
            world.set_polygon_collider(
                platform_entity,
                PolygonCollider {
                    local_vertices: &PLATFORM_COLLIDER_VERTICES,
                },
            );
            world.set_rigid_body(
                platform_entity,
                RigidBody {
                    mass: 1.0,
                    inverse_mass: 0.0,
                    restitution: 0.05,
                    friction: 0.9,
                    moment_of_inertia: 1.0,
                    inverse_moment_of_inertia: 0.0,
                    is_sensor: false,
                },
            );
            self.roles.insert(platform_entity, RoleKind::Platform);
        }

        let goal_entity = world.spawn();
        world.set_transform(
            goal_entity,
            Transform {
                position_x: TRIANGLE_MAN_2_GOAL_POSITION.0,
                position_y: TRIANGLE_MAN_2_GOAL_POSITION.1,
                rotation_rad: 0.0,
                uniform_scale: TRIANGLE_MAN_2_GOAL_POSITION.2,
                velocity_x: 0.0,
                velocity_y: 0.0,
                angular_velocity: 0.0,
            },
        );
        world.set_mesh_instance_with_material(goal_entity, MESH_GOAL, MATERIAL_GOAL);
        world.set_collision_bounds(
            goal_entity,
            CollisionBounds {
                proximity_radius: TRIANGLE_MAN_2_GOAL_POSITION.2 * 0.9,
            },
        );
        world.set_polygon_collider(
            goal_entity,
            PolygonCollider {
                local_vertices: &GOAL_COLLIDER_VERTICES,
            },
        );
        world.set_rigid_body(
            goal_entity,
            RigidBody {
                mass: 1.0,
                inverse_mass: 0.0,
                restitution: 0.0,
                friction: 0.0,
                moment_of_inertia: 1.0,
                inverse_moment_of_inertia: 0.0,
                is_sensor: true,
            },
        );
        self.roles.insert(goal_entity, RoleKind::Goal);

        for seed in TRIANGLE_MAN_2_INITIAL_ASTEROIDS {
            self.spawn_asteroid_entity(world, seed.0, seed.1, seed.2, seed.3, ASTEROID_BASE_SCALE);
        }

        self.level_initialized = true;
    }

    fn spawn_asteroid_entity(
        &mut self,
        world: &mut World,
        position_x: f32,
        position_y: f32,
        velocity_x: f32,
        velocity_y: f32,
        scale: f32,
    ) -> EntityId {
        let entity_id = world.spawn();
        world.set_transform(
            entity_id,
            Transform {
                position_x,
                position_y,
                rotation_rad: 0.0,
                uniform_scale: scale,
                velocity_x,
                velocity_y,
                angular_velocity: 0.0,
            },
        );
        world.set_mesh_instance_with_material(entity_id, MESH_ASTEROID, MATERIAL_ASTEROID);
        world.set_collision_bounds(
            entity_id,
            CollisionBounds {
                proximity_radius: scale * 0.55,
            },
        );
        world.set_polygon_collider(
            entity_id,
            PolygonCollider {
                local_vertices: &ASTEROID_COLLIDER_VERTICES,
            },
        );
        world.set_rigid_body(
            entity_id,
            RigidBody {
                mass: scale.max(ASTEROID_MIN_FRAGMENT_SCALE) * 4.0,
                inverse_mass: 1.0 / (scale.max(ASTEROID_MIN_FRAGMENT_SCALE) * 4.0),
                restitution: 0.12,
                friction: 0.62,
                moment_of_inertia: 1.0,
                inverse_moment_of_inertia: 1.0,
                is_sensor: false,
            },
        );

        self.roles.insert(entity_id, RoleKind::Asteroid);
        self.asteroid_health_points.insert(
            entity_id,
            if scale >= ASTEROID_BASE_SCALE {
                ASTEROID_BASE_HEALTH
            } else {
                ASTEROID_FRAGMENT_HEALTH
            },
        );
        entity_id
    }

    fn spawn_asteroid_wave_entry(&mut self, world: &mut World) {
        let lane = self.asteroid_spawned_total as usize % 5;
        let lane_positions = [-0.78, -0.38, 0.0, 0.38, 0.78];
        let drift = ((self.asteroid_spawned_total as f32) * 1.7).sin() * 0.17;
        let speed_bonus = ((self.asteroid_spawned_total as f32) * 0.37).sin().abs() * 0.12;
        self.spawn_asteroid_entity(
            world,
            lane_positions[lane] + drift,
            1.08,
            drift * 0.18,
            -0.38 - speed_bonus,
            ASTEROID_BASE_SCALE,
        );
        self.asteroid_spawned_total = self.asteroid_spawned_total.saturating_add(1);
    }

    fn spawn_bullet_entity(&mut self, world: &mut World) {
        let Some(player_transform) = world.transform(self.anchor_entity).copied() else {
            return;
        };

        let heading_x = self.aim_angle_rad.cos();
        let heading_y = self.aim_angle_rad.sin();
        let entity_id = world.spawn();
        world.set_transform(
            entity_id,
            Transform {
                position_x: player_transform.position_x + heading_x * 0.12,
                position_y: player_transform.position_y + heading_y * 0.12,
                rotation_rad: self.aim_angle_rad,
                uniform_scale: BULLET_SCALE,
                velocity_x: heading_x * ASTEROID_BULLET_SPEED_UNITS_PER_SEC,
                velocity_y: heading_y * ASTEROID_BULLET_SPEED_UNITS_PER_SEC,
                angular_velocity: 0.0,
            },
        );
        world.set_mesh_instance_with_material(entity_id, MESH_BULLET, MATERIAL_BULLET);
        world.set_collision_bounds(
            entity_id,
            CollisionBounds {
                proximity_radius: 0.05,
            },
        );
        world.set_polygon_collider(
            entity_id,
            PolygonCollider {
                local_vertices: &BULLET_COLLIDER_VERTICES,
            },
        );
        world.set_rigid_body(
            entity_id,
            RigidBody {
                mass: 1.0,
                inverse_mass: 0.0,
                restitution: 0.0,
                friction: 0.0,
                moment_of_inertia: 1.0,
                inverse_moment_of_inertia: 0.0,
                is_sensor: true,
            },
        );
        world.set_lifecycle(
            entity_id,
            Lifecycle {
                ttl_seconds: BULLET_TTL_SECONDS,
            },
        );
        self.roles.insert(entity_id, RoleKind::Bullet);
    }

    fn reconcile_player_platform_contacts(&mut self, world: &mut World) {
        let Some(player_transform) = world.transform(self.anchor_entity).copied() else {
            return;
        };

        let mut landing_y = None;
        for (entity_id, role) in &self.roles {
            if *role != RoleKind::Platform {
                continue;
            }
            let Some(platform_transform) = world.transform(*entity_id).copied() else {
                continue;
            };
            let half_width = PLATFORM_HALF_WIDTH_UNIT * platform_transform.uniform_scale;
            let left_x = platform_transform.position_x - half_width;
            let right_x = platform_transform.position_x + half_width;
            if player_transform.position_x < left_x || player_transform.position_x > right_x {
                continue;
            }

            let top_y = platform_transform.position_y + PLATFORM_HALF_THICKNESS;
            let player_bottom_y =
                player_transform.position_y - PLAYER_BOTTOM_OFFSET * self.locomotion.uniform_scale;
            let target_contact_y = top_y + PLAYER_BOTTOM_OFFSET * self.locomotion.uniform_scale;
            if player_bottom_y >= top_y - 0.08
                && player_bottom_y <= top_y + PLAYER_PLATFORM_SNAP_EPSILON
                && self.locomotion.velocity_y <= 0.0
            {
                landing_y = Some(target_contact_y.max(landing_y.unwrap_or(f32::MIN)));
            }
        }

        if let Some(contact_y) = landing_y {
            self.locomotion.position_y = contact_y;
            self.locomotion.velocity_y = 0.0;
            self.locomotion.on_ground = true;
            if let Some(transform) = world.transform_mut(self.anchor_entity) {
                transform.position_y = contact_y;
                transform.velocity_y = 0.0;
            }
        }
    }

    fn spawn_fragments_from_asteroid(
        &mut self,
        world: &mut World,
        entity_id: EntityId,
        impact_direction_x: f32,
        impact_direction_y: f32,
    ) {
        let Some(parent) = world.transform(entity_id).copied() else {
            return;
        };

        let child_scale = parent.uniform_scale * ASTEROID_FRAGMENT_SCALE_FACTOR;
        if child_scale < ASTEROID_MIN_FRAGMENT_SCALE {
            return;
        }

        let left_dir_x = -impact_direction_y;
        let left_dir_y = impact_direction_x;
        let right_dir_x = impact_direction_y;
        let right_dir_y = -impact_direction_x;

        self.spawn_asteroid_entity(
            world,
            parent.position_x + left_dir_x * child_scale,
            parent.position_y + left_dir_y * child_scale,
            parent.velocity_x + left_dir_x * ASTEROID_FRAGMENT_SPEED_UNIT,
            parent.velocity_y + left_dir_y * ASTEROID_FRAGMENT_SPEED_UNIT,
            child_scale,
        );
        self.spawn_asteroid_entity(
            world,
            parent.position_x + right_dir_x * child_scale,
            parent.position_y + right_dir_y * child_scale,
            parent.velocity_x + right_dir_x * ASTEROID_FRAGMENT_SPEED_UNIT,
            parent.velocity_y + right_dir_y * ASTEROID_FRAGMENT_SPEED_UNIT,
            child_scale,
        );
    }

    fn handle_collision_events(&mut self, world: &mut World, frame_report: &SchedulerFrameReport) {
        let mut despawn_set = BTreeSet::new();

        for event in &frame_report.interaction_events {
            if event.kind != crate::simulation::InteractionEventKind::Collision {
                continue;
            }

            let Some(role_a) = self.role_for_entity(event.entity_a) else {
                continue;
            };
            let Some(role_b) = self.role_for_entity(event.entity_b) else {
                continue;
            };

            let pair = (role_a, role_b);
            if matches!(
                pair,
                (RoleKind::Player, RoleKind::Goal) | (RoleKind::Goal, RoleKind::Player)
            ) {
                self.game_finished = true;
                continue;
            }

            if matches!(
                pair,
                (RoleKind::Player, RoleKind::Asteroid) | (RoleKind::Asteroid, RoleKind::Player)
            ) {
                self.game_finished = true;
                continue;
            }

            if matches!(
                pair,
                (RoleKind::Bullet, RoleKind::Asteroid) | (RoleKind::Asteroid, RoleKind::Bullet)
            ) {
                let bullet_entity = if role_a == RoleKind::Bullet {
                    event.entity_a
                } else {
                    event.entity_b
                };
                let asteroid_entity = if role_a == RoleKind::Asteroid {
                    event.entity_a
                } else {
                    event.entity_b
                };
                despawn_set.insert(bullet_entity);

                let health = self
                    .asteroid_health_points
                    .entry(asteroid_entity)
                    .or_insert(ASTEROID_BASE_HEALTH);
                *health = (*health - 1.0).max(0.0);

                if *health <= 0.0 {
                    let mut direction_x = self.aim_angle_rad.cos();
                    let mut direction_y = self.aim_angle_rad.sin();
                    if let (Some(asteroid), Some(bullet)) = (
                        world.transform(asteroid_entity).copied(),
                        world.transform(bullet_entity).copied(),
                    ) {
                        let dx = asteroid.position_x - bullet.position_x;
                        let dy = asteroid.position_y - bullet.position_y;
                        let len = (dx * dx + dy * dy).sqrt();
                        if len > 1e-6 {
                            direction_x = dx / len;
                            direction_y = dy / len;
                        }
                    }
                    self.spawn_fragments_from_asteroid(
                        world,
                        asteroid_entity,
                        direction_x,
                        direction_y,
                    );
                    despawn_set.insert(asteroid_entity);
                }
            }
        }

        for entity_id in despawn_set {
            world.despawn(entity_id);
            self.roles.remove(&entity_id);
            self.asteroid_health_points.remove(&entity_id);
            if self.aim_indicator_entity == Some(entity_id) {
                self.aim_indicator_entity = None;
            }
        }
    }

    fn sync_indicator_entity(&mut self, world: &mut World) {
        let entity_id = match self.aim_indicator_entity {
            Some(entity_id) if world.transform(entity_id).is_some() => entity_id,
            _ => {
                let entity_id = world.spawn();
                world.set_mesh_instance_with_material(
                    entity_id,
                    MESH_AIM_INDICATOR,
                    MATERIAL_AIM_INDICATOR,
                );
                self.aim_indicator_entity = Some(entity_id);
                entity_id
            }
        };

        let Some(player_transform) = world.transform(self.anchor_entity).copied() else {
            return;
        };

        if let Some(transform) = world.transform_mut(entity_id) {
            transform.position_x =
                player_transform.position_x + self.aim_angle_rad.cos() * AIM_INDICATOR_OFFSET;
            transform.position_y =
                player_transform.position_y + self.aim_angle_rad.sin() * AIM_INDICATOR_OFFSET;
            transform.rotation_rad = self.aim_angle_rad;
            transform.uniform_scale = AIM_INDICATOR_SCALE;
            transform.velocity_x = 0.0;
            transform.velocity_y = 0.0;
            transform.angular_velocity = 0.0;
        } else {
            world.set_transform(
                entity_id,
                Transform {
                    position_x: player_transform.position_x
                        + self.aim_angle_rad.cos() * AIM_INDICATOR_OFFSET,
                    position_y: player_transform.position_y
                        + self.aim_angle_rad.sin() * AIM_INDICATOR_OFFSET,
                    rotation_rad: self.aim_angle_rad,
                    uniform_scale: AIM_INDICATOR_SCALE,
                    velocity_x: 0.0,
                    velocity_y: 0.0,
                    angular_velocity: 0.0,
                },
            );
        }
    }
}

impl SimulationModel for TriangleMan2Simulation {
    fn handle_input_event(&mut self, event: InputEvent) {
        self.input.handle_input_event(&event);
    }

    fn active_input_context_id(&self) -> &'static str {
        self.input.active_context()
    }

    fn set_active_input_context(&mut self, context_id: &'static str) {
        let _ = self.input.set_active_context(context_id);
    }

    fn pop_input_context_request(&mut self) -> Option<InputContextRequest> {
        self.pending_context_requests.pop_front()
    }

    fn sync_anchor_from_world(&mut self, world: &World, anchor_entity: EntityId) {
        self.locomotion.sync_anchor_from_world(world, anchor_entity);
    }

    fn write_anchor_to_world(&self, world: &mut World, anchor_entity: EntityId) {
        self.locomotion.write_anchor_to_world(world, anchor_entity);
    }

    fn update(&mut self, delta_seconds: f32) {
        if delta_seconds <= 0.0 {
            return;
        }

        if self.game_finished {
            self.fire_requested = false;
            return;
        }

        let frame = self.input.snapshot();
        if self.input.active_context() == TRIANGLE_MAN_2_MENU_CONTEXT
            && self.input.take_menu_confirm_requested()
        {
            self.pending_context_requests
                .push_back(InputContextRequest::Pop);
        }
        self.locomotion
            .set_move_axis(frame.move_left, frame.move_right);
        self.locomotion.set_crawl_pressed(frame.crawl_pressed);
        if frame.jump_requested {
            self.locomotion.request_jump();
        }
        self.fire_cooldown_seconds = (self.fire_cooldown_seconds - delta_seconds).max(0.0);
        self.asteroid_spawn_timer_seconds -= delta_seconds;
        self.fire_requested = frame.fire_requested && self.fire_cooldown_seconds <= 0.0;
        self.locomotion.update(delta_seconds);

        if let Some(desired_aim_angle_rad) = frame.desired_aim_angle_rad {
            self.aim_angle_rad = approach_angle(
                self.aim_angle_rad,
                desired_aim_angle_rad,
                AIM_TURN_RATE_RAD_PER_SEC * delta_seconds,
            );
        }
    }

    fn transform_2d(&self) -> SimulationTransform2D {
        self.locomotion.transform_2d()
    }

    fn reconcile_world(
        &mut self,
        world: &mut World,
        _delta_seconds: f32,
        _frame_report: &SchedulerFrameReport,
    ) {
        self.locomotion.reconcile_world(world, self.anchor_entity);
        self.initialize_level_entities(world);
        self.reconcile_player_platform_contacts(world);

        for (entity_id, transform) in world.transforms_mut() {
            if self.role_for_entity(entity_id) == Some(RoleKind::Asteroid) {
                transform.velocity_y -= ASTEROID_GRAVITY_UNITS_PER_SEC2 * _delta_seconds;
            }
        }

        if self.asteroid_spawn_timer_seconds <= 0.0 {
            self.spawn_asteroid_wave_entry(world);
            let pace_factor = (self.asteroid_spawned_total as f32 / 14.0).clamp(0.0, 1.0);
            self.asteroid_spawn_timer_seconds = ASTEROID_SPAWN_INTERVAL_SECONDS
                - (ASTEROID_SPAWN_INTERVAL_SECONDS - ASTEROID_SPAWN_MIN_INTERVAL_SECONDS)
                    * pace_factor;
        }

        self.sync_indicator_entity(world);

        if self.fire_requested {
            self.spawn_bullet_entity(world);
            self.fire_cooldown_seconds = PLAYER_BASE_FIRE_COOLDOWN_SECONDS;
            self.fire_requested = false;
        }

        self.handle_collision_events(world, _frame_report);

        if let Some(anchor_transform) = world.transform(self.anchor_entity) {
            if anchor_transform.position_y <= WORLD_MIN_FALL_Y {
                self.game_finished = true;
            }
        }

        self.roles
            .retain(|entity_id, _| world.transform(*entity_id).is_some());
        self.asteroid_health_points
            .retain(|entity_id, _| world.transform(*entity_id).is_some());

        if world.transform(self.anchor_entity).is_none() {
            self.aim_indicator_entity = None;
            self.roles.remove(&self.anchor_entity);
            self.asteroid_health_points.remove(&self.anchor_entity);
        }
    }
}

#[derive(Clone, Copy, Default)]
struct TriangleMan2ControlFrame {
    move_left: bool,
    move_right: bool,
    crawl_pressed: bool,
    jump_requested: bool,
    fire_requested: bool,
    desired_aim_angle_rad: Option<f32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TriangleMan2Action {
    MoveLeft,
    MoveRight,
    Jump,
    ToggleCrawl,
    Fire,
    AimLeft,
    AimRight,
    AimUp,
    AimDown,
    MoveAxisX,
    AimAxisX,
    AimAxisY,
    MenuLeft,
    MenuRight,
    MenuConfirm,
}

struct TriangleMan2InputManager {
    contexts: InputContextStack<TriangleMan2Action>,
    move_left: bool,
    move_right: bool,
    keyboard_space_down: bool,
    keyboard_shift_down: bool,
    keyboard_ctrl_down: bool,
    gamepad_south_down: bool,
    gamepad_east_down: bool,
    gamepad_shoulder_down: bool,
    jump_requested: bool,
    crawl_enabled: bool,
    fire_requested: bool,
    keyboard_aim_left: bool,
    keyboard_aim_right: bool,
    keyboard_aim_up: bool,
    keyboard_aim_down: bool,
    gamepad_move_x: f32,
    gamepad_aim_x: f32,
    gamepad_aim_y: f32,
    menu_left: bool,
    menu_right: bool,
    menu_confirm_down: bool,
    menu_confirm_requested: bool,
}

impl Default for TriangleMan2InputManager {
    fn default() -> Self {
        let mut gameplay = InputControlMap::new();
        for (binding, action) in [
            (InputBinding::keyboard("KeyA"), TriangleMan2Action::MoveLeft),
            (InputBinding::keyboard("KeyD"), TriangleMan2Action::MoveRight),
            (InputBinding::keyboard("Space"), TriangleMan2Action::Jump),
            (
                InputBinding::keyboard("ShiftLeft"),
                TriangleMan2Action::ToggleCrawl,
            ),
            (InputBinding::keyboard("ControlLeft"), TriangleMan2Action::Fire),
            (InputBinding::keyboard("ControlRight"), TriangleMan2Action::Fire),
            (InputBinding::keyboard("ArrowLeft"), TriangleMan2Action::AimLeft),
            (InputBinding::keyboard("ArrowRight"), TriangleMan2Action::AimRight),
            (InputBinding::keyboard("ArrowUp"), TriangleMan2Action::AimUp),
            (InputBinding::keyboard("ArrowDown"), TriangleMan2Action::AimDown),
            (InputBinding::gamepad("left_stick_x"), TriangleMan2Action::MoveAxisX),
            (InputBinding::gamepad("south_button"), TriangleMan2Action::Jump),
            (InputBinding::gamepad("east_button"), TriangleMan2Action::ToggleCrawl),
            (
                InputBinding::gamepad("right_shoulder"),
                TriangleMan2Action::Fire,
            ),
            (InputBinding::gamepad("right_stick_x"), TriangleMan2Action::AimAxisX),
            (InputBinding::gamepad("right_stick_y"), TriangleMan2Action::AimAxisY),
        ] {
            gameplay
                .bind_rom_action(binding, action)
                .expect("Triangle Man 2 gameplay bindings should not use reserved keys");
        }

        let mut menu = InputControlMap::new();
        for (binding, action) in [
            (InputBinding::keyboard("KeyA"), TriangleMan2Action::MenuLeft),
            (InputBinding::keyboard("KeyD"), TriangleMan2Action::MenuRight),
            (
                InputBinding::keyboard("Space"),
                TriangleMan2Action::MenuConfirm,
            ),
            (InputBinding::gamepad("south_button"), TriangleMan2Action::MenuConfirm),
        ] {
            menu.bind_rom_action(binding, action)
                .expect("Triangle Man 2 menu bindings should not use reserved keys");
        }

        let mut contexts = InputContextStack::new(TRIANGLE_MAN_2_GAMEPLAY_CONTEXT, gameplay);
        contexts.register_context(TRIANGLE_MAN_2_MENU_CONTEXT, menu);

        Self {
            contexts,
            move_left: false,
            move_right: false,
            keyboard_space_down: false,
            keyboard_shift_down: false,
            keyboard_ctrl_down: false,
            gamepad_south_down: false,
            gamepad_east_down: false,
            gamepad_shoulder_down: false,
            jump_requested: false,
            crawl_enabled: false,
            fire_requested: false,
            keyboard_aim_left: false,
            keyboard_aim_right: false,
            keyboard_aim_up: false,
            keyboard_aim_down: false,
            gamepad_move_x: 0.0,
            gamepad_aim_x: 0.0,
            gamepad_aim_y: 0.0,
            menu_left: false,
            menu_right: false,
            menu_confirm_down: false,
            menu_confirm_requested: false,
        }
    }
}

impl TriangleMan2InputManager {
    fn active_context(&self) -> &'static str {
        self.contexts.active_context()
    }

    fn push_context(&mut self, context_id: &'static str) -> bool {
        self.contexts.push_context(context_id)
    }

    fn pop_context(&mut self) -> bool {
        self.contexts.pop_context()
    }

    fn replace_active_context(&mut self, context_id: &'static str) -> bool {
        self.contexts.replace_active_context(context_id)
    }

    fn set_active_context(&mut self, context_id: &'static str) -> bool {
        self.replace_active_context(context_id)
    }

    fn handle_input_event(&mut self, event: &InputEvent) {
        if event.is_repeat && event.is_pressed {
            return;
        }

        let Some(action_event) = self.contexts.resolve_event(event) else {
            return;
        };

        match action_event.context_id {
            TRIANGLE_MAN_2_GAMEPLAY_CONTEXT => match action_event.action {
                TriangleMan2Action::MoveLeft => self.move_left = action_event.is_pressed,
                TriangleMan2Action::MoveRight => self.move_right = action_event.is_pressed,
                TriangleMan2Action::Jump => {
                    if action_event.is_pressed && !self.keyboard_space_down && !self.gamepad_south_down {
                        self.jump_requested = true;
                    }

                    match event.source.as_str() {
                        "keyboard" => self.keyboard_space_down = action_event.is_pressed,
                        "gamepad" => self.gamepad_south_down = action_event.is_pressed,
                        _ => {}
                    }
                }
                TriangleMan2Action::ToggleCrawl => {
                    let button_down = match event.source.as_str() {
                        "keyboard" => &mut self.keyboard_shift_down,
                        "gamepad" => &mut self.gamepad_east_down,
                        _ => return,
                    };

                    if action_event.is_pressed && !*button_down {
                        self.crawl_enabled = !self.crawl_enabled;
                    }
                    *button_down = action_event.is_pressed;
                }
                TriangleMan2Action::Fire => {
                    if action_event.is_pressed
                        && !self.keyboard_ctrl_down
                        && !self.gamepad_shoulder_down
                    {
                        self.fire_requested = true;
                    }

                    match event.source.as_str() {
                        "keyboard" => self.keyboard_ctrl_down = action_event.is_pressed,
                        "gamepad" => self.gamepad_shoulder_down = action_event.is_pressed,
                        _ => {}
                    }
                }
                TriangleMan2Action::AimLeft => self.keyboard_aim_left = action_event.is_pressed,
                TriangleMan2Action::AimRight => self.keyboard_aim_right = action_event.is_pressed,
                TriangleMan2Action::AimUp => self.keyboard_aim_up = action_event.is_pressed,
                TriangleMan2Action::AimDown => self.keyboard_aim_down = action_event.is_pressed,
                TriangleMan2Action::MoveAxisX => {
                    self.gamepad_move_x = apply_deadzone(action_event.value, AIM_INPUT_DEADZONE);
                }
                TriangleMan2Action::AimAxisX => {
                    self.gamepad_aim_x = apply_deadzone(action_event.value, AIM_INPUT_DEADZONE);
                }
                TriangleMan2Action::AimAxisY => {
                    self.gamepad_aim_y = apply_deadzone(-action_event.value, AIM_INPUT_DEADZONE);
                }
                TriangleMan2Action::MenuLeft
                | TriangleMan2Action::MenuRight
                | TriangleMan2Action::MenuConfirm => {}
            },
            TRIANGLE_MAN_2_MENU_CONTEXT => match action_event.action {
                TriangleMan2Action::MenuLeft => self.menu_left = action_event.is_pressed,
                TriangleMan2Action::MenuRight => self.menu_right = action_event.is_pressed,
                TriangleMan2Action::MenuConfirm => {
                    if action_event.is_pressed && !self.menu_confirm_down {
                        self.menu_confirm_requested = true;
                    }
                    self.menu_confirm_down = action_event.is_pressed;
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn snapshot(&mut self) -> TriangleMan2ControlFrame {
        let digital_move_left = self.move_left;
        let digital_move_right = self.move_right;
        let gamepad_move_left = self.gamepad_move_x < -AIM_INPUT_DEADZONE;
        let gamepad_move_right = self.gamepad_move_x > AIM_INPUT_DEADZONE;

        let move_left = digital_move_left || gamepad_move_left;
        let move_right = digital_move_right || gamepad_move_right;

        let desired_aim_angle_rad = if self.gamepad_aim_x.abs() > AIM_INPUT_DEADZONE
            || self.gamepad_aim_y.abs() > AIM_INPUT_DEADZONE
        {
            Some(self.gamepad_aim_y.atan2(self.gamepad_aim_x))
        } else {
            let keyboard_aim_x = axis_value(self.keyboard_aim_left, self.keyboard_aim_right) as f32;
            let keyboard_aim_y = axis_value(self.keyboard_aim_down, self.keyboard_aim_up) as f32;
            if keyboard_aim_x.abs() > f32::EPSILON || keyboard_aim_y.abs() > f32::EPSILON {
                Some(keyboard_aim_y.atan2(keyboard_aim_x))
            } else {
                None
            }
        };

        let frame = TriangleMan2ControlFrame {
            move_left,
            move_right,
            crawl_pressed: self.crawl_enabled,
            jump_requested: self.jump_requested,
            fire_requested: self.fire_requested,
            desired_aim_angle_rad,
        };

        self.jump_requested = false;
        self.fire_requested = false;

        frame
    }

    fn take_menu_confirm_requested(&mut self) -> bool {
        let requested = self.menu_confirm_requested;
        self.menu_confirm_requested = false;
        requested
    }
}

fn triangle_man_2_render_data(webgl_compat: bool) -> RomRenderData {
    let mut vertex_bytes = Vec::with_capacity(
        (PLATFORMER_GROUND_VERTICES.len()
            + PLATFORMER_PLAYER_VERTICES.len()
            + AIM_INDICATOR_VERTICES.len()
            + PLATFORM_VERTICES.len()
            + ASTEROID_VERTICES.len()
            + BULLET_VERTICES.len()
            + GOAL_VERTICES.len())
            * std::mem::size_of::<PlatformerVertex>(),
    );
    vertex_bytes.extend_from_slice(bytemuck::cast_slice(&PLATFORMER_GROUND_VERTICES));
    vertex_bytes.extend_from_slice(bytemuck::cast_slice(&PLATFORMER_PLAYER_VERTICES));
    vertex_bytes.extend_from_slice(bytemuck::cast_slice(&AIM_INDICATOR_VERTICES));
    vertex_bytes.extend_from_slice(bytemuck::cast_slice(&PLATFORM_VERTICES));
    vertex_bytes.extend_from_slice(bytemuck::cast_slice(&ASTEROID_VERTICES));
    vertex_bytes.extend_from_slice(bytemuck::cast_slice(&BULLET_VERTICES));
    vertex_bytes.extend_from_slice(bytemuck::cast_slice(&GOAL_VERTICES));
    let vertex_bytes: &'static [u8] = Box::leak(vertex_bytes.into_boxed_slice());

    RomRenderData {
        shader_source_wgsl: PLATFORMER_SHADER,
        webgl_shader_source_wgsl: if webgl_compat {
            Some(PLATFORMER_SHADER_WEBGL)
        } else {
            None
        },
        vertex_layout: PlatformerVertex::desc(),
        vertex_bytes,
        vertex_count: (PLATFORMER_GROUND_VERTICES.len()
            + PLATFORMER_PLAYER_VERTICES.len()
            + AIM_INDICATOR_VERTICES.len()
            + PLATFORM_VERTICES.len()
            + ASTEROID_VERTICES.len()
            + BULLET_VERTICES.len()
            + GOAL_VERTICES.len()) as u32,
        materials: &TRIANGLE_MAN_2_MATERIALS,
    }
}

fn register_triangle_man_2_assets(world: &mut World) {
    register_platformer_assets(world);
    world.register_mesh_asset(
        MESH_AIM_INDICATOR,
        Mesh {
            vertex_bytes: bytemuck::cast_slice(&AIM_INDICATOR_VERTICES),
        },
    );
    world.register_mesh_asset(
        MESH_PLATFORM,
        Mesh {
            vertex_bytes: bytemuck::cast_slice(&PLATFORM_VERTICES),
        },
    );
    world.register_mesh_asset(
        MESH_ASTEROID,
        Mesh {
            vertex_bytes: bytemuck::cast_slice(&ASTEROID_VERTICES),
        },
    );
    world.register_mesh_asset(
        MESH_BULLET,
        Mesh {
            vertex_bytes: bytemuck::cast_slice(&BULLET_VERTICES),
        },
    );
    world.register_mesh_asset(
        MESH_GOAL,
        Mesh {
            vertex_bytes: bytemuck::cast_slice(&GOAL_VERTICES),
        },
    );
}

fn apply_deadzone(value: f32, deadzone: f32) -> f32 {
    if value.abs() < deadzone { 0.0 } else { value }
}

fn axis_value(negative: bool, positive: bool) -> i8 {
    match (negative, positive) {
        (true, false) => -1,
        (false, true) => 1,
        _ => 0,
    }
}

fn angle_delta(target: f32, current: f32) -> f32 {
    let mut delta = target - current;
    while delta > PI {
        delta -= 2.0 * PI;
    }
    while delta < -PI {
        delta += 2.0 * PI;
    }
    delta
}

fn approach_angle(current: f32, target: f32, max_step: f32) -> f32 {
    let delta = angle_delta(target, current);
    if delta.abs() <= max_step {
        target
    } else {
        current + delta.signum() * max_step
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ASTEROID_MIN_FRAGMENT_SCALE, RoleKind, TRIANGLE_MAN_2_INITIAL_ASTEROIDS,
        TRIANGLE_MAN_2_LEVEL_PLATFORMS, TRIANGLE_MAN_2_GAMEPLAY_CONTEXT,
        TRIANGLE_MAN_2_MENU_CONTEXT, TriangleMan2InputManager, TriangleMan2Simulation,
    };
    use crate::ecs::{EntityId, Transform, World};
    use crate::input::InputEvent;
    use crate::platformer_common::PLATFORMER_STANDING_SCALE;
    use crate::simulation::{
        InteractionEvent, InteractionEventKind, PhysicsConfig, Scheduler, SimulationModel,
    };

    fn simulation_with_anchor() -> (TriangleMan2Simulation, World, EntityId) {
        let mut world = World::new();
        let anchor = world.spawn();
        let standing_scale = PLATFORMER_STANDING_SCALE;
        let ground_y = crate::platformer_common::platformer_ground_contact_y(standing_scale);
        world.set_transform(
            anchor,
            Transform {
                position_x: -0.35,
                position_y: ground_y,
                rotation_rad: 0.0,
                uniform_scale: standing_scale,
                velocity_x: 0.0,
                velocity_y: 0.0,
                angular_velocity: 0.0,
            },
        );
        (
            TriangleMan2Simulation::new(anchor, world.transform(anchor).copied().unwrap()),
            world,
            anchor,
        )
    }

    #[test]
    fn keyboard_move_jump_and_duck_toggle_work() {
        let (mut simulation, _, _) = simulation_with_anchor();

        simulation.handle_input_event(InputEvent {
            source: "keyboard".to_owned(),
            control: "KeyA".to_owned(),
            value: 1.0,
            device_index: None,
            is_pressed: true,
            is_repeat: false,
        });
        simulation.handle_input_event(InputEvent {
            source: "keyboard".to_owned(),
            control: "Space".to_owned(),
            value: 1.0,
            device_index: None,
            is_pressed: true,
            is_repeat: false,
        });
        simulation.handle_input_event(InputEvent {
            source: "keyboard".to_owned(),
            control: "ShiftLeft".to_owned(),
            value: 1.0,
            device_index: None,
            is_pressed: true,
            is_repeat: false,
        });
        simulation.update(1.0 / 60.0);

        assert!(simulation.locomotion.velocity_x < 0.0);
        assert!(simulation.locomotion.velocity_y > 0.0);
        assert!(simulation.locomotion.uniform_scale < PLATFORMER_STANDING_SCALE);
    }

    #[test]
    fn keyboard_arrow_keys_steer_continuous_aim() {
        let (mut simulation, _, _) = simulation_with_anchor();

        simulation.handle_input_event(InputEvent {
            source: "keyboard".to_owned(),
            control: "ArrowRight".to_owned(),
            value: 1.0,
            device_index: None,
            is_pressed: true,
            is_repeat: false,
        });
        simulation.update(1.0 / 60.0);
        let initial_aim = simulation.aim_angle_rad;

        simulation.handle_input_event(InputEvent {
            source: "keyboard".to_owned(),
            control: "ArrowUp".to_owned(),
            value: 1.0,
            device_index: None,
            is_pressed: true,
            is_repeat: false,
        });
        for _ in 0..6 {
            simulation.update(1.0 / 60.0);
        }

        assert!(initial_aim.abs() < 0.1);
        assert!(simulation.aim_angle_rad > 0.0);
        assert!(simulation.aim_angle_rad < std::f32::consts::FRAC_PI_2 + 0.05);
    }

    #[test]
    fn gamepad_left_stick_and_right_stick_map_to_movement_and_aim() {
        let (mut simulation, _, _) = simulation_with_anchor();

        simulation.handle_input_event(InputEvent {
            source: "gamepad".to_owned(),
            control: "left_stick_x".to_owned(),
            value: -0.8,
            device_index: Some(0),
            is_pressed: true,
            is_repeat: false,
        });
        simulation.handle_input_event(InputEvent {
            source: "gamepad".to_owned(),
            control: "right_stick_x".to_owned(),
            value: 1.0,
            device_index: Some(0),
            is_pressed: true,
            is_repeat: false,
        });
        simulation.handle_input_event(InputEvent {
            source: "gamepad".to_owned(),
            control: "right_stick_y".to_owned(),
            value: 0.0,
            device_index: Some(0),
            is_pressed: false,
            is_repeat: false,
        });
        simulation.update(1.0 / 60.0);

        assert!(simulation.locomotion.velocity_x < 0.0);
        assert!(simulation.aim_angle_rad.abs() < 0.1);
    }

    #[test]
    fn gamepad_right_stick_y_is_reversed_for_aiming() {
        let (mut simulation, _, _) = simulation_with_anchor();

        simulation.handle_input_event(InputEvent {
            source: "gamepad".to_owned(),
            control: "right_stick_y".to_owned(),
            value: -1.0,
            device_index: Some(0),
            is_pressed: true,
            is_repeat: false,
        });
        simulation.update(1.0 / 60.0);

        assert!(simulation.aim_angle_rad > 0.0);
    }

    #[test]
    fn aim_indicator_is_spawned_and_tracks_rotation() {
        let (mut simulation, mut world, _) = simulation_with_anchor();

        simulation.handle_input_event(InputEvent {
            source: "keyboard".to_owned(),
            control: "ArrowUp".to_owned(),
            value: 1.0,
            device_index: None,
            is_pressed: true,
            is_repeat: false,
        });
        simulation.update(1.0 / 60.0);
        simulation.write_anchor_to_world(&mut world, simulation.anchor_entity);
        simulation.reconcile_world(
            &mut world,
            1.0 / 60.0,
            &crate::simulation::SchedulerFrameReport::default(),
        );

        let indicator_entity = simulation
            .aim_indicator_entity
            .expect("aim indicator should be spawned");
        let indicator_transform = world
            .transform(indicator_entity)
            .copied()
            .expect("aim indicator should have a transform");

        assert!(indicator_transform.rotation_rad > 0.0);
        assert!(indicator_transform.uniform_scale > 0.0);
    }

    #[test]
    fn keyboard_ctrl_spawns_a_short_lived_fire_pulse() {
        let (mut simulation, mut world_fire, _) = simulation_with_anchor();
        let (mut control_simulation, mut world_control, _) = simulation_with_anchor();

        simulation.handle_input_event(InputEvent {
            source: "keyboard".to_owned(),
            control: "ControlLeft".to_owned(),
            value: 1.0,
            device_index: None,
            is_pressed: true,
            is_repeat: false,
        });
        simulation.update(1.0 / 60.0);
        simulation.write_anchor_to_world(&mut world_fire, simulation.anchor_entity);
        simulation.reconcile_world(
            &mut world_fire,
            1.0 / 60.0,
            &crate::simulation::SchedulerFrameReport::default(),
        );

        control_simulation.update(1.0 / 60.0);
        control_simulation
            .write_anchor_to_world(&mut world_control, control_simulation.anchor_entity);
        control_simulation.reconcile_world(
            &mut world_control,
            1.0 / 60.0,
            &crate::simulation::SchedulerFrameReport::default(),
        );

        assert!(world_fire.transforms().count() > world_control.transforms().count());
    }

    #[test]
    fn nested_rom_context_uses_its_own_control_map() {
        let mut input = TriangleMan2InputManager::default();

        assert_eq!(input.active_context(), TRIANGLE_MAN_2_GAMEPLAY_CONTEXT);
        assert!(input.push_context(TRIANGLE_MAN_2_MENU_CONTEXT));
        assert_eq!(input.active_context(), TRIANGLE_MAN_2_MENU_CONTEXT);

        input.handle_input_event(&InputEvent {
            source: "keyboard".to_owned(),
            control: "Space".to_owned(),
            value: 1.0,
            device_index: None,
            is_pressed: true,
            is_repeat: false,
        });

        let frame = input.snapshot();
        assert!(!frame.jump_requested);
        assert!(input.take_menu_confirm_requested());

        assert!(input.pop_context());
        assert_eq!(input.active_context(), TRIANGLE_MAN_2_GAMEPLAY_CONTEXT);

        input.handle_input_event(&InputEvent {
            source: "keyboard".to_owned(),
            control: "Space".to_owned(),
            value: 1.0,
            device_index: None,
            is_pressed: false,
            is_repeat: false,
        });
        input.handle_input_event(&InputEvent {
            source: "keyboard".to_owned(),
            control: "Space".to_owned(),
            value: 1.0,
            device_index: None,
            is_pressed: true,
            is_repeat: false,
        });

        assert!(input.snapshot().jump_requested);
    }

    #[test]
    fn keyboard_right_ctrl_also_spawns_shot() {
        let (mut simulation, mut world_fire, _) = simulation_with_anchor();
        let (mut control_simulation, mut world_control, _) = simulation_with_anchor();

        simulation.handle_input_event(InputEvent {
            source: "keyboard".to_owned(),
            control: "ControlRight".to_owned(),
            value: 1.0,
            device_index: None,
            is_pressed: true,
            is_repeat: false,
        });
        simulation.update(1.0 / 60.0);
        simulation.write_anchor_to_world(&mut world_fire, simulation.anchor_entity);
        simulation.reconcile_world(
            &mut world_fire,
            1.0 / 60.0,
            &crate::simulation::SchedulerFrameReport::default(),
        );

        control_simulation.update(1.0 / 60.0);
        control_simulation
            .write_anchor_to_world(&mut world_control, control_simulation.anchor_entity);
        control_simulation.reconcile_world(
            &mut world_control,
            1.0 / 60.0,
            &crate::simulation::SchedulerFrameReport::default(),
        );

        assert!(world_fire.transforms().count() > world_control.transforms().count());
    }

    #[test]
    fn reconcile_spawns_platforms_goal_and_seed_asteroids() {
        let (mut simulation, mut world, _) = simulation_with_anchor();

        simulation.update(1.0 / 60.0);
        simulation.write_anchor_to_world(&mut world, simulation.anchor_entity);
        simulation.reconcile_world(
            &mut world,
            1.0 / 60.0,
            &crate::simulation::SchedulerFrameReport::default(),
        );

        let platform_count = simulation
            .roles
            .values()
            .filter(|role| **role == RoleKind::Platform)
            .count();
        let asteroid_count = simulation
            .roles
            .values()
            .filter(|role| **role == RoleKind::Asteroid)
            .count();
        let goal_count = simulation
            .roles
            .values()
            .filter(|role| **role == RoleKind::Goal)
            .count();
        let player_transform = world
            .transform(simulation.anchor_entity)
            .copied()
            .expect("player should have transform");
        let expected_ground_y =
            crate::platformer_common::platformer_ground_contact_y(PLATFORMER_STANDING_SCALE);

        assert_eq!(platform_count, TRIANGLE_MAN_2_LEVEL_PLATFORMS.len());
        assert!(asteroid_count >= TRIANGLE_MAN_2_INITIAL_ASTEROIDS.len());
        assert_eq!(goal_count, 1);
        assert!((player_transform.position_y - expected_ground_y).abs() < 0.001);
    }

    #[test]
    fn asteroids_collide_with_platform_surfaces() {
        let (mut simulation, mut world, _) = simulation_with_anchor();

        simulation.update(1.0 / 60.0);
        simulation.write_anchor_to_world(&mut world, simulation.anchor_entity);
        simulation.reconcile_world(
            &mut world,
            1.0 / 60.0,
            &crate::simulation::SchedulerFrameReport::default(),
        );

        let platform_entity = simulation
            .roles
            .iter()
            .find_map(|(entity_id, role)| {
                if *role == RoleKind::Platform {
                    Some(*entity_id)
                } else {
                    None
                }
            })
            .expect("platform should exist");
        let platform_transform = world
            .transform(platform_entity)
            .copied()
            .expect("platform should have transform");

        let asteroid = simulation.spawn_asteroid_entity(
            &mut world,
            platform_transform.position_x,
            platform_transform.position_y + 0.28,
            0.0,
            -0.25,
            super::ASTEROID_BASE_SCALE,
        );

        let mut scheduler = Scheduler::new(PhysicsConfig::default());
        let mut collided = false;
        for _ in 0..90 {
            let report = scheduler.update_world(&mut world, 1.0 / 60.0);
            collided = collided
                || report.interaction_events.iter().any(|event| {
                    event.kind == InteractionEventKind::Collision
                        && ((event.entity_a == asteroid && event.entity_b == platform_entity)
                            || (event.entity_b == asteroid && event.entity_a == platform_entity))
                });

            simulation.reconcile_world(&mut world, 1.0 / 60.0, &report);
            if collided {
                break;
            }
        }

        assert!(collided);
    }

    #[test]
    fn bullet_collision_destroys_small_asteroid() {
        let (mut simulation, mut world, _) = simulation_with_anchor();
        simulation.level_initialized = true;
        simulation
            .roles
            .insert(simulation.anchor_entity, RoleKind::Player);

        let asteroid = simulation.spawn_asteroid_entity(
            &mut world,
            0.0,
            0.0,
            0.0,
            0.0,
            ASTEROID_MIN_FRAGMENT_SCALE * 1.1,
        );
        simulation.aim_angle_rad = 0.0;
        simulation.spawn_bullet_entity(&mut world);
        let bullet = simulation
            .roles
            .iter()
            .find_map(|(entity_id, role)| {
                if *role == RoleKind::Bullet {
                    Some(*entity_id)
                } else {
                    None
                }
            })
            .expect("bullet should exist");

        let mut report = crate::simulation::SchedulerFrameReport::default();
        report.interaction_events.push(InteractionEvent {
            entity_a: asteroid,
            entity_b: bullet,
            kind: InteractionEventKind::Collision,
        });

        simulation.reconcile_world(&mut world, 1.0 / 60.0, &report);

        assert!(world.transform(asteroid).is_none());
        assert!(world.transform(bullet).is_none());
    }
}
