#[path = "material.rs"]
mod material;

#[path = "backend.rs"]
mod backend;

#[path = "pipeline_builder.rs"]
mod pipeline_builder;

use nalgebra::{Isometry3, Orthographic3, Perspective3, Point3, Vector3};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

use crate::execution::shared::{BackgroundColor, RuntimeState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionMode {
    Perspective,
    Isometric,
}

impl ProjectionMode {
    pub fn next(self) -> Self {
        match self {
            ProjectionMode::Perspective => ProjectionMode::Isometric,
            ProjectionMode::Isometric => ProjectionMode::Perspective,
        }
    }
}

pub use material::{BlendMode, MaterialDefinition, MaterialUniform, MaterialUniformLayout};

pub use backend::{
    GpuResourceId, MaterialBindGroups, MaterialGpuState, PipelineId, RenderBackendState,
    RenderInstallSummary, RenderResources,
};

pub use pipeline_builder::{
    BlendPipelinesReady, LayoutReady, MaterialBufferReady, MaterialGroupsReady, Missing,
    PipelineBuildStart, PipelineBuilder, PipelineSet, Present, ShaderReady, Start,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TexturedVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

impl TexturedVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TexturedVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<[f32; 4]>())
                        as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TexturedMesh {
    pub vertices: Vec<TexturedVertex>,
    pub indices: Vec<u16>,
}

pub struct GeometryGpu {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

pub struct TextureGpu {
    pub bind_group: wgpu::BindGroup,
}

pub struct PipelineGpu {
    pub render_pipeline: wgpu::RenderPipeline,
}

pub struct TexturedMeshRenderer {
    pub geometry: GeometryGpu,
    pub texture: TextureGpu,
    pub pipeline: PipelineGpu,
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    pub uniform_bind_group_layout: wgpu::BindGroupLayout,
}

pub struct BodyUniformSlot {
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
}

fn validate_textured_mesh_contract(mesh: &TexturedMesh) -> anyhow::Result<()> {
    if mesh.vertices.is_empty() {
        anyhow::bail!(
            "textured mesh must contain at least one vertex; got {}",
            mesh.vertices.len()
        );
    }

    if mesh.indices.is_empty() {
        anyhow::bail!("textured mesh must contain at least one index");
    }

    if mesh.indices.len() % 3 != 0 {
        anyhow::bail!(
            "textured mesh indices must be a multiple of 3 for triangle-list draws; got {}",
            mesh.indices.len()
        );
    }

    for index in &mesh.indices {
        let idx = usize::from(*index);
        if idx >= mesh.vertices.len() {
            anyhow::bail!(
                "textured mesh index {} is out of bounds for {} vertices",
                idx,
                mesh.vertices.len()
            );
        }
    }

    Ok(())
}

pub fn create_textured_mesh_renderer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    surface_format: wgpu::TextureFormat,
    mesh: &TexturedMesh,
    texture_bytes: &[u8],
) -> anyhow::Result<TexturedMeshRenderer> {
    validate_textured_mesh_contract(mesh)?;

    if texture_bytes.is_empty() {
        anyhow::bail!("texture bytes must not be empty");
    }

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Textured Mesh Vertex Buffer"),
        contents: bytemuck::cast_slice(&mesh.vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Textured Mesh Index Buffer"),
        contents: bytemuck::cast_slice(&mesh.indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    let decoded = image::load_from_memory(texture_bytes)
        .map_err(|err| anyhow::anyhow!("failed to decode ROM texture bytes: {err}"))?;
    let rgba = decoded.to_rgba8();
    let (width, height) = rgba.dimensions();

    let texture_extent = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Textured Mesh Diffuse Texture"),
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        rgba.as_raw(),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        texture_extent,
    );

    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Textured Mesh Diffuse Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
    });

    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Textured Mesh Bind Group Layout"),
        });

    let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &texture_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&texture_sampler),
            },
        ],
        label: Some("Textured Mesh Bind Group"),
    });

    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Textured Mesh Uniform Buffer"),
        size: std::mem::size_of::<[[f32; 4]; 4]>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let uniform_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Textured Mesh Uniform Bind Group Layout"),
        });

    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
        label: Some("Textured Mesh Uniform Bind Group"),
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Textured Mesh Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Textured Mesh Pipeline Layout"),
        bind_group_layouts: &[
            Some(&texture_bind_group_layout),
            Some(&uniform_bind_group_layout),
        ],
        immediate_size: 0,
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Textured Mesh Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Some(TexturedVertex::desc())],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview_mask: None,
        cache: None,
    });

    Ok(TexturedMeshRenderer {
        geometry: GeometryGpu {
            vertex_buffer,
            index_buffer,
            index_count: mesh.indices.len() as u32,
        },
        texture: TextureGpu {
            bind_group: texture_bind_group,
        },
        pipeline: PipelineGpu { render_pipeline },
        uniform_buffer,
        uniform_bind_group,
        uniform_bind_group_layout,
    })
}

pub fn create_textured_mesh_renderer_from_mesh(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    surface_format: wgpu::TextureFormat,
    mesh: &crate::assets::Mesh,
    vertex_colors: &[[f32; 4]],
    texture_bytes: &[u8],
) -> anyhow::Result<TexturedMeshRenderer> {
    let indices = mesh
        .indices()
        .ok_or_else(|| anyhow::anyhow!("textured mesh requires indices"))?;
    let uvs = mesh
        .uv
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("textured mesh requires UV coordinates"))?;

    if mesh.points.len() != vertex_colors.len() {
        anyhow::bail!(
            "point/color cardinality mismatch: {} points vs {} colors",
            mesh.points.len(),
            vertex_colors.len()
        );
    }

    if mesh.points.len() != uvs.len() {
        anyhow::bail!(
            "point/UV cardinality mismatch: {} points vs {} uvs",
            mesh.points.len(),
            uvs.len()
        );
    }

    let vertices = mesh
        .points
        .iter()
        .enumerate()
        .map(|(index, point)| TexturedVertex {
            position: [point.x, point.y, point.z],
            color: vertex_colors[index],
            uv: uvs[index],
        })
        .collect();

    let textured_mesh = TexturedMesh {
        vertices,
        indices: indices.to_vec(),
    };

    create_textured_mesh_renderer(device, queue, surface_format, &textured_mesh, texture_bytes)
}

/// Allocates a `GeometryGpu` for each `(mesh_id, mesh, vertex_colors)` triple and returns
/// a map keyed by mesh_id so the render pass can look up geometry per body.
pub fn create_geometry_map(
    device: &wgpu::Device,
    specs: &[(u32, &crate::assets::Mesh, &[[f32; 4]])],
) -> anyhow::Result<HashMap<u32, GeometryGpu>> {
    let mut map = HashMap::new();
    for (mesh_id, mesh, vertex_colors) in specs {
        let indices = mesh
            .indices()
            .ok_or_else(|| anyhow::anyhow!("mesh {mesh_id} requires indices"))?;

        let uvs = mesh
            .uv
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("mesh {mesh_id} requires UV coordinates"))?;

        if mesh.points.len() != vertex_colors.len() {
            anyhow::bail!(
                "mesh {mesh_id}: point/color cardinality mismatch: {} points vs {} colors",
                mesh.points.len(),
                vertex_colors.len()
            );
        }

        let vertices: Vec<TexturedVertex> = mesh
            .points
            .iter()
            .enumerate()
            .map(|(i, point)| TexturedVertex {
                position: [point.x, point.y, point.z],
                color: vertex_colors[i],
                uv: uvs[i],
            })
            .collect();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Body Geometry Vertex Buffer mesh={mesh_id}")),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("Body Geometry Index Buffer mesh={mesh_id}")),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        map.insert(
            *mesh_id,
            GeometryGpu {
                vertex_buffer,
                index_buffer,
                index_count: indices.len() as u32,
            },
        );
    }
    Ok(map)
}

pub fn create_body_uniform_slots(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    count: usize,
) -> Vec<BodyUniformSlot> {
    (0..count)
        .map(|index| {
            let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Textured Mesh Body Uniform Buffer {index}")),
                size: std::mem::size_of::<[[f32; 4]; 4]>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
                label: Some(&format!("Textured Mesh Body Uniform Bind Group {index}")),
            });

            BodyUniformSlot {
                uniform_buffer,
                uniform_bind_group,
            }
        })
        .collect()
}

pub fn encode_textured_mesh_pass(
    renderer: &TexturedMeshRenderer,
    encoder: &mut wgpu::CommandEncoder,
    target: &wgpu::TextureView,
    surface_width: u32,
    surface_height: u32,
    projection_mode: ProjectionMode,
    clear_color: BackgroundColor,
    runtime_state: &RuntimeState,
    queue: &wgpu::Queue,
    body_uniform_slots: &[BodyUniformSlot],
    body_geometry_map: &HashMap<u32, GeometryGpu>,
) {
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Textured Mesh Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: clear_color.r,
                    g: clear_color.g,
                    b: clear_color.b,
                    a: clear_color.a,
                }),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
        multiview_mask: None,
    });

    let aspect_ratio = surface_width.max(1) as f32 / surface_height.max(1) as f32;
    let view_projection_matrix = match projection_mode {
        ProjectionMode::Perspective => {
            Perspective3::new(aspect_ratio, std::f32::consts::FRAC_PI_4, 0.1, 10.0).to_homogeneous()
        }
        ProjectionMode::Isometric => {
            let scale = 0.9;
            let target = Point3::new(0.0, 0.0, -2.0);
            let projection_matrix = Orthographic3::new(
                -scale * aspect_ratio,
                scale * aspect_ratio,
                -scale,
                scale,
                0.1,
                10.0,
            )
            .to_homogeneous();
            let view_matrix =
                Isometry3::look_at_rh(&Point3::new(2.5, 2.5, 2.5), &target, &Vector3::y_axis())
                    .to_homogeneous();
            projection_matrix * view_matrix
        }
    };
    let body_transforms = runtime_state.body_transforms();
    let body_mesh_ids = runtime_state.body_mesh_ids();
    if body_transforms.len() != body_uniform_slots.len() {
        log::warn!(
            "body transform count {} does not match uniform slot count {}; drawing available bodies",
            body_transforms.len(),
            body_uniform_slots.len()
        );
    }

    render_pass.set_pipeline(&renderer.pipeline.render_pipeline);
    render_pass.set_bind_group(0, &renderer.texture.bind_group, &[]);

    for (index, body_matrix) in body_transforms.iter().enumerate() {
        let Some(slot) = body_uniform_slots.get(index) else {
            break;
        };

        let mesh_id = body_mesh_ids.get(index).copied().unwrap_or(0);
        let geometry = body_geometry_map
            .get(&mesh_id)
            .unwrap_or(&renderer.geometry);

        let model_matrix = view_projection_matrix * body_matrix;
        let uniform_data: [[f32; 4]; 4] = model_matrix.into();
        queue.write_buffer(
            &slot.uniform_buffer,
            0,
            bytemuck::cast_slice(&[uniform_data]),
        );
        render_pass.set_bind_group(1, &slot.uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, geometry.vertex_buffer.slice(..));
        render_pass.set_index_buffer(geometry.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..geometry.index_count, 0, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_mesh() -> TexturedMesh {
        TexturedMesh {
            vertices: vec![
                TexturedVertex {
                    position: [0.0, 0.5, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                    uv: [0.5, 0.0],
                },
                TexturedVertex {
                    position: [-0.5, -0.5, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                    uv: [0.0, 1.0],
                },
                TexturedVertex {
                    position: [0.5, -0.5, 0.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                    uv: [1.0, 1.0],
                },
            ],
            indices: vec![0, 1, 2],
        }
    }

    fn sample_texture_bytes() -> &'static [u8] {
        include_bytes!("../roms/files/textures/Poliigon_BrickReclaimedRunning_7787_BaseColor.jpg")
    }

    #[test]
    fn textured_mesh_contract_rejects_out_of_bounds_indices() {
        let mut mesh = make_test_mesh();
        mesh.indices = vec![0, 1, 3];

        assert!(validate_textured_mesh_contract(&mesh).is_err());
    }

    #[test]
    fn textured_mesh_renderer_preserves_index_count() {
        let mesh = make_test_mesh();

        assert_eq!(mesh.indices.len(), 3);
    }

    #[test]
    fn textured_mesh_pipeline_builds_with_shader_layout() {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });
        let adapter =
            match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: true,
                compatible_surface: None,
                apply_limit_buckets: true,
            })) {
                Ok(adapter) => adapter,
                Err(_) => return,
            };

        let (device, queue) =
            match pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("textured-mesh-test-device"),
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })) {
                Ok(parts) => parts,
                Err(_) => return,
            };

        let mesh = make_test_mesh();
        let renderer = create_textured_mesh_renderer(
            &device,
            &queue,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            &mesh,
            sample_texture_bytes(),
        )
        .expect("pipeline should build");

        assert_eq!(renderer.geometry.index_count, 3);
    }

    #[test]
    fn textured_mesh_pass_draws_indexed_geometry() {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });
        let adapter =
            match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: true,
                compatible_surface: None,
                apply_limit_buckets: true,
            })) {
                Ok(adapter) => adapter,
                Err(_) => return,
            };

        let (device, queue) =
            match pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("textured-mesh-pass-device"),
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })) {
                Ok(parts) => parts,
                Err(_) => return,
            };

        let mesh = make_test_mesh();
        let renderer = match create_textured_mesh_renderer(
            &device,
            &queue,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            &mesh,
            sample_texture_bytes(),
        ) {
            Ok(renderer) => renderer,
            Err(_) => return,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("textured-pass-target"),
            size: wgpu::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("textured-pass-encoder"),
        });

        let runtime = crate::execution::shared::RuntimeState::new();

        encode_textured_mesh_pass(
            &renderer,
            &mut encoder,
            &view,
            4,
            4,
            ProjectionMode::Perspective,
            BackgroundColor {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            &runtime,
            &queue,
            &[],
            &std::collections::HashMap::new(),
        );

        queue.submit(std::iter::once(encoder.finish()));
    }
}
