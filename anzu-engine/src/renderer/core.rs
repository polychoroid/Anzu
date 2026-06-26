use wgpu::util::DeviceExt;

use crate::rom::RomPackage;
use crate::simulation::SimulationModel;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BatchVertex {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

pub struct RomRenderData {
    pub shader_source_wgsl: &'static str,
    pub vertex_layout: wgpu::VertexBufferLayout<'static>,
    pub vertex_bytes: &'static [u8],
    pub vertex_count: u32,
}

pub trait RenderRomPackage: RomPackage {
    fn render_data(&self) -> RomRenderData;
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
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    rotation_uniform_buffer: wgpu::Buffer,
    rotation_bind_group: wgpu::BindGroup,
}

impl RenderCore {
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
            });

        let rotation_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Model Rotation Bind Group"),
            layout: &rotation_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: rotation_uniform_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ROM Shader"),
            source: wgpu::ShaderSource::Wgsl(render_data.shader_source_wgsl.into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ROM Pipeline Layout"),
                bind_group_layouts: &[Some(&rotation_bind_group_layout)],
                immediate_size: 0,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ROM Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[render_data.vertex_layout],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
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
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ROM Vertex Buffer"),
            contents: render_data.vertex_bytes,
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            vertex_count: render_data.vertex_count,
            rotation_uniform_buffer,
            rotation_bind_group,
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
        vertex_count: u32,
    ) -> Result<(), RenderError> {
        self.queue.write_buffer(
            &self.rotation_uniform_buffer,
            0,
            bytemuck::bytes_of(&uniform),
        );

        if !vertex_bytes.is_empty() {
            self.vertex_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("ROM Vertex Buffer (Dynamic Batch)"),
                    contents: vertex_bytes,
                    usage: wgpu::BufferUsages::VERTEX,
                });
        }
        self.vertex_count = vertex_count;

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
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
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

            pass.set_pipeline(&self.render_pipeline);
            pass.set_bind_group(0, &self.rotation_bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            if self.vertex_count > 0 {
                pass.draw(0..self.vertex_count, 0..1);
            }
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }
}
