use anzu_runtime::assets::{AssetHandle, AssetId, DrawAssetInstance};
use anzu_runtime::display::{RenderBackend, SupportedRenderBackends};
use log::{debug, info};
use std::collections::HashMap;
use std::io::Error;
use wgpu::{
    Backends, BindGroup, BindGroupEntry, BlendState, Buffer, BufferUsages, ColorTargetState,
    ColorWrites, Device, DeviceDescriptor, ExperimentalFeatures, Face, Features, FragmentState,
    FrontFace, IndexFormat, Instance, InstanceDescriptor, Limits, MultisampleState,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PolygonMode, PowerPreference,
    PresentMode, PrimitiveState, PrimitiveTopology, Queue, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource, Surface,
    SurfaceCapabilities, SurfaceColorSpace, SurfaceConfiguration, SurfaceTarget, TextureUsages,
    Trace, VertexState,
};

use crate::loader::{Asset, Material, Primitive, Uploaded};

pub struct WgpuRenderBackend<'window> {
    surface: Surface<'window>,
    device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
    configuration: SurfaceConfiguration,
    transform_buffer: Buffer,
    transform_bind_group: BindGroup,
    assets: HashMap<AssetId, Asset<Uploaded>>,
}

impl<'window> WgpuRenderBackend<'window> {
    pub async fn new(
        target: impl Into<SurfaceTarget<'static>>,
        selected_backends: SupportedRenderBackends,
        width: u32,
        height: u32,
    ) -> Result<Self, ()> {
        let instance = Instance::new(InstanceDescriptor {
            backends: selected_backends.get_backends(),
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        debug!(
            "Enabled backends: {:?}",
            Instance::enabled_backend_features()
        );

        let surface = instance.create_surface(target).unwrap();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: true,
            })
            .await
            .unwrap();

        let active_backend = &adapter.get_info().backend;

        debug!("Active backend: {:?}", active_backend);

        let surface_capabilities = surface.get_capabilities(&adapter);

        debug!("Surface capabilities: {:?}", &surface_capabilities);

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                experimental_features: ExperimentalFeatures::disabled(),
                required_limits: selected_backends.get_limits(),
                memory_hints: Default::default(),
                trace: Trace::Off,
            })
            .await
            .unwrap();

        let configuration =
            selected_backends.get_surface_configurtion(surface_capabilities, (width, height));

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Naive Shader"),
            source: ShaderSource::Wgsl(include_str!("./shaders/shader.wgsl").into()),
        });

        let transform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Matrix Transform Buffer"),
            size: std::mem::size_of::<[[f32; 4]; 4]>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let transform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Matrix Transform Bind Group Layout"),
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

        let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Matrix Transform Bind Group"),
            layout: &transform_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            }],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Derp what is layout"),
            bind_group_layouts: &[Some(&transform_layout), Some(&Material::layout(&device))],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Naive Ass Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(Primitive::desc())], // Later inctance and material buffers.
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: configuration.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        Ok(Self {
            surface: surface,
            device: device,
            queue: queue,
            render_pipeline: render_pipeline,
            configuration: configuration,
            transform_buffer: transform_buffer,
            transform_bind_group: transform_bind_group,
            assets: HashMap::new(),
        })
    }
}

impl<'window> RenderBackend for WgpuRenderBackend<'window> {
    fn load_assets(&mut self, assets: Vec<AssetHandle>) -> Result<(), Error> {
        info!("Loading assets");

        assets
            .iter()
            .try_for_each(|asset: &AssetHandle| -> Result<(), Error> {
                let uploaded_asset = crate::loader::load_model(asset.bytes, &self.device)?;
                info!("{:?} loaded", asset.id);
                self.assets.insert(asset.id, uploaded_asset);
                Ok(())
            })?;

        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.configuration.width = width;
        self.configuration.height = height;
        self.surface.configure(&self.device, &self.configuration);
    }

    fn render(&mut self, instances: Vec<DrawAssetInstance>) -> Result<(), Error> {
        // This is bad, this is very bad.
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.configuration);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => return Ok(()),
            wgpu::CurrentSurfaceTexture::Validation => return Ok(()),
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        if let Some(instance) = instances.first() {
            self.queue.write_buffer(
                &self.transform_buffer,
                0,
                bytemuck::bytes_of(&instance.transform),
            );
        }

        // todo: should be iterating all instances.
        let asset_id = &instances[2].asset_id;
        if let Some(ref asset) = self.assets.get(asset_id) {
            {
                info!("Render requested for asset: {:?}", asset_id);

                let primitive = &asset.state.meshes[0].primitives[0]; // Again shouldn't be selecting by index
                let material_index = primitive.material.unwrap();

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });

                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_vertex_buffer(0, primitive.vertex_buffer.slice(..));
                render_pass.set_index_buffer(primitive.index_buffer.slice(..), IndexFormat::Uint32);
                render_pass.set_bind_group(0, &self.transform_bind_group, &[]);
                render_pass.set_bind_group(
                    1,
                    &asset.state.materials[material_index].bind_group,
                    &[],
                );
                render_pass.draw_indexed(0..primitive.index_count, 0, 0..1);
            }
        } else {
            info!("Render requested to no loaded mesh");
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(output);

        Ok(())
    }
}

trait BackendConfiguration {
    fn get_backends(&self) -> Backends;
    fn get_limits(&self) -> Limits;
    fn get_surface_configurtion(
        &self,
        capabilities: SurfaceCapabilities,
        size: (u32, u32),
    ) -> SurfaceConfiguration;
}

impl BackendConfiguration for SupportedRenderBackends {
    fn get_backends(&self) -> Backends {
        match self {
            SupportedRenderBackends::WebGpu => Backends::BROWSER_WEBGPU,
            SupportedRenderBackends::WebGl => Backends::GL,
            SupportedRenderBackends::Native => Backends::PRIMARY,
        }
    }

    fn get_limits(&self) -> Limits {
        match self {
            SupportedRenderBackends::WebGl => Limits::downlevel_webgl2_defaults(),
            _ => Limits::default(),
        }
    }

    fn get_surface_configurtion(
        &self,
        capabilities: SurfaceCapabilities,
        size: (u32, u32),
    ) -> SurfaceConfiguration {
        let present_mode: PresentMode =
            if capabilities.present_modes.contains(&PresentMode::Mailbox) {
                PresentMode::Mailbox
            } else {
                PresentMode::Fifo
            };

        debug!("Present mode: {:?}", &present_mode);

        let surface_format = capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(capabilities.formats[0]);

        SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0.max(1),
            height: size.1.max(1),
            present_mode: present_mode,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: { 2 },
            color_space: SurfaceColorSpace::Auto,
        }
    }
}
