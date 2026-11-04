Anzu Browser Runtime Assets
===========================

This folder contains the browser-hosted runtime assets for Anzu.

## Runtime Integration Status (2026-10-16)

`index.html` loads the generated `anzu_winit.js` package from `pkg/`. The active Rust source is in `anzu-winit`; `manifest.json` is not read by that app. Generated files can be older than the source and do not prove that the current source builds or runs.

The current browser app starts a local `You Touched My Boats` session and a `wgpu` display adapter. The ROM update path is incomplete, and no remote host or transport exists. See the [API reference](../documents/api.md#current-source-interfaces) for the implemented interfaces and limits.

## Contents

- `index.html` - loads the generated WebAssembly and JavaScript package into the canvas.
- `manifest.json` - legacy startup metadata. The active app does not load this file.
- `pkg/` - contains generated output from `wasm-pack` or `wasm-bindgen`, including JavaScript glue and the WebAssembly binary.

## Build and Run

The following build command is a candidate workflow. It was not run during this documentation review.

1. Build the browser package from the active crate:

```bash
cd ../anzu-winit
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

