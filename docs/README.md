Anzu Browser Runtime Assets
===========================

This folder contains the browser-hosted runtime assets for Anzu.

## Contents

- `index.html` - loads the generated WebAssembly and JavaScript package into the canvas.
- `manifest.json` - legacy startup metadata. The active engine entry point does not load this file.
- `pkg/` - contains generated output from `wasm-pack` or `wasm-bindgen`, including JavaScript glue and the WebAssembly binary.

## Build and Run

1. Build the engine package from its crate directory:

```bash
cd ../anzu-engine
wasm-pack build --target web --out-dir ../docs/pkg --release
```

2. Start a web server in this `docs/` directory:

```bash
python3 -m http.server 8000
```

3. Open this URL:
- http://localhost:8000/index.html

## Notes

- Inspect the browser console for startup and rendering errors.
- A rebuild can replace the generated files in `pkg/`.

