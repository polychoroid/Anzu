# Deployment

## Build and Release

Primary deploy target is a browser-hosted static site that serves the compiled wasm package and JS bindings for the current Triangle Man runtime.

### Build Artifacts
- `static/index.html` - browser entry page and canvas bootstrap.
- `static/manifest.json` - startup manifest payload loaded at boot.
- `static/pkg/*` - generated wasm/js bundle from wasm-pack.

### Release Build

```bash
cd anzu-engine
wasm-pack build --target web --out-dir ../static/pkg --release
```

### Local Smoke Test

```bash
cd static
python3 -m http.server 8000
```

Open `http://localhost:8000/index.html` and verify:
- canvas initializes
- Triangle Man runtime starts and responds to input
- fixed-timestep simulation keeps running after resize
- resize does not crash rendering
- console shows manifest load result
- material-driven blend behavior is visible (opaque asteroids, alpha player edges, additive bullet glow)
- bullet hits on asteroids produce four fragments and those fragments do not disappear immediately when impact happens near screen bounds

## Environment

Current runtime is browser-first, loads configuration from static assets at startup, and uses the manifest-driven browser shell.

- The browser shell fetches `manifest.json` relative to the served static root.
- WebGPU is preferred; browser WebGL fallback remains available through wgpu's browser backend support.
- The deployed experience is the fixed 60 Hz Triangle Man runtime described in the rest of the docs set.

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
wasm-pack build --target web --out-dir ../static/pkg --release
```

3. Optional hosted smoke test in PR environments (serve `static/` and verify canvas startup, manifest loading, resize handling, and Triangle Man input/render behavior).
