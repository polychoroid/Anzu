use log::{LevelFilter, info};
use std::error::Error;
use winit::event_loop::EventLoop;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::UnwrapThrowExt;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

#[cfg(not(target_arch = "wasm32"))]
use gloo_timers::callback::Interval;

mod app;
mod display;
mod input;
use crate::app::{App, AppEvent};

pub fn run() -> Result<(), Box<dyn Error>> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut builder = env_logger::Builder::from_default_env();
        builder.filter(None, LevelFilter::Info).init();
        info!("Logger initialized");
    }

    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
        info!("Logger initialized");
    }

    let event_loop = EventLoop::with_user_event().build()?;
    let proxy = event_loop.create_proxy();

    #[cfg(not(target_arch = "wasm32"))]
    {
        let interval = Interval::new(16, move || {
            let _ = proxy.send_event(AppEvent::Tick);
        });
        let mut app = App::new(interval);
        event_loop.run_app(&mut app)?;
        info!("Starting app natively.");
    }

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::closure::Closure;
        let callback = Closure::wrap(Box::new(move || {
            let _ = proxy.send_event(AppEvent::Tick);
        }) as Box<dyn FnMut()>);

        web_sys::window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                16,
            )
            .unwrap();

        callback.forget();

        let app = App::new();
        event_loop.spawn_app(app);
        info!("Starting app in WASM32");
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    run().unwrap_throw();

    Ok(())
}
