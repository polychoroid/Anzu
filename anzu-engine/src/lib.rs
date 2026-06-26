#[cfg(target_arch = "wasm32")]
mod asset_manifest;
#[cfg(target_arch = "wasm32")]
mod ecs;
mod input;
#[cfg(target_arch = "wasm32")]
mod platform_browser;
mod renderer;
#[cfg(target_arch = "wasm32")]
mod rom;
#[cfg(target_arch = "wasm32")]
mod simulation;
#[cfg(target_arch = "wasm32")]
mod triangle_man;

#[cfg(target_arch = "wasm32")]
use std::sync::Arc;

use wasm_bindgen::prelude::*;

/// Start called by wasm-bindgen when the module is initialized in the browser.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn start() -> Result<(), JsValue> {
    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        let rom_resolver: Arc<dyn Fn(&str) -> Option<Box<dyn renderer::RenderRomPackage>>> =
            Arc::new(|rom_id| match rom_id {
                "anzu.triangle_man" => Some(Box::new(triangle_man::TriangleManRom)),
                _ => None,
            });
        platform_browser::run(rom_resolver).map_err(|error| JsValue::from_str(&error))?;
    }

    Ok(())
}
