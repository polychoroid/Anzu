use std::collections::BTreeMap;
use std::num::NonZeroU64;

use wgpu::util::DeviceExt;

use crate::ecs::MaterialId;
use crate::rom::RomPackage;
use crate::simulation::SimulationModel;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BatchVertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
    pub barycentric: [f32; 3],
    pub light_pos: [f32; 2],
    pub edge_mask: [f32; 3],
}

#[derive(Clone, Copy, Debug)]
pub struct DrawBatch {
    pub material_id: MaterialId,
    pub vertex_offset: u32,
    pub vertex_count: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaterialBlendMode {
    Opaque,
    Alpha,
    Additive,
}

#[derive(Clone, Copy, Debug)]
pub struct MaterialDefinition {
    pub material_id: MaterialId,
    pub blend_mode: MaterialBlendMode,
    pub base_color_tint: [f32; 3],
    pub emissive_strength: f32,
    pub metallic: f32,
    pub roughness: f32,
    pub specular_strength: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct MaterialUniform {
    base_color_tint: [f32; 3],
    emissive_strength: f32,
    shading_params: [f32; 4],
}

impl MaterialUniform {
    fn from_definition(definition: MaterialDefinition) -> Self {
        Self {
            base_color_tint: definition.base_color_tint,
            emissive_strength: definition.emissive_strength,
            shading_params: [
                definition.metallic,
                definition.roughness,
                definition.specular_strength,
                0.0,
            ],
        }
    }
}

struct MaterialRegistry {
    defs: BTreeMap<MaterialId, MaterialDefinition>,
    fallback: MaterialDefinition,
}

impl MaterialRegistry {
    fn from_defs(defs: &'static [MaterialDefinition]) -> Self {
        let fallback = MaterialDefinition {
            material_id: MaterialId::default(),
            blend_mode: MaterialBlendMode::Alpha,
            base_color_tint: [1.0, 1.0, 1.0],
            emissive_strength: 0.0,
            metallic: 0.0,
            roughness: 0.8,
            specular_strength: 0.25,
        };
        let mut map = BTreeMap::new();
        for def in defs {
            map.insert(def.material_id, *def);
        }
        Self {
            defs: map,
            fallback,
        }
    }

    fn resolve(&self, material_id: MaterialId) -> MaterialDefinition {
        self.defs
            .get(&material_id)
            .copied()
            .unwrap_or(self.fallback)
    }

    fn fallback_definition(&self) -> MaterialDefinition {
        self.fallback
    }
}

pub struct RomRenderData {
    pub shader_source_wgsl: &'static str,
    pub webgl_shader_source_wgsl: Option<&'static str>,
    pub vertex_layout: wgpu::VertexBufferLayout<'static>,
    pub vertex_bytes: &'static [u8],
    pub vertex_count: u32,
    pub materials: &'static [MaterialDefinition],
}

pub trait RenderRomPackage: RomPackage {
    fn render_data(&self, webgl_compat: bool) -> RomRenderData;
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RotationUniform {
    angle: f32,
    scale: f32,
    translation: [f32; 2],
}

impl RotationUniform {
    pub fn from_state(simulation: &dyn SimulationModel) -> Self {
        let transform = simulation.transform_2d();
        Self {
            angle: transform.rotation_rad,
            scale: transform.uniform_scale,
            translation: [transform.position_x, transform.position_y],
        }
    }

    pub fn identity() -> Self {
        Self {
            angle: 0.0,
            scale: 1.0,
            translation: [0.0, 0.0],
        }
    }
}

pub enum RenderError {
    SurfaceCreation(String),
    AdapterRequest(String),
    DeviceRequest(String),
    UnsupportedSurface(&'static str),
    SurfaceValidation,
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::SurfaceCreation(error) => {
                write!(f, "failed to create surface: {error}")
            }
            RenderError::AdapterRequest(error) => {
                write!(f, "failed to request adapter: {error}")
            }
            RenderError::DeviceRequest(error) => {
                write!(f, "failed to request device: {error}")
            }
            RenderError::UnsupportedSurface(message) => write!(f, "{message}"),
            RenderError::SurfaceValidation => {
                write!(
                    f,
                    "surface validation error while acquiring current texture"
                )
            }
        }
    }
}

pub struct RenderCore {
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    opaque_pipeline: wgpu::RenderPipeline,
    alpha_pipeline: wgpu::RenderPipeline,
    additive_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_buffer_capacity_bytes: u64,
    vertex_count: u32,
    rotation_uniform_buffer: wgpu::Buffer,
    material_uniform_buffer: wgpu::Buffer,
    fallback_material_bind_group: wgpu::BindGroup,
    material_bind_groups: BTreeMap<MaterialId, wgpu::BindGroup>,
    material_registry: MaterialRegistry,
}

impl RenderCore {
    fn create_pipeline(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        vertex_layout: wgpu::VertexBufferLayout<'static>,
        blend_mode: MaterialBlendMode,
    ) -> wgpu::RenderPipeline {
        let blend = match blend_mode {
            MaterialBlendMode::Opaque => None,
            MaterialBlendMode::Alpha => Some(wgpu::BlendState::ALPHA_BLENDING),
            MaterialBlendMode::Additive => Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
        };

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ROM Render Pipeline"),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_layout],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
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
        })
    }

    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        config: wgpu::SurfaceConfiguration,
        simulation: &dyn SimulationModel,
        render_data: RomRenderData,
    ) -> Self {
        let initial_uniform = RotationUniform::from_state(simulation);

        let rotation_uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Model Rotation Uniform Buffer"),
                contents: bytemuck::bytes_of(&initial_uniform),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let rotation_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Model Rotation Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: NonZeroU64::new(
                                std::mem::size_of::<MaterialUniform>() as u64,
                            ),
                        },
                        count: None,
                    },
                ],
            });

        let material_registry = MaterialRegistry::from_defs(render_data.materials);
        let material_uniform_alignment = device.limits().min_uniform_buffer_offset_alignment.max(1);
        let material_uniform_size = std::mem::size_of::<MaterialUniform>() as u32;
        let material_uniform_stride =
            material_uniform_size.div_ceil(material_uniform_alignment) * material_uniform_alignment;
        let material_slot_count = render_data.materials.len() + 1;
        let mut material_uniform_bytes =
            vec![0u8; material_uniform_stride as usize * material_slot_count];

        for (index, definition) in render_data.materials.iter().copied().enumerate() {
            let uniform = MaterialUniform::from_definition(definition);
            let src = bytemuck::bytes_of(&uniform);
            let dst_offset = index * material_uniform_stride as usize;
            let dst = &mut material_uniform_bytes[dst_offset..dst_offset + src.len()];
            dst.copy_from_slice(src);
        }

        let fallback_definition = material_registry.fallback_definition();
        let fallback_uniform = MaterialUniform::from_definition(fallback_definition);
        let fallback_offset = render_data.materials.len() * material_uniform_stride as usize;
        let fallback_src = bytemuck::bytes_of(&fallback_uniform);
        let fallback_dst =
            &mut material_uniform_bytes[fallback_offset..fallback_offset + fallback_src.len()];
        fallback_dst.copy_from_slice(fallback_src);

        let material_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material Uniform Buffer"),
            contents: &material_uniform_bytes,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let create_material_bind_group = |offset: u64| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Model Material Bind Group"),
                layout: &rotation_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: rotation_uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &material_uniform_buffer,
                            offset,
                            size: NonZeroU64::new(std::mem::size_of::<MaterialUniform>() as u64),
                        }),
                    },
                ],
            })
        };

        let mut material_bind_groups = BTreeMap::new();
        for (index, definition) in render_data.materials.iter().enumerate() {
            material_bind_groups.insert(
                definition.material_id,
                create_material_bind_group(index as u64 * material_uniform_stride as u64),
            );
        }

        let fallback_material_bind_group =
            create_material_bind_group(fallback_offset as u64);

        let shader_source = render_data
            .webgl_shader_source_wgsl
            .unwrap_or(render_data.shader_source_wgsl);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ROM Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ROM Pipeline Layout"),
                bind_group_layouts: &[Some(&rotation_bind_group_layout)],
                immediate_size: 0,
            });

        let vertex_layout = render_data.vertex_layout;

        let opaque_pipeline = Self::create_pipeline(
            &device,
            &config,
            &render_pipeline_layout,
            &shader,
            vertex_layout.clone(),
            MaterialBlendMode::Opaque,
        );
        let alpha_pipeline = Self::create_pipeline(
            &device,
            &config,
            &render_pipeline_layout,
            &shader,
            vertex_layout.clone(),
            MaterialBlendMode::Alpha,
        );
        let additive_pipeline = Self::create_pipeline(
            &device,
            &config,
            &render_pipeline_layout,
            &shader,
            vertex_layout,
            MaterialBlendMode::Additive,
        );

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ROM Vertex Buffer"),
            contents: render_data.vertex_bytes,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        let vertex_buffer_capacity_bytes = render_data.vertex_bytes.len() as u64;

        Self {
            device,
            queue,
            config,
            opaque_pipeline,
            alpha_pipeline,
            additive_pipeline,
            vertex_buffer,
            vertex_buffer_capacity_bytes,
            vertex_count: render_data.vertex_count,
            rotation_uniform_buffer,
            material_uniform_buffer,
            fallback_material_bind_group,
            material_bind_groups,
            material_registry,
        }
    }

    pub fn resize(&mut self, surface: &wgpu::Surface<'static>, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        surface.configure(&self.device, &self.config);
    }

    pub fn render(
        &mut self,
        surface: &wgpu::Surface<'static>,
        uniform: RotationUniform,
        vertex_bytes: &[u8],
        draw_batches: &[DrawBatch],
    ) -> Result<(), RenderError> {
        self.queue.write_buffer(
            &self.rotation_uniform_buffer,
            0,
            bytemuck::bytes_of(&uniform),
        );

        if !vertex_bytes.is_empty() {
            let required_capacity = vertex_bytes.len() as u64;
            if required_capacity > self.vertex_buffer_capacity_bytes {
                let grown_capacity = required_capacity.next_power_of_two();
                self.vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("ROM Vertex Buffer (Dynamic Batch)"),
                    size: grown_capacity,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                self.vertex_buffer_capacity_bytes = grown_capacity;
            }
            self.queue
                .write_buffer(&self.vertex_buffer, 0, vertex_bytes);
        }
        self.vertex_count = draw_batches
            .iter()
            .map(|batch| batch.vertex_offset.saturating_add(batch.vertex_count))
            .max()
            .unwrap_or(0);

        let frame = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                surface.configure(&self.device, &self.config);
                frame
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => return Err(RenderError::SurfaceValidation),
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            let draw_mode = |pass: &mut wgpu::RenderPass<'_>, mode: MaterialBlendMode| {
                let pipeline = match mode {
                    MaterialBlendMode::Opaque => &self.opaque_pipeline,
                    MaterialBlendMode::Alpha => &self.alpha_pipeline,
                    MaterialBlendMode::Additive => &self.additive_pipeline,
                };
                pass.set_pipeline(pipeline);

                for batch in draw_batches.iter() {
                    let definition = self.material_registry.resolve(batch.material_id);
                    if definition.blend_mode != mode {
                        continue;
                    }

                    let material_bind_group = self
                        .material_bind_groups
                        .get(&batch.material_id)
                        .unwrap_or(&self.fallback_material_bind_group);
                    pass.set_bind_group(0, material_bind_group, &[]);
                    if batch.vertex_count > 0 {
                        let start = batch.vertex_offset;
                        let end = batch.vertex_offset.saturating_add(batch.vertex_count);
                        pass.draw(start..end, 0..1);
                    }
                }
            };

            draw_mode(&mut pass, MaterialBlendMode::Opaque);
            draw_mode(&mut pass, MaterialBlendMode::Alpha);
            draw_mode(&mut pass, MaterialBlendMode::Additive);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }
}
