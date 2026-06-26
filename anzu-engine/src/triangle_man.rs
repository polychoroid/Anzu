use std::f32::consts::TAU;

use crate::ecs::{
    CollisionBounds, EntityId, Mesh, MeshAssetId, PolygonCollider, RigidBody, Transform, World,
};
use crate::renderer::{RenderRomPackage, RomRenderData};
use crate::rom::RomPackage;
use crate::simulation::{SimulationModel, SimulationTransform2D};

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
        position: [-0.5, -0.5],
        color: [1.0, 0.2, 0.2],
    },
    Vertex {
        position: [0.5, -0.5],
        color: [0.2, 1.0, 0.2],
    },
    Vertex {
        position: [0.0, 0.6],
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

pub const TRIANGLE_COLLIDER: [[f32; 2]; 3] = [[-0.5, -0.5], [0.5, -0.5], [0.0, 0.6]];
pub const SQUARE_COLLIDER: [[f32; 2]; 4] = [[-0.35, -0.35], [0.35, -0.35], [0.35, 0.35], [-0.35, 0.35]];

#[derive(Clone, Copy)]
pub struct TriangleManSpec {
    pub angular_velocity_rad_per_sec: f32,
    pub scale: f32,
}

impl Default for TriangleManSpec {
    fn default() -> Self {
        Self {
            angular_velocity_rad_per_sec: 1.2,
            scale: 0.85,
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
        const MESH_TRIANGLE: MeshAssetId = MeshAssetId(1);
        const MESH_SQUARE: MeshAssetId = MeshAssetId(2);
        const MESH_DIAMOND: MeshAssetId = MeshAssetId(3);

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

        let setup_entity = |world: &mut World,
                            position_x: f32,
                            velocity_x: f32,
                            angular_velocity: f32,
                            mesh_asset: MeshAssetId,
                            collider: &'static [[f32; 2]]|
         -> EntityId {
            let entity_id = world.spawn();
            let scale = 0.7;
            let max_radius = collider
                .iter()
                .map(|vertex| (vertex[0] * vertex[0] + vertex[1] * vertex[1]).sqrt())
                .fold(0.0f32, f32::max)
                * scale;
            let mass = 1.6;
            let moment_of_inertia = 0.5 * mass * max_radius * max_radius;

            world.set_transform(
                entity_id,
                Transform {
                    position_x,
                    position_y: 0.0,
                    rotation_rad: 0.0,
                    uniform_scale: scale,
                    velocity_x,
                    velocity_y: 0.0,
                    angular_velocity,
                    z_depth: 0.0,
                },
            );

            world.set_collision_bounds(
                entity_id,
                CollisionBounds {
                    radius: max_radius,
                    proximity_radius: max_radius * 1.5,
                },
            );

            world.set_polygon_collider(
                entity_id,
                PolygonCollider {
                    local_vertices: collider,
                },
            );

            world.set_rigid_body(
                entity_id,
                RigidBody {
                    mass,
                    inverse_mass: 1.0 / mass,
                    restitution: 0.88,
                    friction: 0.08,
                    moment_of_inertia,
                    inverse_moment_of_inertia: 1.0 / moment_of_inertia,
                },
            );

            world.set_mesh_instance(entity_id, mesh_asset);
            entity_id
        };

        let anchor_entity = world.spawn();
        world.set_transform(
            anchor_entity,
            Transform {
                position_x: -0.9,
                position_y: -0.9,
                rotation_rad: 0.0,
                uniform_scale: 0.1,
                velocity_x: 0.0,
                velocity_y: 0.0,
                angular_velocity: 0.0,
                z_depth: 0.0,
            },
        );

        let _first_entity = setup_entity(world, -0.65, 0.45, 0.8, MESH_TRIANGLE, &TRIANGLE_COLLIDER);
        let _second_entity = setup_entity(world, 0.65, -0.45, -0.8, MESH_SQUARE, &SQUARE_COLLIDER);

        anchor_entity
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
    _anchor_entity: EntityId,
    spec: TriangleManSpec,
    position_x: f32,
    position_y: f32,
    angle_rad: f32,
    elapsed_seconds: f32,
}

impl TriangleManSimulation {
    pub fn new(anchor_entity: EntityId, spec: TriangleManSpec, initial: Transform) -> Self {
        Self {
            _anchor_entity: anchor_entity,
            spec,
            position_x: initial.position_x,
            position_y: initial.position_y,
            angle_rad: initial.rotation_rad,
            elapsed_seconds: 0.0,
        }
    }
}

impl SimulationModel for TriangleManSimulation {
    fn update(&mut self, delta_seconds: f32) {
        if delta_seconds <= 0.0 {
            return;
        }

        self.elapsed_seconds += delta_seconds;
        self.angle_rad =
            (self.angle_rad + self.spec.angular_velocity_rad_per_sec * delta_seconds) % TAU;
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
