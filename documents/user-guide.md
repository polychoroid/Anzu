# User Guide

## Overview
Anzu currently provides a browser runtime. It initializes a canvas and runs `TextureDemoRom`.

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
