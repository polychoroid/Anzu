use anzu_render_wgpu::render::WgpuRenderBackend;
use anzu_runtime::display::{DisplayAdapter, DrawBatch, RenderBackend, SupportedRenderBackends};
use log::info;
#[cfg(target_arch = "wasm32")]
use log::{debug, error};
use std::io::Error;
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};
use winit::window::Window;

type Target = Arc<Window>;
type Backend = WgpuRenderBackend<'static>;

pub struct WinitDisplayAdapter {
    target: Option<Target>,
    backend: Option<Backend>,
    #[cfg(target_arch = "wasm32")]
    size: (u32, u32),
    #[cfg(target_arch = "wasm32")]
    pending: Rc<RefCell<Option<Backend>>>,
}

impl<'a> WinitDisplayAdapter {
    pub fn new() -> Self {
        Self {
            target: None,
            backend: None,
            #[cfg(target_arch = "wasm32")]
            size: (1, 1),
            #[cfg(target_arch = "wasm32")]
            pending: Rc::new(RefCell::new(None)),
        }
    }

    fn connect_backend(&mut self, target: Target) {
        let size = target.inner_size();
        let backends = Self::backend_options();

        info!("Selected backends: {:?}", &backends);
        #[cfg(target_arch = "wasm32")]
        {
            let slot = self.pending.clone();
            let window = target.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match WgpuRenderBackend::new(window.clone(), backends, size.width, size.height)
                    .await
                {
                    Ok(backend) => {
                        *slot.borrow_mut() = Some(backend);
                        window.request_redraw();
                    }
                    Err(e) => {
                        error!("{:?}", e);
                    }
                }
            })
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let backend = pollster::block_on(WgpuRenderBackend::new(
                target,
                backends,
                size.width,
                size.height,
            ))
            .unwrap();
            self.backend = Some(backend);
        }
    }

    fn backend_options() -> SupportedRenderBackends {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            let context = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.create_element("canvas").ok())
                .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
                .and_then(|c| c.get_context("webgpu").ok().flatten())
                .is_some();

            debug!("Canvas context: {:?}", &context);

            if context {
                SupportedRenderBackends::WebGpu
            } else {
                SupportedRenderBackends::WebGl
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        SupportedRenderBackends::Native
    }

    #[cfg(target_arch = "wasm32")]
    fn adopt_pending(&mut self) {
        if self.backend.is_none() {
            if let Some(mut backend) = self.pending.borrow_mut().take() {
                backend.resize(self.size.0, self.size.1);
                self.backend = Some(backend);
            }
        }
    }
}

impl<'a> DisplayAdapter for WinitDisplayAdapter {
    type Target = Target;
    type Backend = Backend;

    fn resize(&mut self, width: u32, height: u32) {
        #[cfg(target_arch = "wasm32")]
        self.adopt_pending();

        match self.backend.as_mut() {
            Some(backend) => backend.resize(width, height),
            None => (),
        };
    }

    fn render(&mut self, data: Vec<DrawBatch>) -> Result<(), Error> {
        #[cfg(target_arch = "wasm32")]
        self.adopt_pending();

        match self.backend.as_mut() {
            Some(backend) => backend.render(data),
            None => Ok(()),
        }
    }

    fn connect_target(&mut self, target: Self::Target) {
        if self.target.is_some() || self.backend.is_some() {
            return;
        }

        self.target = Some(target.clone());
        self.connect_backend(target.clone());
    }
}
