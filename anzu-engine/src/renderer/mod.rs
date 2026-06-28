#[cfg(target_arch = "wasm32")]
mod core;

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm::State;

#[cfg(target_arch = "wasm32")]
pub use core::{MaterialBlendMode, MaterialDefinition, RenderRomPackage, RomRenderData};
