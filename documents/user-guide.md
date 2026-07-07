# User Guide

## Overview
Anzu currently ships a browser runtime that initializes a WebGPU/WebGL-backed canvas, loads its manifest at startup, and runs the selected ROM.

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

Open `http://localhost:8000/index.html`.

## Usage

Expected behavior:
- Full-window canvas is displayed.
- The ROM configured in `manifest.json` starts and responds to input.
- Simulation continues at a fixed 60 Hz tick.
- Window resize keeps rendering active.
- Startup attempts to load `manifest.json` and logs summary/warning in console.
- For `anzu.triangle_man`: materials drive visible rendering differences and bullet collisions can fragment asteroids.
- For `anzu.triangle_man_2`: triangle stands on the ground strip and supports walk/crawl/jump movement.

## Troubleshooting

- Blank page: verify server is running in `docs` and `docs/pkg` exists.
- Console fetch errors for `manifest.json`: verify file is present and served by same origin.
- Rendering failure: check browser console for surface/device initialization errors.
- If stale code appears, hard refresh browser cache.
