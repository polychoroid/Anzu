Anzu Engine
===========

Browser-first Rust/WASM runtime crate for Anzu.

Current implementation highlights:
- WebAssembly startup path using wasm-bindgen + winit web event loop.
- wgpu renderer running ROM-selected browser runtime via `rom_id` dispatch.
- Two ROM packages are currently wired: `anzu.triangle_man` and `anzu.triangle_man_2`.
- Frame contract: `update(delta_time)` then `render()` on redraw.
- Asset manifest loading pattern (`manifest.json`) with typed validation errors.
- Material registry and per-entity material assignment through `material_id`.
- Blend-aware render pipelines (`Opaque`, `Alpha`, `Additive`) selected per draw batch.
- Per-material uniform parameters with dynamic buffer offsets.
- PBR-lite fragment shading controls (emissive + metallic/roughness/specular parameters).
- Role-pair collision policy table in Triangle Man drives solid vs hitbox outcomes.
- Bullet hitbox impacts can despawn and replace asteroids with four fragment entities.
- Fragment edge-spawn regression tests are present for near-boundary survival behavior.

Structure
- `src/lib.rs` - browser startup, event loop integration, redraw/update/render scheduling.
- `src/platform_browser.rs` - browser shell integration and top-level window/canvas lifecycle.
- `src/renderer/mod.rs` - renderer module entry points.
- `src/renderer/wasm.rs` - browser renderer state, frame loop hooks, and browser input normalization.
- `src/renderer/core.rs` - GPU pipelines, material registry, and per-batch material uniform binding.
- `src/asset_manifest.rs` - async fetch/parse/validate manifest loader.
- `../docs/index.html` - browser shell + canvas bootstrap.
- `../docs/manifest.json` - startup manifest used by browser runtime loader.

Input model
- Browser adapters produce generic input events.
- Browser-specific gamepad quirks are normalized in `src/renderer/wasm.rs` to semantic controls where possible (for example trigger aliases).
- ROM code consumes semantic controls by default; raw controls remain available as optional escape hatches for specialized hardware.

Build and run

1. Build the wasm package:

```bash
wasm-pack build --target web --out-dir ../docs/pkg --release
```

2. Serve static assets:

```bash
python3 -m http.server --directory ../docs 8000
```

3. Open:
- http://localhost:8000/index.html

Automated wasm smoke check

Use this quick command sequence to catch common wasm runtime regressions before manual browser testing:

```bash
cargo test && \
cargo check --target wasm32-unknown-unknown && \
wasm-pack build --target web --out-dir ../docs/pkg --release
```

Expected result:
- all tests pass
- wasm check succeeds
- `docs/pkg` is regenerated without build errors

Alternative lower-level build:

```bash
cargo build --lib --release --target wasm32-unknown-unknown
wasm-bindgen \
	--target web \
	--out-dir ../docs/pkg \
	target/wasm32-unknown-unknown/release/anzu_engine.wasm
```

License compliance check

Install `cargo-deny`:

```bash
cargo install cargo-deny
```

Run dependency license checks:

```bash
cargo deny --config deny.toml check licenses
```

