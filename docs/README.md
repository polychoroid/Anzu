Anzu Browser Runtime Assets
===========================

This folder contains browser-hosted runtime assets for Anzu.

Contents
- `index.html` - browser shell that loads the generated wasm/js package and displays runtime diagnostics.
- `manifest.json` - runtime manifest consumed by the engine startup path.
- `pkg/` - generated output from `wasm-pack` or `wasm-bindgen` (JavaScript glue + wasm binary).

Build and run

1. Build the engine package from the crate directory:

```bash
cd ../anzu-engine
wasm-pack build --target web --out-dir ../docs/pkg --release
```

2. Serve this `docs/` directory:

```bash
python3 -m http.server 8000
```

3. Open:
- http://localhost:8000/index.html

Notes
- If browser behavior seems inconsistent (focus, gamepad exposure, permission policy), use the in-page diagnostics panel in `index.html`.
- `pkg/` contents are generated artifacts and may be replaced on rebuild.

