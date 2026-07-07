# Deployment

## Build and Release

Primary deploy target is a browser-hosted static site that serves the compiled wasm package and JS bindings for the currently selected ROM runtime.

### Build Artifacts
- `docs/index.html` - browser entry page and canvas bootstrap.
- `docs/manifest.json` - startup manifest payload loaded at boot.
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

Open `http://localhost:8000/index.html` and verify:
- canvas initializes
- ROM from `manifest.json` starts and responds to input
- fixed-timestep simulation keeps running after resize
- resize does not crash rendering
- console shows manifest load result
- For `anzu.triangle_man`: material-driven blend behavior is visible (opaque asteroids, alpha player edges, additive bullet glow)
- For `anzu.triangle_man`: bullet hits on asteroids produce four fragments and those fragments do not disappear immediately when impact happens near screen bounds
- For `anzu.triangle_man_2`: ground contact, walk, crawl, and jump behavior are visibly stable

## Environment

Current runtime is browser-first, loads configuration from static assets at startup, and uses the manifest-driven browser shell.

- The browser shell fetches `manifest.json` relative to the served static root.
- WebGPU is preferred; browser WebGL fallback remains available through wgpu's browser backend support.
- The deployed experience is the fixed 60 Hz ROM runtime selected by `manifest.json`.

## CI/CD

Recommended pipeline steps:

1. Rust format/lint:

```bash
cd anzu-engine
cargo fmt --check
cargo clippy --target wasm32-unknown-unknown
```

2. Build verification:

```bash
cd anzu-engine
cargo build --lib --target wasm32-unknown-unknown --release
wasm-pack build --target web --out-dir ../docs/pkg --release
```

3. Optional hosted smoke test in PR environments (serve `docs/` and verify canvas startup, manifest loading, resize handling, and active ROM behavior).
