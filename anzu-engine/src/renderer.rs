#[cfg(target_arch = "wasm32")]
use std::{f32::consts::TAU, sync::Arc};

#[cfg(target_arch = "wasm32")]
use wgpu::util::DeviceExt;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use winit::window::Window;

#[cfg(target_arch = "wasm32")]
const TRIANGLE_SHADER: &str = r#"
struct RotationUniform {
    angle: f32,
    scale: f32,
    _pad0: vec2<f32>,
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

    var out: VertexOutput;
    out.position = vec4<f32>(rotated, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
"#;

#[cfg(target_arch = "wasm32")]
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

#[cfg(target_arch = "wasm32")]
impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x3];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[cfg(target_arch = "wasm32")]
const TRIANGLE_VERTICES: [Vertex; 3] = [
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

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy)]
struct TriangleSpec {
    angular_velocity_rad_per_sec: f32,
    scale: f32,
}

#[cfg(target_arch = "wasm32")]
impl Default for TriangleSpec {
    fn default() -> Self {
        Self {
            angular_velocity_rad_per_sec: 1.2,
            scale: 0.85,
        }
    }
}

#[cfg(target_arch = "wasm32")]
struct TriangleSimulation {
    angle_rad: f32,
    elapsed_seconds: f32,
}

#[cfg(target_arch = "wasm32")]
impl TriangleSimulation {
    fn new() -> Self {
        Self {
            angle_rad: 0.0,
            elapsed_seconds: 0.0,
        }
    }

    fn update(&mut self, delta_seconds: f32, spec: TriangleSpec) {
        if delta_seconds <= 0.0 {
            return;
        }

        self.elapsed_seconds += delta_seconds;
        self.angle_rad = (self.angle_rad + spec.angular_velocity_rad_per_sec * delta_seconds) % TAU;
    }
}

#[cfg(target_arch = "wasm32")]
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct RotationUniform {
    angle: f32,
    scale: f32,
    _pad0: [f32; 2],
}

#[cfg(target_arch = "wasm32")]
impl RotationUniform {
    fn from_state(simulation: &TriangleSimulation, spec: TriangleSpec) -> Self {
        Self {
            angle: simulation.angle_rad,
            scale: spec.scale,
            _pad0: [0.0, 0.0],
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub struct State {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    triangle_spec: TriangleSpec,
    triangle_simulation: TriangleSimulation,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
    rotation_uniform_buffer: wgpu::Buffer,
    rotation_bind_group: wgpu::BindGroup,
}

#[cfg(target_arch = "wasm32")]
impl State {
    pub async fn new(window: Arc<Window>) -> Result<Self, JsValue> {
        let size = window.inner_size();

        let mut instance_descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        #[cfg(target_arch = "wasm32")]
        {
            instance_descriptor.backends = wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            instance_descriptor.backends = wgpu::Backends::PRIMARY;
        }

        #[cfg(target_arch = "wasm32")]
        let instance = wgpu::util::new_instance_with_webgpu_detection(instance_descriptor).await;

        #[cfg(not(target_arch = "wasm32"))]
        let instance = wgpu::Instance::new(instance_descriptor);

        let surface = instance
            .create_surface(window.clone())
            .map_err(|error| JsValue::from_str(&format!("failed to create surface: {error}")))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|error| JsValue::from_str(&format!("failed to request adapter: {error}")))?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|error| JsValue::from_str(&format!("failed to request device: {error}")))?;

        let surface_caps = surface.get_capabilities(&adapter);

        if surface_caps.formats.is_empty() {
            return Err(JsValue::from_str("surface reports no supported formats"));
        }
        if surface_caps.present_modes.is_empty() {
            return Err(JsValue::from_str("surface reports no present modes"));
        }
        if surface_caps.alpha_modes.is_empty() {
            return Err(JsValue::from_str("surface reports no alpha modes"));
        }

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
            .or_else(|| surface_caps.formats.first().copied())
            .ok_or_else(|| JsValue::from_str("surface format selection failed"))?;

        let present_mode = surface_caps
            .present_modes
            .first()
            .copied()
            .ok_or_else(|| JsValue::from_str("surface present mode selection failed"))?;

        let alpha_mode = surface_caps
            .alpha_modes
            .first()
            .copied()
            .ok_or_else(|| JsValue::from_str("surface alpha mode selection failed"))?;

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

        let triangle_spec = TriangleSpec::default();
        let triangle_simulation = TriangleSimulation::new();
        let initial_uniform = RotationUniform::from_state(&triangle_simulation, triangle_spec);

        let rotation_uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Triangle Rotation Uniform Buffer"),
                contents: bytemuck::bytes_of(&initial_uniform),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let rotation_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Triangle Rotation Bind Group Layout"),
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
            label: Some("Triangle Rotation Bind Group"),
            layout: &rotation_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: rotation_uniform_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Triangle Shader"),
            source: wgpu::ShaderSource::Wgsl(TRIANGLE_SHADER.into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Triangle Pipeline Layout"),
                bind_group_layouts: &[Some(&rotation_bind_group_layout)],
                immediate_size: 0,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Triangle Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
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
            label: Some("Triangle Vertex Buffer"),
            contents: bytemuck::cast_slice(&TRIANGLE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            triangle_spec,
            triangle_simulation,
            render_pipeline,
            vertex_buffer,
            vertex_count: TRIANGLE_VERTICES.len() as u32,
            rotation_uniform_buffer,
            rotation_bind_group,
        })
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn update(&mut self, delta_seconds: f32) {
        self.triangle_simulation
            .update(delta_seconds, self.triangle_spec);
    }

    pub fn render(&mut self) -> Result<(), JsValue> {
        let uniform = RotationUniform::from_state(&self.triangle_simulation, self.triangle_spec);
        self.queue.write_buffer(
            &self.rotation_uniform_buffer,
            0,
            bytemuck::bytes_of(&uniform),
        );

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                self.surface.configure(&self.device, &self.config);
                frame
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(JsValue::from_str(
                    "surface validation error while acquiring current texture",
                ));
            }
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
            pass.draw(0..self.vertex_count, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub struct State;

#[cfg(not(target_arch = "wasm32"))]
impl State {
    pub async fn new(_window: ()) -> Result<Self, wasm_bindgen::JsValue> {
        Ok(Self)
    }

    pub fn window(&self) -> &() {
        &()
    }

    pub fn resize(&mut self, _width: u32, _height: u32) {}

    pub fn update(&mut self, _delta_seconds: f32) {}

    pub fn render(&mut self) -> Result<(), wasm_bindgen::JsValue> {
        Ok(())
    }
}
