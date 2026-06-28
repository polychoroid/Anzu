#[cfg(any(target_arch = "wasm32", test))]
mod asset_manifest;
#[cfg(any(target_arch = "wasm32", test))]
mod ecs;
mod input;
#[cfg(target_arch = "wasm32")]
mod platform_browser;
mod renderer;
#[cfg(any(target_arch = "wasm32", test))]
mod rom;
#[cfg(any(target_arch = "wasm32", test))]
mod simulation;
#[cfg(any(target_arch = "wasm32", test))]
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

#[wasm_bindgen]
pub fn is_game_over() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        return platform_browser::game_is_over();
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

#[wasm_bindgen]
pub fn reset_game() {
    #[cfg(target_arch = "wasm32")]
    {
        platform_browser::request_game_reset();
    }
}
