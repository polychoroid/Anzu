# User Guide

## Overview
The instructions below describe the legacy browser artifacts and texture demo. They do not describe the new `anzu-winit` integration, which is incomplete. Everything under `anzu-engine` is excluded from the next development stage.

The [current roadmap](../BACKLOG.md#current-priority-session-and-authority-integration) starts with local sessions, committed state updates, and session-owned display extraction. Remote connections are planned, not an available user feature. See [architecture](architecture.md#runtime-rewrite-direction) for the target design.

## Getting Started

### Prerequisites
- Modern browser
- Python 3 (or any static file server)
- Prebuilt package in `docs/pkg`

### Build

```bash
cd anzu-engine
wasm-pack build --target web --out-dir ../docs/pkg --release
```

### Run

```bash
cd docs
python3 -m http.server 8000
```

Open `http://localhost:8000/index.html` in a browser.

## Usage

Expected behavior:
- The runtime displays a full-window canvas.
- `TextureDemoRom` starts and renders its scene.
- The runtime uses a 16 ms fixed step and limits each frame to five steps.
- Rendering continues after a window resize.
- Press `P` to change projection mode.
- Press `Escape` to exit.

## Troubleshooting

- Blank page: Confirm that the server runs from `docs` and that `docs/pkg` exists.
- Startup failure: Confirm that the server hosts `index.html` and the generated files in `docs/pkg` from the same origin.
- Rendering failure: Check the browser console for surface or device initialization errors.
- Stale code: Hard-refresh the browser page.
