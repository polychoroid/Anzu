use anzu_runtime::display::{DrawBatch, RenderBackend, SupportedRenderBackends};
use log::info;
use std::io::Error;
use wgpu::{
    Backend, Backends, Device, DeviceDescriptor, ExperimentalFeatures, Features, Instance,
    InstanceDescriptor, Limits, PowerPreference, PresentMode, Queue, RequestAdapterOptions,
    Surface, SurfaceCapabilities, SurfaceColorSpace, SurfaceConfiguration, SurfaceTarget,
    TextureUsages, Trace,
};

pub struct WgpuRenderBackend<'window> {
    device: Device,
    queue: Queue,
    surface: Surface<'window>,
    configuration: SurfaceConfiguration,
}

impl<'window> WgpuRenderBackend<'window> {
    pub async fn new(
        target: impl Into<SurfaceTarget<'static>>,
        selected_backends: SupportedRenderBackends,
        width: u32,
        height: u32,
    ) -> Result<Self, ()> {
        let instance = Instance::new(InstanceDescriptor {
            backends: Self::select_backend(selected_backends),
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        info!(
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

        info!("Active backend: {:?}", active_backend);

        let surface_capabilities = surface.get_capabilities(&adapter);

        info!("Surface capabilities: {:?}", &surface_capabilities);

        let limts = if active_backend == &Backend::Gl {
            Limits::downlevel_webgl2_defaults()
        } else {
            Limits::defaults()
        };

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                experimental_features: ExperimentalFeatures::disabled(),
                required_limits: limts,
                memory_hints: Default::default(),
                trace: Trace::Off,
            })
            .await
            .unwrap();

        let configuration = Self::select_surface_configuration(
            surface_capabilities,
            active_backend,
            (width, height),
        );

        Ok(Self {
            surface: surface,
            device: device,
            queue: queue,
            configuration: configuration,
        })
    }

    fn select_surface_configuration(
        capabilities: SurfaceCapabilities,
        backend: &Backend,
        size: (u32, u32),
    ) -> SurfaceConfiguration {
        let present_mode: PresentMode =
            if capabilities.present_modes.contains(&PresentMode::Mailbox) {
                PresentMode::Mailbox
            } else {
                PresentMode::Fifo
            };

        info!("Present mode: {:?}", &present_mode);

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
            desired_maximum_frame_latency: if backend == &Backend::Gl { 1 } else { 2 },
            color_space: SurfaceColorSpace::Auto,
        }
    }

    fn select_backend(selected_backends: SupportedRenderBackends) -> Backends {
        match selected_backends {
            SupportedRenderBackends::WebGpu => Backends::BROWSER_WEBGPU,
            SupportedRenderBackends::WebGl => Backends::GL,
            SupportedRenderBackends::Native => Backends::PRIMARY,
        }
    }
}

impl<'window> RenderBackend for WgpuRenderBackend<'window> {
    fn resize(&mut self, width: u32, height: u32) {
        self.configuration.width = width;
        self.configuration.height = height;
        self.surface.configure(&self.device, &self.configuration);
    }

    fn render(&mut self, _data: Vec<DrawBatch>) -> Result<(), Error> {
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

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
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
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(output);

        Ok(())
    }
}
