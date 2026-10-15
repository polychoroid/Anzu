# Deployment

## Build and Release

The primary deployment target is a browser-hosted static site. It serves the compiled WebAssembly package and JavaScript bindings for the ROM that the manifest selects.

### Build Artifacts
- `docs/index.html` - browser entry page and canvas bootstrap.
- `docs/manifest.json` - legacy startup metadata that the active engine does not load.
- `docs/pkg/*` - generated wasm/js bundle from wasm-pack.

### Release Build

```bash
cd anzu-engine
wasm-pack build --target web --out-dir ../docs/pkg --release
```

### Local Smoke Test

```bash
cd docs
python3 -m http.server 8000
```

Open `http://localhost:8000/index.html` and verify these results:
- The canvas initializes.
- The texture demo renders its scene.
- Press `P` to change projection mode. Press `Escape` to exit.
- The runtime continues to render after a resize.
- The browser console does not report startup or rendering errors.

## Environment

The current browser shell loads the generated engine package from `docs/pkg`. The active engine entry point does not load `docs/manifest.json` or select a ROM from it.

- `docs/index.html` loads the generated JavaScript and WebAssembly files.
- The WebAssembly target enables wgpu's WebGL feature. Do not assume WebGPU support from the native build.
- Press `P` to change projection mode. Press `Escape` to exit.

## CI/CD

Recommended pipeline steps:

1. Format and lint the Rust crate:

```bash
cd anzu-engine
cargo fmt --check
cargo clippy --target wasm32-unknown-unknown
```

2. Build the WebAssembly package:

```bash
cd anzu-engine
cargo build --lib --target wasm32-unknown-unknown --release
wasm-pack build --target web --out-dir ../docs/pkg --release
```

3. In a pull request environment, optionally run a hosted smoke test. Serve `docs/` and verify canvas startup, resize handling, and texture-demo behavior.
