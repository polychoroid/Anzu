#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(target_arch = "wasm32")]
use web_sys::{window, HtmlCanvasElement};

/// Attach the canvas with the provided id into the DOM (no-op if missing).
/// Returns `JsValue` on error to propagate to JS.
#[cfg(target_arch = "wasm32")]
pub fn attach_canvas_by_id(id: &str) -> Result<(), JsValue> {
    let win = window().ok_or_else(|| JsValue::from_str("no window"))?;
    let doc = win
        .document()
        .ok_or_else(|| JsValue::from_str("no document"))?;

    if let Some(el) = doc.get_element_by_id(id) {
        let _canvas: HtmlCanvasElement = el
            .dyn_into()
            .map_err(|_| JsValue::from_str("element is not a canvas"))?;
        // We don't need to hold onto the canvas here; the engine will query it later.
        Ok(())
    } else {
        // If canvas not present, that's fine for minimal scaffold.
        web_sys::console::warn_1(&JsValue::from_str(&format!(
            "canvas with id '{}' not found",
            id
        )));
        Ok(())
    }
}

// On non-wasm targets provide a stub so the same API exists.
#[cfg(not(target_arch = "wasm32"))]
pub fn attach_canvas_by_id(_id: &str) -> Result<(), wasm_bindgen::JsValue> {
    Ok(())
}
