# Developer Guide

## Repository Structure
- `anzu-engine/` — Core Rust library compiled to WebAssembly for browser deployment.
  - `src/lib.rs` — wasm-bindgen startup and browser event loop wiring.
  - `src/renderer.rs` — GPU state, rendering pipeline, resize/reconfigure behavior.
  - `src/asset_manifest.rs` — manifest fetch/parse/validate loader.
  - `Cargo.toml` — Rust dependencies and wasm target configuration.
  - `static/` — Browser shell and static runtime assets.
- `docs/` — Documentation set (architecture, developer, deployment, operations, user, API).
- `BACKLOG.md` — Top-down roadmap and task tracking.

## Local Development

### Prerequisites
Ensure you have:
- Rust toolchain (1.70+) with the `wasm32-unknown-unknown` target:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- `wasm-pack` (recommended for simplest workflow):
  ```bash
  cargo install wasm-pack
  ```
  *Note:* `wasm-pack` is actively maintained by the wasm-bindgen team ([current docs](https://wasm-bindgen.github.io/wasm-pack/)).
  *Alternatively:* Use `wasm-bindgen` CLI directly after a cargo build (lower-level, more manual control).

### Building the WASM Package

**Option A: Using `wasm-pack` (recommended)**
```bash
cd anzu-engine
wasm-pack build --target web --out-dir static/pkg
```

**Option B: Using `cargo` + `wasm-bindgen` CLI**
```bash
cd anzu-engine
cargo build --lib --release --target wasm32-unknown-unknown
wasm-bindgen \
  --target web \
  --out-dir static/pkg \
  target/wasm32-unknown-unknown/release/anzu_engine.wasm
```

### Hosting the Static Files

Serve the `static/` directory with any HTTP server. A few common options:

**Python 3 (built-in)**
```bash
cd anzu-engine/static
python3 -m http.server 8000
```
Then open `http://localhost:8000/index.html` in your browser.

**Node.js http-server**
```bash
npm install -g http-server
cd anzu-engine/static
http-server
```

**Simple shell script (for development)**
```bash
cd anzu-engine/static && python3 -m http.server 8000 &
sleep 1 && open http://localhost:8000/index.html  # macOS
# On Linux, use: xdg-open http://localhost:8000/index.html
# On Windows, use: start http://localhost:8000/index.html
```

### Browser Requirements
- **Supported:** modern desktop browsers with WebGPU support preferred.
- **Fallback behavior:** runtime initializes browser WebGPU with WebGL fallback support via wgpu browser backends.
- **Testing:** use browser Developer Tools console to inspect startup, manifest loader, and render errors.

### Development Workflow

1. **Edit code** in `anzu-engine/src/*.rs`.
2. **Rebuild** the wasm package using `wasm-pack build`.
3. **Refresh the browser** to load the new version (or enable auto-reload if using a file watcher).
4. **Debug** via browser DevTools (console for `console.log` output from Rust).

### Typical Validation Commands

```bash
cd anzu-engine
cargo build --lib --target wasm32-unknown-unknown --release
wasm-pack build --target web --out-dir static/pkg --release
```

### Formatting and Linting

- **Format Rust code:**
  ```bash
  cd anzu-engine && cargo fmt
  ```
- **Check for lint warnings:**
  ```bash
  cd anzu-engine && cargo clippy --target wasm32-unknown-unknown
  ```

### Testing Workflow
- Unit tests can be run on the native target:
  ```bash
  cd anzu-engine && cargo test
  ```
- Browser-specific tests require a browser test harness (not yet integrated; see Milestone 2).

## Conventions
- **Rust style:** Follow standard Rust idioms; use `cargo fmt` and `cargo clippy` to maintain consistency.
- **WebGPU best practices:** 
  - Use wgpu v29+ API.
  - Keep shader code as embedded WGSL strings (see examples in triangle demo code).
  - Avoid deprecated or platform-specific wgpu APIs.
- **Documentation expectations:**
  - Add doc comments (`///`) to public functions and types in Rust.
  - Keep README files in each module directory up-to-date with build/run instructions.
  - Link to external resources (wgpu docs, WebGPU spec) when relevant.
