# User Guide

## Overview
Anzu currently ships a browser demo runtime that initializes a WebGPU/WebGL-backed canvas and renders a continuously rotating triangle.

## Getting Started

### Prerequisites
- Modern browser
- Python 3 (or any static file server)
- Prebuilt package in `static/pkg`

### Build

```bash
cd anzu-engine
wasm-pack build --target web --out-dir ../static/pkg --release
```

### Run

```bash
cd static
python3 -m http.server 8000
```

Open `http://localhost:8000/index.html`.

## Usage

Expected behavior:
- Full-window canvas is displayed.
- Triangle rotates smoothly over time.
- Window resize keeps rendering active.
- Startup attempts to load `manifest.json` and logs summary/warning in console.

## Troubleshooting

- Blank page: verify server is running in `static` and `static/pkg` exists.
- Console fetch errors for `manifest.json`: verify file is present and served by same origin.
- Rendering failure: check browser console for surface/device initialization errors.
- If stale code appears, hard refresh browser cache.
