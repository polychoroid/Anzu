use crate::ecs::{EntityId, MaterialId, Mesh, MeshAssetId, Transform, World};
use crate::input::InputEvent;
use crate::renderer::{MaterialBlendMode, MaterialDefinition, RomRenderData};
use crate::simulation::SimulationTransform2D;

pub const PLATFORMER_PLAYER_MESH: MeshAssetId = MeshAssetId(11);
pub const PLATFORMER_GROUND_MESH: MeshAssetId = MeshAssetId(12);

pub const PLATFORMER_PLAYER_MATERIAL: MaterialId = MaterialId(11);
pub const PLATFORMER_GROUND_MATERIAL: MaterialId = MaterialId(12);

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PlatformerVertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
    pub barycentric: [f32; 3],
    pub light_pos: [f32; 2],
    pub edge_mask: [f32; 3],
}

impl PlatformerVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x3,
        2 => Float32x3,
        3 => Float32x2,
        4 => Float32x3
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<PlatformerVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub const PLATFORMER_PLAYER_VERTICES: [PlatformerVertex; 3] = [
    PlatformerVertex {
        position: [-0.45, -0.42],
        color: [0.90, 0.72, 0.30],
        barycentric: [1.0, 0.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [0.45, -0.42],
        color: [0.95, 0.80, 0.40],
        barycentric: [0.0, 1.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [0.0, 0.56],
        color: [0.98, 0.90, 0.58],
        barycentric: [0.0, 0.0, 1.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
];

pub const PLATFORMER_GROUND_VERTICES: [PlatformerVertex; 6] = [
    PlatformerVertex {
        position: [-1.2, -0.06],
        color: [0.18, 0.36, 0.24],
        barycentric: [1.0, 0.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [1.2, -0.06],
        color: [0.18, 0.36, 0.24],
        barycentric: [0.0, 1.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [1.2, 0.06],
        color: [0.24, 0.48, 0.30],
        barycentric: [0.0, 0.0, 1.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [-1.2, -0.06],
        color: [0.18, 0.36, 0.24],
        barycentric: [1.0, 0.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [1.2, 0.06],
        color: [0.24, 0.48, 0.30],
        barycentric: [0.0, 1.0, 0.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
    PlatformerVertex {
        position: [-1.2, 0.06],
        color: [0.20, 0.42, 0.26],
        barycentric: [0.0, 0.0, 1.0],
        light_pos: [0.0, 0.0],
        edge_mask: [1.0, 1.0, 1.0],
    },
];

pub const PLATFORMER_SHADER: &str = r#"
struct RotationUniform {
    angle: f32,
    scale: f32,
    translation: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> u_rotation: RotationUniform;

struct MaterialUniform {
    base_color_tint: vec3<f32>,
    emissive_strength: f32,
    shading_params: vec4<f32>,
};

@group(0) @binding(1)
var<uniform> u_material: MaterialUniform;

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
    let base_color = in.color * u_material.base_color_tint;
    let lit_color = clamp(base_color + vec3<f32>(u_material.emissive_strength), vec3<f32>(0.0), vec3<f32>(3.0));
    return vec4<f32>(lit_color, 1.0);
}
"#;

pub const PLATFORMER_SHADER_WEBGL: &str = PLATFORMER_SHADER;

pub const PLATFORMER_MATERIALS: [MaterialDefinition; 2] = [
    MaterialDefinition {
        material_id: PLATFORMER_GROUND_MATERIAL,
        blend_mode: MaterialBlendMode::Alpha,
        base_color_tint: [1.0, 1.0, 1.0],
        emissive_strength: 0.0,
        metallic: 0.0,
        roughness: 0.8,
        specular_strength: 0.1,
    },
    MaterialDefinition {
        material_id: PLATFORMER_PLAYER_MATERIAL,
        blend_mode: MaterialBlendMode::Alpha,
        base_color_tint: [1.0, 1.0, 1.0],
        emissive_strength: 0.0,
        metallic: 0.0,
        roughness: 0.7,
        specular_strength: 0.1,
    },
];

pub const PLATFORMER_STANDING_SCALE: f32 = 0.15;
pub const PLATFORMER_CRAWL_SCALE: f32 = 0.11;
const PLATFORMER_GROUND_CENTER_Y: f32 = -0.78;
const PLATFORMER_GROUND_HALF_THICKNESS: f32 = 0.06;
pub const PLATFORMER_GROUND_TOP_Y: f32 =
    PLATFORMER_GROUND_CENTER_Y + PLATFORMER_GROUND_HALF_THICKNESS;
pub const PLATFORMER_PLAYER_BOTTOM_OFFSET: f32 = 0.42;

pub fn platformer_ground_contact_y(scale: f32) -> f32 {
    PLATFORMER_GROUND_TOP_Y + PLATFORMER_PLAYER_BOTTOM_OFFSET * scale
}

pub fn platformer_render_data(webgl_compat: bool) -> RomRenderData {
    let mut vertex_bytes = Vec::with_capacity(
        (PLATFORMER_PLAYER_VERTICES.len() + PLATFORMER_GROUND_VERTICES.len())
            * std::mem::size_of::<PlatformerVertex>(),
    );
    vertex_bytes.extend_from_slice(bytemuck::cast_slice(&PLATFORMER_GROUND_VERTICES));
    vertex_bytes.extend_from_slice(bytemuck::cast_slice(&PLATFORMER_PLAYER_VERTICES));
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
        vertex_count: (PLATFORMER_GROUND_VERTICES.len() + PLATFORMER_PLAYER_VERTICES.len()) as u32,
        materials: &PLATFORMER_MATERIALS,
    }
}

pub fn register_platformer_assets(world: &mut World) {
    world.register_mesh_asset(
        PLATFORMER_GROUND_MESH,
        Mesh {
            vertex_bytes: bytemuck::cast_slice(&PLATFORMER_GROUND_VERTICES),
        },
    );
    world.register_mesh_asset(
        PLATFORMER_PLAYER_MESH,
        Mesh {
            vertex_bytes: bytemuck::cast_slice(&PLATFORMER_PLAYER_VERTICES),
        },
    );
}

pub fn spawn_platformer_ground(world: &mut World) -> EntityId {
    let entity_id = world.spawn();
    world.set_transform(
        entity_id,
        Transform {
            position_x: 0.0,
            position_y: PLATFORMER_GROUND_CENTER_Y,
            rotation_rad: 0.0,
            uniform_scale: 1.0,
            velocity_x: 0.0,
            velocity_y: 0.0,
            angular_velocity: 0.0,
        },
    );
    world.set_mesh_instance_with_material(
        entity_id,
        PLATFORMER_GROUND_MESH,
        PLATFORMER_GROUND_MATERIAL,
    );
    entity_id
}

pub fn spawn_platformer_player(
    world: &mut World,
    position_x: f32,
    position_y: f32,
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
            velocity_x: 0.0,
            velocity_y: 0.0,
            angular_velocity: 0.0,
        },
    );
    world.set_mesh_instance_with_material(
        entity_id,
        PLATFORMER_PLAYER_MESH,
        PLATFORMER_PLAYER_MATERIAL,
    );
    entity_id
}

#[derive(Clone, Copy, Debug)]
pub struct PlatformerTuning {
    pub walk_accel_units_per_sec2: f32,
    pub crawl_accel_units_per_sec2: f32,
    pub air_accel_units_per_sec2: f32,
    pub ground_drag_per_sec: f32,
    pub air_drag_per_sec: f32,
    pub gravity_units_per_sec2: f32,
    pub jump_speed_units_per_sec: f32,
    pub walk_max_speed_units_per_sec: f32,
    pub crawl_max_speed_units_per_sec: f32,
    pub standing_scale: f32,
    pub crawl_scale: f32,
    pub lean_factor: f32,
    pub lean_cap_rad: f32,
}

impl Default for PlatformerTuning {
    fn default() -> Self {
        Self {
            walk_accel_units_per_sec2: 3.4,
            crawl_accel_units_per_sec2: 2.2,
            air_accel_units_per_sec2: 1.0,
            ground_drag_per_sec: 7.0,
            air_drag_per_sec: 0.5,
            gravity_units_per_sec2: 3.8,
            jump_speed_units_per_sec: 1.25,
            walk_max_speed_units_per_sec: 0.55,
            crawl_max_speed_units_per_sec: 0.28,
            standing_scale: PLATFORMER_STANDING_SCALE,
            crawl_scale: PLATFORMER_CRAWL_SCALE,
            lean_factor: 0.38,
            lean_cap_rad: 0.18,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PlatformerLocomotionState {
    pub position_x: f32,
    pub position_y: f32,
    pub rotation_rad: f32,
    pub uniform_scale: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub on_ground: bool,
    move_left: bool,
    move_right: bool,
    crawl_pressed: bool,
    jump_requested: bool,
    tuning: PlatformerTuning,
}

impl PlatformerLocomotionState {
    pub fn new(initial: Transform, tuning: PlatformerTuning) -> Self {
        let scale = if initial.uniform_scale > 0.0 {
            initial.uniform_scale
        } else {
            tuning.standing_scale
        };
        let ground_contact = platformer_ground_contact_y(scale);
        let on_ground = initial.position_y <= ground_contact + 0.002 && initial.velocity_y <= 0.0;

        Self {
            position_x: initial.position_x,
            position_y: if initial.position_y.abs() <= f32::EPSILON {
                ground_contact
            } else {
                initial.position_y
            },
            rotation_rad: initial.rotation_rad,
            uniform_scale: scale,
            velocity_x: initial.velocity_x,
            velocity_y: initial.velocity_y,
            on_ground,
            move_left: false,
            move_right: false,
            crawl_pressed: false,
            jump_requested: false,
            tuning,
        }
    }

    pub fn set_move_axis(&mut self, move_left: bool, move_right: bool) {
        self.move_left = move_left;
        self.move_right = move_right;
    }

    pub fn set_crawl_pressed(&mut self, crawl_pressed: bool) {
        self.crawl_pressed = crawl_pressed;
    }

    pub fn request_jump(&mut self) {
        self.jump_requested = true;
    }

    pub fn handle_input_event(&mut self, event: &InputEvent) {
        if event.source != "keyboard" {
            return;
        }

        if event.is_repeat && event.is_pressed {
            return;
        }

        match event.control.as_str() {
            "KeyA" => self.move_left = event.is_pressed,
            "KeyD" => self.move_right = event.is_pressed,
            "KeyS" => self.crawl_pressed = event.is_pressed,
            "KeyW" if event.is_pressed => self.jump_requested = true,
            _ => {}
        }
    }

    pub fn sync_anchor_from_world(&mut self, world: &World, anchor_entity: EntityId) {
        if let Some(transform) = world.transform(anchor_entity) {
            self.position_x = transform.position_x;
            self.position_y = transform.position_y;
            self.rotation_rad = transform.rotation_rad;
            self.uniform_scale = transform.uniform_scale;
            self.velocity_x = transform.velocity_x;
            self.velocity_y = transform.velocity_y;
            self.on_ground = self.position_y
                <= platformer_ground_contact_y(self.uniform_scale) + 0.002
                && self.velocity_y <= 0.0;
        }
    }

    pub fn write_anchor_to_world(&self, world: &mut World, anchor_entity: EntityId) {
        if let Some(transform) = world.transform_mut(anchor_entity) {
            transform.position_x = self.position_x;
            transform.position_y = self.position_y;
            transform.rotation_rad = self.rotation_rad;
            transform.uniform_scale = self.uniform_scale;
            transform.velocity_x = self.velocity_x;
            transform.velocity_y = self.velocity_y;
            transform.angular_velocity = 0.0;
        }
    }

    pub fn update(&mut self, delta_seconds: f32) {
        if delta_seconds <= 0.0 {
            return;
        }

        let move_axis = match (self.move_left, self.move_right) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        };
        let crawling = self.crawl_pressed;
        let desired_speed = if crawling {
            self.tuning.crawl_max_speed_units_per_sec
        } else {
            self.tuning.walk_max_speed_units_per_sec
        };
        let target_velocity_x = move_axis * desired_speed;
        let accel = if self.on_ground {
            if crawling {
                self.tuning.crawl_accel_units_per_sec2
            } else {
                self.tuning.walk_accel_units_per_sec2
            }
        } else {
            self.tuning.air_accel_units_per_sec2
        };

        let max_delta_vx = accel * delta_seconds;
        self.velocity_x += (target_velocity_x - self.velocity_x).clamp(-max_delta_vx, max_delta_vx);

        if self.on_ground && move_axis.abs() <= f32::EPSILON {
            let drag = (1.0 - self.tuning.ground_drag_per_sec * delta_seconds).clamp(0.0, 1.0);
            self.velocity_x *= drag;
        } else if !self.on_ground {
            let drag = (1.0 - self.tuning.air_drag_per_sec * delta_seconds).clamp(0.0, 1.0);
            self.velocity_x *= drag;
            self.velocity_y -= self.tuning.gravity_units_per_sec2 * delta_seconds;
        }

        if self.on_ground && self.jump_requested {
            self.velocity_y = self.tuning.jump_speed_units_per_sec;
            self.on_ground = false;
        }
        self.jump_requested = false;

        self.uniform_scale = if crawling {
            self.tuning.crawl_scale
        } else {
            self.tuning.standing_scale
        };

        let lean_target = (self.velocity_x * self.tuning.lean_factor)
            .clamp(-self.tuning.lean_cap_rad, self.tuning.lean_cap_rad);
        if self.on_ground {
            self.rotation_rad = lean_target;
        } else {
            self.rotation_rad = self.rotation_rad * 0.92 + lean_target * 0.08;
        }
    }

    pub fn reconcile_world(&mut self, world: &mut World, anchor_entity: EntityId) {
        let ground_contact = platformer_ground_contact_y(self.uniform_scale);
        if self.position_y <= ground_contact && self.velocity_y <= 0.0 {
            self.position_y = ground_contact;
            self.velocity_y = 0.0;
            self.on_ground = true;
        } else {
            self.on_ground = false;
        }

        if let Some(transform) = world.transform_mut(anchor_entity) {
            transform.position_x = self.position_x;
            transform.position_y = self.position_y;
            transform.rotation_rad = self.rotation_rad;
            transform.uniform_scale = self.uniform_scale;
            transform.velocity_x = self.velocity_x;
            transform.velocity_y = self.velocity_y;
        }
    }

    pub fn transform_2d(&self) -> SimulationTransform2D {
        SimulationTransform2D {
            position_x: self.position_x,
            position_y: self.position_y,
            rotation_rad: self.rotation_rad,
            uniform_scale: self.uniform_scale,
        }
    }
}
