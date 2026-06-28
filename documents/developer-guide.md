# Developer Guide

## Repository Structure
- `anzu-engine/` — Core Rust library compiled to WebAssembly for browser deployment.
  - `src/lib.rs` — wasm-bindgen startup and browser event loop wiring.
  - `src/platform_browser.rs` — browser shell, canvas hookup, and top-level event loop integration.
  - `src/renderer/mod.rs` / `src/renderer/wasm.rs` — GPU state, material-aware draw extraction, resize/reconfigure behavior, and browser input normalization.
  - `src/renderer/core.rs` — render pipelines, material registry, blend-mode routing, and per-material uniform updates.
  - `src/asset_manifest.rs` — manifest fetch/parse/validate loader.
  - `Cargo.toml` — Rust dependencies and wasm target configuration.
- `static/` — Browser shell and static runtime assets.
- `documents/` — Documentation set (architecture, developer, deployment, operations, user, API).
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
wasm-pack build --target web --out-dir ../static/pkg --release
```

**Option B: Using `cargo` + `wasm-bindgen` CLI**
```bash
cd anzu-engine
cargo build --lib --release --target wasm32-unknown-unknown
wasm-bindgen \
  --target web \
  --out-dir ../static/pkg \
  target/wasm32-unknown-unknown/release/anzu_engine.wasm
```

### Hosting the Static Files

Serve the `static/` directory with any HTTP server. A few common options:

**Python 3 (built-in)**
```bash
cd static
python3 -m http.server 8000
```
Then open `http://localhost:8000/index.html` in your browser.

**Node.js http-server**
```bash
npm install -g http-server
cd static
http-server
```

**Simple shell script (for development)**
```bash
cd static && python3 -m http.server 8000 &
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
wasm-pack build --target web --out-dir ../static/pkg --release
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
- Triangle Man includes regression tests for fragment independence and near-edge fragment survival behavior.

## Conventions
- **Rust style:** Follow standard Rust idioms; use `cargo fmt` and `cargo clippy` to maintain consistency.
- **Simulation architecture:** Compose intent first, then apply it to world objects once per tick, then run broadphase/narrowphase physics, then reconcile ROM rules.
- **Spatial indexing:** Prefer uniform-grid or hash-based broadphase structures before adding more collision-heavy content.
- **ROM ownership:** Keep control vocabularies, spawn rules, and scenario tuning in ROM code or ROM-owned data, not in engine-core modules.
- **Input normalization:** Normalize browser- and device-specific gamepad quirks in `src/renderer/wasm.rs` into semantic controls such as `left_trigger` and `right_trigger`; keep raw controls like `axis_4` available only as optional escape hatches for specialized hardware.
- **WebGPU best practices:** 
  - Use wgpu v29+ API.
  - Keep shader code as embedded WGSL strings (see the current renderer implementation).
  - Avoid deprecated or platform-specific wgpu APIs.
- **Material system conventions:**
  - Prefer material parameters and registry data over adding effect-specific fields to shared vertex structs.
  - Keep per-entity visual differences in `MaterialDefinition`/`material_id`, not in ECS shape/component proliferation.
  - Add new blend or shading behavior by extending material pipeline routing before introducing new render-only entity pathways.
- **Collision policy conventions:**
  - Keep role-pair outcome behavior in the editable policy table rather than hardcoded role trait branches.
  - Treat bullet interactions as hitbox events when physical impulse transfer is intentionally disabled.
  - When adding fragment replacement behavior, include regression coverage for world-boundary spawn behavior.
- **Resource classification:**
  - Declare asset class and priority in manifest metadata (`critical`, `scaled_optional`, `reused`, `streaming`).
  - Treat missing classification metadata as a validation error.
- **Streaming and async rules:**
  - Use staged load flow (`fetch` -> `decode` -> `upload` -> `activate`) for streamable assets.
  - Keep upload work off frame-critical path via queue tiers (`critical_stream`, `normal_decode`, `background_cleanup`).
  - Enforce per-frame upload byte limits to avoid frame spikes.
- **Budgeting and fallback:**
  - Track CPU/WASM and GPU usage separately when possible.
  - Under pressure, degrade optional quality first (LOD/fallback), then evict non-critical content.
  - Do not block simulation ticks waiting for optional high-LOD content.
- **Verification gates:**
  - New asset/streaming work must include measurable checks (startup time, frame time, memory, queue depth, eviction churn).
  - Include at least one pressure test proving graceful degradation and recovery.
- **Simulation profiling gates:**
  - Record per-phase fixed-tick timings before and after broadphase or pipeline changes.
  - Keep candidate pair counts, collision counts, and frame pacing visible in logs during stress runs.
- **Documentation expectations:**
  - Add doc comments (`///`) to public functions and types in Rust.
  - Keep README files in each module directory up-to-date with build/run instructions.
  - Link to external resources (wgpu docs, WebGPU spec) when relevant.
