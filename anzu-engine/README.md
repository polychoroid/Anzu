Anzu Engine
===========

Browser-first Rust/WASM runtime crate for Anzu.

Current implementation highlights:
- WebAssembly startup path using wasm-bindgen + winit web event loop.
- wgpu renderer with rotating triangle demo.
- Frame contract: `update(delta_time)` then `render()` on redraw.
- Asset manifest loading pattern (`manifest.json`) with typed validation errors.

Structure
- `src/lib.rs` - browser startup, event loop integration, redraw/update/render scheduling.
- `src/renderer.rs` - GPU state creation, resize handling, frame rendering.
- `src/asset_manifest.rs` - async fetch/parse/validate manifest loader.
- `static/index.html` - browser shell + canvas bootstrap.
- `static/manifest.json` - sample manifest used by startup loader.

Build and run

1. Build the wasm package:

```bash
wasm-pack build --target web --out-dir static/pkg --release
```

2. Serve static assets:

```bash
python3 -m http.server --directory static 8000
```

3. Open:
- http://localhost:8000/index.html

Alternative lower-level build:

```bash
cargo build --lib --release --target wasm32-unknown-unknown
wasm-bindgen \
	--target web \
	--out-dir static/pkg \
	target/wasm32-unknown-unknown/release/anzu_engine.wasm
```

