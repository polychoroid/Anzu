#[cfg(target_arch = "wasm32")]
mod asset_manifest;
mod renderer;

#[cfg(target_arch = "wasm32")]
use std::sync::Arc;

use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use js_sys::Date;
#[cfg(target_arch = "wasm32")]
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::Window,
};

#[cfg(target_arch = "wasm32")]
use winit::platform::web::{EventLoopExtWebSys, WindowAttributesExtWebSys};

#[cfg(target_arch = "wasm32")]
pub struct App {
    proxy: Option<EventLoopProxy<renderer::State>>,
    state: Option<renderer::State>,
    last_frame_time_ms: Option<f64>,
}

#[cfg(target_arch = "wasm32")]
impl App {
    fn new(event_loop: &EventLoop<renderer::State>) -> Self {
        Self {
            proxy: Some(event_loop.create_proxy()),
            state: None,
            last_frame_time_ms: None,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl ApplicationHandler<renderer::State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let mut window_attributes = Window::default_attributes();

        const CANVAS_ID: &str = "anzu-canvas";

        let Some(browser_window) = wgpu::web_sys::window() else {
            web_sys::console::error_1(&JsValue::from_str("Browser window is unavailable"));
            event_loop.exit();
            return;
        };

        let Some(document) = browser_window.document() else {
            web_sys::console::error_1(&JsValue::from_str("Document is unavailable"));
            event_loop.exit();
            return;
        };

        let Some(canvas) = document.get_element_by_id(CANVAS_ID) else {
            web_sys::console::error_1(&JsValue::from_str("Canvas element #anzu-canvas not found"));
            event_loop.exit();
            return;
        };

        let Ok(html_canvas_element) = canvas.dyn_into::<web_sys::HtmlCanvasElement>() else {
            web_sys::console::error_1(&JsValue::from_str("Canvas element is not an HtmlCanvasElement"));
            event_loop.exit();
            return;
        };

        window_attributes = window_attributes.with_canvas(Some(html_canvas_element));

        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                web_sys::console::error_1(&JsValue::from_str(&format!(
                    "Failed to create window: {error}"
                )));
                event_loop.exit();
                return;
            }
        };

        if let Some(proxy) = self.proxy.take() {
            wasm_bindgen_futures::spawn_local(async move {
                match asset_manifest::load_from_url("manifest.json").await {
                    Ok(registry) => {
                        web_sys::console::log_1(&JsValue::from_str(&format!(
                            "Loaded asset manifest v{} with {} entries (hashed: {}, types: {})",
                            registry.manifest_version(),
                            registry.asset_count(),
                            registry.hashed_asset_count(),
                            registry.asset_type_breakdown()
                        )));
                    }
                    Err(error) => {
                        web_sys::console::warn_1(&error.into_js_value());
                    }
                }

                match renderer::State::new(window).await {
                    Ok(state) => {
                        if proxy.send_event(state).is_err() {
                            web_sys::console::error_1(&JsValue::from_str(
                                "Failed to send renderer state event",
                            ));
                        }
                    }
                    Err(error) => {
                        web_sys::console::error_1(&error);
                    }
                }
            });
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: renderer::State) {
        event.window().request_redraw();
        event.resize(
            event.window().inner_size().width,
            event.window().inner_size().height,
        );
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(state) => state,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                let now_ms = Date::now();
                let delta_seconds = match self.last_frame_time_ms.replace(now_ms) {
                    Some(previous_ms) => ((now_ms - previous_ms) / 1000.0).clamp(0.0, 0.25) as f32,
                    None => 0.0,
                };

                state.update(delta_seconds);

                if let Err(error) = state.render() {
                    web_sys::console::error_1(&error);
                    event_loop.exit();
                    return;
                }

                state.window().request_redraw();
            }
            _ => {}
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn run() -> Result<(), String> {
    let event_loop = EventLoop::with_user_event()
        .build()
        .map_err(|error| error.to_string())?;
    let app = App::new(&event_loop);
    event_loop.spawn_app(app);
    Ok(())
}

/// Start called by wasm-bindgen when the module is initialized in the browser.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn start() -> Result<(), JsValue> {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        run().map_err(|error| JsValue::from_str(&error))?;
    }

    Ok(())
}
