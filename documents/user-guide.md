# User Guide

## Overview
Anzu currently ships a browser game runtime that initializes a WebGPU/WebGL-backed canvas, loads its manifest at startup, and runs Triangle Man.

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
- Triangle Man starts and responds to input.
- Simulation continues at a fixed 60 Hz tick.
- Window resize keeps rendering active.
- Startup attempts to load `manifest.json` and logs summary/warning in console.
- Materials drive visible rendering differences: asteroid shells (opaque), player edges (alpha), and bright additive bullet glow.
- Bullet hits on asteroids replace them with four smaller fragments.
- Fragment behavior remains stable near screen bounds (no immediate edge-pop despawn on spawn).

## Troubleshooting

- Blank page: verify server is running in `static` and `static/pkg` exists.
- Console fetch errors for `manifest.json`: verify file is present and served by same origin.
- Rendering failure: check browser console for surface/device initialization errors.
- If stale code appears, hard refresh browser cache.
