use anzu_rom_you_touched_my_boats::BoatTouching;
use anzu_runtime::server::LocalServer;
use anzu_runtime::session::{Session, SessionData};
use anzu_runtime::simulation::Simulation;
use log::{debug, error, info};
use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

#[cfg(target_arch = "wasm32")]
use winit::event_loop::EventLoop;

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowAttributesExtWebSys;

use crate::display::WinitDisplayAdapter;
use crate::input::WinitInputAdapter;

pub struct AppEvent;

pub struct App {
    #[cfg(target_arch = "wasm32")]
    _proxy: Option<winit::event_loop::EventLoopProxy<AppEvent>>,
    session: SessionData<
        WinitInputAdapter,
        WinitDisplayAdapter,
        LocalServer<BoatTouching, BoatTouching>,
    >,
    window: Option<Arc<Window>>,
}

impl App {
    pub fn new(#[cfg(target_arch = "wasm32")] event_loop: &EventLoop<AppEvent>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());

        let simulation = BoatTouching::new();
        let server = LocalServer::<BoatTouching, BoatTouching>::new(simulation);
        let input_adapter = WinitInputAdapter {};
        let display_adapter = WinitDisplayAdapter::new();
        let session = SessionData::<
            WinitInputAdapter,
            WinitDisplayAdapter,
            LocalServer<BoatTouching, BoatTouching>,
        >::new(input_adapter, display_adapter, server);

        Self {
            #[cfg(target_arch = "wasm32")]
            _proxy: proxy,
            session: session,
            window: None,
        }
    }
}

impl ApplicationHandler<AppEvent> for App {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        let _ = (event_loop, cause);
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: AppEvent) {
        let _ = (event_loop, event);
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let _ = (event_loop, device_id, event);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }

    fn memory_warning(&mut self, event_loop: &ActiveEventLoop) {
        let _ = event_loop;
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        debug!("Application resumed");

        if self.window.is_some() {
            info!("Window was ready");
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        let window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            let canvas = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id("canvas"))
                .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok());
            window_attributes = window_attributes.with_canvas(canvas);
        }

        let some_window = match event_loop.create_window(window_attributes) {
            Ok(window) => Some(window),
            Err(e) => {
                error!("{:?}", e);
                return ();
            }
        };

        if let Some(window) = some_window {
            let arc_window: Arc<Window> = window.into();
            self.window = Some(arc_window.clone());
            self.session.attach_display_target(arc_window.clone());
        }

        info!("New window created");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        window_event: WindowEvent,
    ) {
        debug!("Window event: {:?}", &window_event);
        match window_event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => self.session.queue_input(event),
            WindowEvent::RedrawRequested => {
                self.session.request_redraw();
            }
            WindowEvent::Resized(size) => {
                if let Some(ref window) = self.window {
                    if window.id() == window_id {
                        self.session.resize_display(size.width, size.height);
                    }
                }
            }
            _ => {}
        }
    }
}
