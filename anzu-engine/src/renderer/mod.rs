#[cfg(any(target_arch = "wasm32", test))]
mod core;

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm::State;

#[cfg(any(target_arch = "wasm32", test))]
pub use core::{MaterialBlendMode, MaterialDefinition, RenderRomPackage, RomRenderData};
