# Deployment

## Build and Release

Primary deploy target is browser-hosted static content + wasm package.

### Build Artifacts
- `static/index.html` - browser entry page.
- `static/manifest.json` - startup manifest payload.
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
- triangle rotates continuously
- resize does not crash rendering
- console shows manifest load result

## Environment

Current runtime is browser-first and reads configuration from static assets.

- Backend selection is handled by wgpu browser initialization.
- WebGPU is preferred; browser WebGL fallback is available through wgpu browser backend support.
- `manifest.json` is fetched relative to the served static root.

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

3. Optional hosted smoke test in PR environments (serve `static/` and run browser check script).
