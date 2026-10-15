# Developer Guide

## Repository Structure
- `anzu-engine/` — Rust library for browser WebAssembly and native targets.
  - `src/lib.rs` — application startup, event handling, update, and render scheduling.
  - `src/assets/` — mesh data providers.
  - `src/ecs/` — entity and world types.
  - `src/execution/` — rendering, compute, materials, pipeline building, backend, and shared state.
  - `src/extraction/` — render-data extraction.
  - `src/roms/texture_demo.rs` — active texture-demo content.
  - `src/simulation/physics.rs` — simulation physics.
  - `src/abandoned_generated_code/` — archived code that the active crate does not compile.
- `anzu-runtime/` — early in-process session and authority prototype. It is not a production network service.
- `docs/` — browser shell, manifest file, assets, and generated WebAssembly package.
- `documents/` — architecture, API, development, deployment, operations, and user guides.
- `BACKLOG.md` — roadmap and task tracking.

## Local Development

### Prerequisites
Install these tools:
- Rust toolchain (1.85 or later) with the `wasm32-unknown-unknown` target:
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

**Option A: Use `wasm-pack`**
```bash
cd anzu-engine
wasm-pack build --target web --out-dir ../docs/pkg --release
```

**Option B: Use `cargo` and the `wasm-bindgen` CLI**
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
Open `http://localhost:8000/index.html` in your browser.

**Node.js http-server**
```bash
npm install -g http-server
cd docs
http-server
```

**Simple shell command (for development)**
```bash
cd docs && python3 -m http.server 8000 &
sleep 1 && open http://localhost:8000/index.html  # macOS
# On Linux, use: xdg-open http://localhost:8000/index.html
# On Windows, use: start http://localhost:8000/index.html
```

### Browser Requirements
- **Targets:** Check `anzu-engine/Cargo.toml` for the configured browser and native targets.
- **Browser backend:** The WebAssembly target enables wgpu's WebGL feature. Do not infer WebGPU support from the native build.
- **Testing:** Inspect the browser developer console for startup and rendering errors.

### Development Workflow

1. Edit Rust files in `anzu-engine/src/`.
2. Rebuild the WebAssembly package with `wasm-pack build`.
3. Refresh the browser to load the new package.
4. Inspect browser developer tools for console output and runtime errors.

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
- The repository does not define a browser test harness. Test browser-specific behavior in a browser.
- Run engine tests and the WebAssembly target check with:
  ```bash
  cd anzu-engine && cargo test
  cd anzu-engine && cargo check --target wasm32-unknown-unknown
  ```

### Brief Tutorial: Unit Testing with `simulation/physics.rs`

The physics module is a good unit-test target. Its deterministic math and state construction can be checked without browser setup.

1. Pick pure or near-pure behavior first.
- Best first tests: the inertia calculation and `BodyState::default`.
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

Console output during tests:
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

- **Rust style:** Use standard Rust idioms. Run `cargo fmt` and `cargo clippy` to check the code.
- **Terminology baseline:** Treat Anzu as a simulation and visualization system first. Consider games a domain profile built on top of that core.
- **Glossary source of truth:** Use the shared terms in `documents/architecture.md` (`Core Glossary`) for names in docs, APIs, and module boundaries.
- **Simulation architecture:** Compose intent first, then apply it to world objects once per tick, then run broadphase/narrowphase physics, then reconcile ROM rules.
- **Spatial indexing:** Prefer uniform-grid or hash-based broadphase structures before adding more collision-heavy content.
- **ROM ownership:** Keep control vocabularies, spawn rules, and scenario tuning in ROM code or ROM-owned data, not in engine-core modules.
- **Input behavior:** The active texture demo handles `P` to change projection mode. It handles `Escape` to exit. Older gamepad and input-context code is in `src/abandoned_generated_code/`.
- **WebGPU best practices:** 
  - Use the wgpu version defined in `anzu-engine/Cargo.toml` (currently v30).
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
- **Design status:** The asset-class, streaming, budgeting, and pressure-response items below are target guidance. Confirm implementation status in source before describing them as current features.
- **Documentation expectations:**
  - Add doc comments (`///`) to public functions and types in Rust.
  - Keep README files in each module directory up-to-date with build/run instructions.
  - Link to external resources (wgpu docs, WebGPU spec) when relevant.

## Proposed GLB Physics Mesh Ingestion

The active engine does not load GLB files. Treat this section as design guidance, not as a current workflow.

Recommended direction:
- Use GLB as a source package, then convert its geometry into a dedicated physics mesh before the solver uses it.

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

## Proposed Blender GLB Manifest v1

The active engine does not load the scene manifest described here. Do not use this proposal as a current authoring workflow.

Design goal:
- Use a manifest-backed startup scene so Blender-exported GLB content can have deterministic runtime behavior without source edits.

Blender export settings (exact options):

| Blender UI path | Option name | Value | Why |
|---|---|---|---|
| File > Export > glTF 2.0 | Format | GLB | Single binary payload per object file. |
| Include | Selected Objects | On (recommended) | Prevent accidental helper mesh export. |
| Transform | +Y Up | On | Keep runtime axis contract consistent. |
| Transform | Apply Modifiers | On | Bake final mesh result used by runtime. |
| Geometry | UVs | On | Required for textured material sampling. |
| Geometry | Normals | On | Required for stable lighting/shading behavior. |
| Geometry | Tangents | On (recommended) | Future-proof for tangent-space workflows. |
| Geometry | Vertex Colors | On (if used) | Preserve authored color channels. |
| Geometry | Compression | Off (first pass) | Reduce parser complexity for initial loader. |
| Animation | Animations | Off (first pass) | This feature scope targets static and transform-driven startup content. |
| Animation | Skinning | Off (first pass) | Skeletal paths are out of scope for 11.5. |
| Materials | Materials | Export | Preserve material slot structure for mapping. |
| Materials | Images | Automatic | Keep referenced texture payloads consistent with export. |

Pre-export scene prep:
1. Object Mode > Object > Apply > All Transforms.
2. Edit Mode > Mesh > Clean Up > Degenerate Dissolve (if needed).
3. Object Data Properties > Normals > ensure custom normals are valid when used.
4. UV Editor > confirm UVMap exists and is non-empty for textured meshes.

Schema summary:
1. File path: `anzu-engine/src/roms/files/manifest/texture_demo.scene.json`
2. Required top-level fields: `manifest_version`, `profile_id`, `objects`, `textures`, `startup_scene`
3. `objects[]`: `object_asset_id`, `source.kind`, `source.path`, `default_material_id`, `generate_body`
4. `textures[]`: `texture_id`, `path`, `color_space`
5. `startup_scene.instances[]`: `instance_id`, `object_asset_id`, `transform`, `texture_id`, `animation`

Compact example:
```json
{
  "manifest_version": "1",
  "profile_id": "texture_demo",
  "objects": [
    {
      "object_asset_id": "tetra_a",
      "source": { "kind": "glb", "path": "platonic/tetrahedron.glb" },
      "default_material_id": "mat_default",
      "generate_body": true
    }
  ],
  "textures": [
    {
      "texture_id": "checker_01",
      "path": "checker/checker_01.png",
      "color_space": "srgb"
    }
  ],
  "startup_scene": {
    "instances": [
      {
        "instance_id": "tetra_a_0",
        "object_asset_id": "tetra_a",
        "transform": {
          "translation": [0.0, 0.0, 0.0],
          "rotation_euler_deg": [0.0, 0.0, 0.0],
          "scale": [1.0, 1.0, 1.0]
        },
        "texture_id": "checker_01",
        "animation": { "mode": "spin", "phase_offset": 0.0 }
      }
    ]
  }
}
```

Validator failure targets:
1. Schema/version: `ManifestVersionUnsupported`, `ManifestMissingField`, `ManifestUnknownField`
2. Identity: `DuplicateObjectAssetId`, `DuplicateTextureId`, `DuplicateInstanceId`
3. Path safety and files: `ObjectPathOutsideRoot`, `TexturePathOutsideRoot`, `ObjectFileNotFound`, `TextureFileNotFound`
4. Reference integrity: `ObjectReferenceNotFound`, `TextureReferenceNotFound`, `StartupSceneEmpty`
5. GLB/content: `ObjectSourceKindUnsupported`, `GlbParseFailed`, `GlbMissingNormals`, `GlbMissingUv0`, `TransformScaleInvalid`
