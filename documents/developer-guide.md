# Developer Guide

## Repository Structure
- `anzu-engine/` — Core Rust library compiled to WebAssembly for browser deployment.
  - `src/lib.rs` — wasm-bindgen startup and browser event loop wiring.
  - `src/platform_browser.rs` — browser shell, canvas hookup, and top-level event loop integration.
  - `src/renderer/mod.rs` / `src/renderer/wasm.rs` — GPU state, material-aware draw extraction, resize/reconfigure behavior, and browser input normalization.
  - `src/renderer/core.rs` — render pipelines, material registry, blend-mode routing, and per-material uniform updates.
  - `src/asset_manifest.rs` — manifest fetch/parse/validate loader.
  - `Cargo.toml` — Rust dependencies and wasm target configuration.
- `docs/` — Browser shell and runtime assets served for local/dev/prod static hosting.
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
wasm-pack build --target web --out-dir ../docs/pkg --release
```

**Option B: Using `cargo` + `wasm-bindgen` CLI**
```bash
cd anzu-engine
cargo build --lib --release --target wasm32-unknown-unknown
wasm-bindgen \
  --target web \
  --out-dir ../docs/pkg \
  target/wasm32-unknown-unknown/release/anzu_engine.wasm
```

### Hosting the Static Files

Serve the `docs/` directory with any HTTP server. A few common options:

**Python 3 (built-in)**
```bash
cd docs
python3 -m http.server 8000
```
Then open `http://localhost:8000/index.html` in your browser.

**Node.js http-server**
```bash
npm install -g http-server
cd docs
http-server
```

**Simple shell script (for development)**
```bash
cd docs && python3 -m http.server 8000 &
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
wasm-pack build --target web --out-dir ../docs/pkg --release
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
- Browser-specific tests require a browser test harness (not yet integrated).
- Triangle Man includes regression tests for fragment independence and near-edge fragment survival behavior.
- Input-context routing changes should be validated with:
  ```bash
  cd anzu-engine && cargo test renderer::control_plane::tests:: -- --nocapture
  cd anzu-engine && cargo test triangle_man::tests:: -- --nocapture
  cd anzu-engine && cargo test triangle_man_2::tests:: -- --nocapture
  cd anzu-engine && cargo check --target wasm32-unknown-unknown
  ```

### Brief Tutorial: Unit Testing with `simulation/physics.rs`

This module is a good unit-test target because it has deterministic math helpers (`get_box_inertia`) and state construction behavior (`BodyState::new`) that can be validated without browser runtime setup.

1. Pick pure or near-pure behavior first.
- Best first tests: `BodyState::get_box_inertia` and `BodyState::default`.
- These tests verify stable physics config math and default state assumptions.

2. Add a local test module in `anzu-engine/src/simulation/physics.rs`.
- Use a `#[cfg(test)] mod tests` block at the bottom of the file.
- Import only what you need from `super` and `nalgebra`.

3. Start with one numeric assertion on inertia.
- Example shape: mass = 12.0, extents = (2, 4, 6).
- Expected diagonal inertia values:
  - $I_x = m(y^2 + z^2)/12 = 52$
  - $I_y = m(x^2 + z^2)/12 = 40$
  - $I_z = m(x^2 + y^2)/12 = 20$
- Assert matrix diagonal entries with a small epsilon (for example `1e-5`).

4. Add a default-construction assertion.
- Validate that `BodyState::default()` succeeds and produces finite values for inverse mass/inertia.
- This catches accidental regressions such as division-by-zero paths or invalid mesh-derived extents.

5. Run focused tests during iteration.
```bash
cd anzu-engine
cargo test simulation::physics:: -- --nocapture
```

Note on console output during tests:
- `println!` and `eprintln!` are allowed in unit tests.
- Rust test harness captures output by default and only shows it on failure.
- Use `-- --nocapture` to stream test output while tests run.

6. Expand coverage only after the first tests pass.
- Add edge tests for zero/near-zero mass behavior if the design should support it.
- Add tests for mesh-driven spatial properties only when deterministic fixtures are available.

Minimal starter checklist:
- One passing test for `get_box_inertia` numeric correctness.
- One passing test for `BodyState::default` baseline validity.
- One negative or edge-case test documenting expected behavior for invalid inputs.

## Conventions

Runtime pipeline continuation:
- See `documents/runtime-execution-recommendations.md` for staged implementation guidance and a next-session resume checklist for Milestone 4 runtime execution work.

- **Rust style:** Follow standard Rust idioms; use `cargo fmt` and `cargo clippy` to maintain consistency.
- **Terminology baseline:** Treat Anzu as a simulation and visualization system first. Consider games a domain profile built on top of that core.
- **Glossary source of truth:** Use the shared terms in `documents/architecture.md` (`Core Glossary`) for naming docs, APIs, and module boundaries.
- **Simulation architecture:** Compose intent first, then apply it to world objects once per tick, then run broadphase/narrowphase physics, then reconcile ROM rules.
- **Spatial indexing:** Prefer uniform-grid or hash-based broadphase structures before adding more collision-heavy content.
- **ROM ownership:** Keep control vocabularies, spawn rules, and scenario tuning in ROM code or ROM-owned data, not in engine-core modules.
- **Input normalization:** Normalize browser- and device-specific gamepad quirks in `src/renderer/wasm.rs` into semantic controls such as `left_trigger` and `right_trigger`; keep raw controls like `axis_4` available only as optional escape hatches for specialized hardware.
- **Input context orchestration:** `ControlPlaneState` is the sole owner of active context stack state at runtime. ROMs expose context IDs and may request transitions via `SimulationModel::pop_input_context_request()`, but they do not own final context activation.
- **Overlay preemption:** When overlay UI is visible, engine overlay context is active and gameplay input forwarding is suppressed. When overlay closes, runtime resumes the prior ROM context from the engine-owned stack.
- **Reserved key rule:** `Escape` is reserved for engine overlay toggle and must not be used in ROM bindings (`bind_rom_action` rejects it).
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

## GLB Physics Mesh Ingestion Guidance

Decision:
- Use GLB as the source package, but canonicalize imported geometry into a dedicated physics mesh representation before solver use.

Why:
- Display mesh data and physics mesh data have different requirements (materials, normals, tangents, skinning vs collision shape and mass property construction).
- GLB primitives often use index formats and scene-node transforms that should be normalized into engine-owned local-space geometry before broadphase/narrowphase processing.

Recommended contract:
1. Treat GLB as input transport only. Build canonical local-space physics mesh records during import.
2. Keep physics mesh topology immutable at runtime unless a topology mutation path is explicitly requested.
3. Store one shared shape entry per unique physics mesh and reference it from body state via shape id.
4. Keep body state for transform/motion/material overrides separate from shared shape geometry.
5. Persist construction-time mass properties with the shape record; derive per-body inverse mass and inverse inertia from policy.

Blender export profile (baseline):
1. Apply object transforms before export so local-space geometry is stable.
2. Triangulate geometry during export to avoid importer-side ambiguity.
3. Export unit scale consistently and document expected world units.
4. Exclude non-physics helper meshes from collision ingestion by naming convention or metadata tag.

Import validation checklist:
1. Verify vertex/index counts and index range validity after decode.
2. Reject or repair degenerate triangles before physics mesh registration.
3. Compute and assert finite centroid, extents, and mass properties.
4. Confirm deterministic canonicalization for repeated imports of identical content.
5. Log shape id reuse ratio to validate shared-geometry behavior.
