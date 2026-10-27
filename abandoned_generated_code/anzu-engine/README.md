Anzu Engine
===========

Anzu's Rust engine library. It supports browser WebAssembly and native targets.

## Current Implementation

- `winit` starts the native or browser event loop. `wasm-bindgen` exposes the WebAssembly startup function.
- The engine entry point starts `TextureDemoRom` directly. It does not use `rom_id` dispatch.
- On each redraw, the event handler calls the update stage and then renders the frame.
- `TextureDemoRom` provides textured scene content and queues background-color updates.
- Press `P` to change projection mode. Press `Escape` to exit.
- The crate has separate modules for assets, execution, ROM content, and simulation physics.
- Older generated engine code is stored in `src/abandoned_generated_code/`. The active crate does not compile it.

## Source Structure

- `src/lib.rs` - starts the event loop and owns the current application setup.
- `src/assets/` - provides mesh data to the renderer.
- `src/ecs/` - contains the entity-component-system world.
- `src/execution/` - contains rendering, compute, materials, pipeline building, backend, and shared runtime code.
- `src/extraction/` - contains render-data extraction code.
- `src/roms/texture_demo.rs` - defines the active texture-demo content and behavior.
- `src/simulation/physics.rs` - contains simulation physics types and behavior.
- `../docs/index.html` - loads the generated WebAssembly package into the canvas.
- `../docs/manifest.json` - remains in the repository, but the active engine entry point does not load it.

## Input Model

- `winit` delivers window and input events to the application handler.
- The current demo handles `P` to change projection mode and `Escape` to exit.
- The active engine does not include the older browser gamepad normalization path. That code remains in `src/abandoned_generated_code/`.

## Build and Run

1. Build the WebAssembly package:

```bash
wasm-pack build --target web --out-dir ../docs/pkg --release
```

2. Start a web server for the static assets:

```bash
python3 -m http.server --directory ../docs 8000
```

3. Open this URL:
- http://localhost:8000/index.html

## Automated WebAssembly Smoke Check

Run these checks before browser testing:

```bash
cargo test && \
cargo check --target wasm32-unknown-unknown && \
wasm-pack build --target web --out-dir ../docs/pkg --release
```

Expected result:
- All tests pass.
- The WebAssembly check succeeds.
- The build regenerates `docs/pkg` without errors.

Alternative lower-level build:

```bash
cargo build --lib --release --target wasm32-unknown-unknown
wasm-bindgen \
	--target web \
	--out-dir ../docs/pkg \
	target/wasm32-unknown-unknown/release/anzu_engine.wasm
```

## License Check

Install `cargo-deny`:

```bash
cargo install cargo-deny
```

Run the dependency license checks:

```bash
cargo deny --config deny.toml check licenses
```

