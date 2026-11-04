# Deployment

## Current Integration Scope (2026-11-04)

The active browser source is in `anzu-winit`. It starts a local `You Touched My Boats` session and connects to the current `wgpu` renderer. The session does not apply published updates, and the ROM does not yet update boat state. The terminal display is also unfinished.

No remote host, WebSocket service, release pipeline, or verified browser build procedure is implemented. WebSocket over TLS remains a proposal, not a deployment option.

`docs/index.html` imports `anzu_winit.js` and `anzu_winit_bg.wasm` from `docs/pkg/`. The checked-in `docs/manifest.json` is not loaded by the active app. Generated files may not match the Rust source.

## Serve the Checked-In Package

This procedure serves existing files. It does not rebuild or validate them.

```bash
python3 -m http.server 8000 --directory docs
```

Open `http://localhost:8000/`. Check the browser console for startup, canvas, and GPU errors. This is a prototype smoke check, not a release acceptance test.

## Environment

The browser shell loads the generated `anzu-winit` package from `docs/pkg`. The active app does not load `docs/manifest.json` or select a ROM from it.

- `docs/index.html` loads the generated JavaScript and WebAssembly files.
- `anzu-render-wgpu` selects browser WebGPU when the browser exposes a `webgpu` canvas context. Otherwise, it selects the WebGL backend.
- The native backend selection is a development path. It is not a supported desktop release target.

## Release Status

There is no verified CI/CD or release procedure. Before deployment, validate a source build, generated package freshness, browser startup, resize behavior, and surface-failure handling. Do not advertise playable boat controls or remote sessions as available features.
