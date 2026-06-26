use std::sync::Arc;

use js_sys::Date;
use wasm_bindgen::{JsCast, JsValue};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    platform::web::{EventLoopExtWebSys, WindowAttributesExtWebSys},
    window::Window,
};

use crate::{asset_manifest, renderer};

const FRAME_LOG_INTERVAL: u64 = 120;

fn log_info(message: &str) {
    web_sys::console::log_1(&JsValue::from_str(message));
}

fn log_warn(message: &str) {
    web_sys::console::warn_1(&JsValue::from_str(message));
}

fn log_error(message: &str) {
    web_sys::console::error_1(&JsValue::from_str(message));
}

pub struct App {
    proxy: Option<EventLoopProxy<renderer::State>>,
    state: Option<renderer::State>,
    last_frame_time_ms: Option<f64>,
    frame_count: u64,
    frame_delta_accumulator_seconds: f64,
    render_error_count: u64,
    manifest_loader: Arc<dyn asset_manifest::AssetManifestLoader>,
    rom_resolver: Arc<dyn Fn(&str) -> Option<Box<dyn renderer::RenderRomPackage>>>,
}

impl App {
    fn new(
        event_loop: &EventLoop<renderer::State>,
        manifest_loader: Arc<dyn asset_manifest::AssetManifestLoader>,
        rom_resolver: Arc<dyn Fn(&str) -> Option<Box<dyn renderer::RenderRomPackage>>>,
    ) -> Self {
        Self {
            proxy: Some(event_loop.create_proxy()),
            state: None,
            last_frame_time_ms: None,
            frame_count: 0,
            frame_delta_accumulator_seconds: 0.0,
            render_error_count: 0,
            manifest_loader,
            rom_resolver,
        }
    }
}

impl ApplicationHandler<renderer::State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log_info("[APP] resumed: starting browser bootstrap");
        let mut window_attributes = Window::default_attributes();

        const CANVAS_ID: &str = "anzu-canvas";

        let Some(browser_window) = wgpu::web_sys::window() else {
            log_error("[APP] bootstrap failed: browser window is unavailable");
            event_loop.exit();
            return;
        };

        let Some(document) = browser_window.document() else {
            log_error("[APP] bootstrap failed: document is unavailable");
            event_loop.exit();
            return;
        };

        let Some(canvas) = document.get_element_by_id(CANVAS_ID) else {
            log_error("[APP] bootstrap failed: canvas element #anzu-canvas not found");
            event_loop.exit();
            return;
        };

        let Ok(html_canvas_element) = canvas.dyn_into::<web_sys::HtmlCanvasElement>() else {
            log_error("[APP] bootstrap failed: canvas element is not an HtmlCanvasElement");
            event_loop.exit();
            return;
        };

        log_info("[APP] canvas resolved and attached to window attributes");

        window_attributes = window_attributes.with_canvas(Some(html_canvas_element));

        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                log_error(&format!(
                    "[APP] bootstrap failed: window creation error: {error}"
                ));
                event_loop.exit();
                return;
            }
        };

        let size = window.inner_size();
        log_info(&format!(
            "[APP] window created: initial_size={}x{}",
            size.width, size.height
        ));

        if let Some(proxy) = self.proxy.take() {
            let manifest_loader = Arc::clone(&self.manifest_loader);
            let rom_resolver = Arc::clone(&self.rom_resolver);
            log_info("[APP] spawning async runtime/bootstrap task");
            wasm_bindgen_futures::spawn_local(async move {
                log_info("[MANIFEST] loading manifest from 'manifest.json'");
                let manifest = match manifest_loader.load_manifest("manifest.json").await {
                    Ok(manifest) => manifest,
                    Err(error) => {
                        log_error(&format!("[MANIFEST] load failed: {}", error));
                        return;
                    }
                };

                let Some(manifest_rom_id) = manifest.rom_id() else {
                    log_error("[MANIFEST] missing required field: rom_id");
                    return;
                };
                log_info(&format!(
                    "[MANIFEST] resolved rom_id='{}' version={} assets={}",
                    manifest_rom_id,
                    manifest.manifest_version(),
                    manifest.asset_count()
                ));

                let physics_config = manifest.physics_config();

                let Some(rom) = rom_resolver(manifest_rom_id) else {
                    log_error(&format!(
                        "[ROM] unresolved rom_id='{}' from manifest",
                        manifest_rom_id,
                    ));
                    return;
                };
                log_info(&format!(
                    "[ROM] instantiated rom package id='{}'",
                    rom.rom_id()
                ));
                match renderer::State::new(window, rom, physics_config).await {
                    Ok(state) => {
                        log_info("[APP] renderer state created and dispatching to event loop");
                        if proxy.send_event(state).is_err() {
                            log_error("[APP] failed to send renderer state event");
                        }
                    }
                    Err(error) => {
                        log_error(&format!(
                            "[APP] renderer state initialization failed: {:?}",
                            error
                        ));
                    }
                }
            });
        } else {
            log_warn("[APP] resumed with missing event-loop proxy; bootstrap task was not started");
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: renderer::State) {
        log_info("[APP] user event received: renderer state installed");
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
            WindowEvent::CloseRequested => {
                log_info("[APP] close requested: exiting event loop");
                event_loop.exit()
            }
            WindowEvent::Resized(size) => {
                log_info(&format!(
                    "[RENDER] resize event: width={} height={}",
                    size.width, size.height
                ));
                state.resize(size.width, size.height)
            }
            WindowEvent::RedrawRequested => {
                let now_ms = Date::now();
                let delta_seconds = match self.last_frame_time_ms.replace(now_ms) {
                    Some(previous_ms) => ((now_ms - previous_ms) / 1000.0).clamp(0.0, 0.25) as f32,
                    None => 0.0,
                };

                self.frame_count = self.frame_count.saturating_add(1);
                self.frame_delta_accumulator_seconds += f64::from(delta_seconds);

                state.update(delta_seconds);

                if let Err(error) = state.render() {
                    self.render_error_count = self.render_error_count.saturating_add(1);
                    log_error(&format!(
                        "[FRAME] render failure at frame={} error={:?}",
                        self.frame_count, error
                    ));
                    event_loop.exit();
                    return;
                }

                if self.frame_count % FRAME_LOG_INTERVAL == 0 {
                    let avg_dt_seconds =
                        self.frame_delta_accumulator_seconds / FRAME_LOG_INTERVAL as f64;
                    let fps = if avg_dt_seconds > 0.0 {
                        1.0 / avg_dt_seconds
                    } else {
                        0.0
                    };
                    log_info(&format!(
                        "[FRAME] summary frames={} avg_dt_ms={:.3} fps={:.2} render_errors={}",
                        FRAME_LOG_INTERVAL,
                        avg_dt_seconds * 1000.0,
                        fps,
                        self.render_error_count
                    ));
                    self.frame_delta_accumulator_seconds = 0.0;
                }

                state.window().request_redraw();
            }
            _ => {}
        }
    }
}

pub fn run(
    rom_resolver: Arc<dyn Fn(&str) -> Option<Box<dyn renderer::RenderRomPackage>>>,
) -> Result<(), String> {
    log_info("[APP] run invoked: creating event loop");
    let event_loop = EventLoop::with_user_event()
        .build()
        .map_err(|error| error.to_string())?;
    let manifest_loader: Arc<dyn asset_manifest::AssetManifestLoader> =
        Arc::new(asset_manifest::WebAssetManifestLoader);
    let app = App::new(&event_loop, manifest_loader, rom_resolver);
    log_info("[APP] event loop created: spawning application handler");
    event_loop.spawn_app(app);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::run;

    type TraitResolver = Arc<dyn Fn(&str) -> Option<Box<dyn crate::renderer::RenderRomPackage>>>;
    type PlatformRunSignature = fn(TraitResolver) -> Result<(), String>;

    #[test]
    fn platform_run_signature_is_trait_bound_only() {
        // Compile-time boundary check: platform run API is expressed only in trait terms.
        let run_fn: PlatformRunSignature = run;
        let _ = run_fn;
    }
}
