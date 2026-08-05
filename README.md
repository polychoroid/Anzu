# Anzu

Anzu is a browser-first Rust/WASM engine project focused on a deterministic runtime core, WebGPU/WebGL rendering via wgpu, and a roadmap toward data-driven assets, networking, and secure remote content delivery.

Live site: [https://polychoroid.github.io/Anzu/](https://polychoroid.github.io/Anzu/)

Current project status (through Milestone 4.6 slice):
- Browser-hosted runtime loop with fixed 60 Hz simulation tick.
- Full ECS implementation with multi-entity rendering and physics.
- Input system supporting keyboard, mouse, and gamepad events.
- Deterministic simulation (input replay produces identical output).
- Working ROMs: Triangle Man (asteroids combat) and Triangle Man 2 (platformer slice with ground, walk/crawl/jump).
- Default manifest ROM is currently Triangle Man 2 (`docs/manifest.json`).
- Collision detection and rigid body physics with configurable restitution.
- Material system foundation with per-entity `material_id` routing.
- Blend-aware render pipelines (`Opaque`, `Alpha`, `Additive`) selected per draw batch.
- Per-material uniform parameters (`base_color_tint`, `emissive_strength`, `metallic`, `roughness`, `specular_strength`).
- PBR-lite directional shading pass integrated into current WGSL fragment stage.
- Role-pair collision policy table supports configurable solid vs hitbox interactions.
- Bullet hitbox collisions can replace asteroids with four independent fragment entities.
- Edge-case fragment spawn near world bounds is covered by regression tests and spawn clamping.
- Manifest-based asset loading pattern is implemented.
- Core architecture and backlog are documented in the docs set.

## Repository Layout

- `anzu-engine/` - Rust crate compiled to WebAssembly and loaded by browser shell.
- `docs/` - Browser entry page, manifest, content assets, and generated wasm/js package.
- `documents/` - Architecture, developer, deployment, operations, and API documentation.
- `BACKLOG.md` - Top-down implementation roadmap with milestones/tasks.

## Quick Start

1. Build WASM package:

```bash
cd anzu-engine
wasm-pack build --target web --out-dir ../docs/pkg --release
```

2. Serve browser assets:

```bash
cd docs
python3 -m http.server 8000
```

3. Open `localhost:8000` and run the ROM selected in `docs/manifest.json`.

**Controls (default Triangle Man 2 ROM)**:
- **A/D** - Move left/right
- **W** - Jump
- **S** - Crawl stance

If you switch `docs/manifest.json` to `anzu.triangle_man`, controls return to the asteroids profile:
- **W/A/S/D** - Thrust forward/left/reverse/right
- **Space/Mouse Left** - Fire bullets

## Documentation Index

- `documents/architecture.md` - runtime and renderer layering contracts.
- `documents/developer-guide.md` - local build, test, and development workflow.
- `documents/deployment.md` - packaging and release checklist.
- `documents/operations.md` - runtime diagnostics and maintenance guidance.
- `documents/api.md` - module responsibilities and public API notes.
- `documents/user-guide.md` - user-facing run behavior and troubleshooting.
- `documents/runtime-execution-recommendations.md` - implementation recommendations and restart checklist for Milestone 4 runtime pipeline work.

## Milestones

### Completed
- **Milestone 1** (Walking Skeleton): Browser WASM engine foundation with basic render loop
- **Milestone 2** (ECS Foundation): Multi-entity rendering, physics, collision detection, and lifecycle management
- **Milestone 3** (Input & Determinism): Input-driven movement and deterministic simulation
- **Milestone 4.5** (Declarative Collision Outcomes): Policy-driven collisions and asteroid fragmentation in Triangle Man

### In Progress / Upcoming
Full implementation roadmap is tracked in `BACKLOG.md`:
- Milestone 4.6: Triangle Man 2 platformer scaffold (browser acceptance still open)
- Milestone 4.7: Triangle Man 3D isometric adventure (new)
- Milestone 5: Performance profiling gates
- Milestones 5.5-5.7: Extensible materials, parameterized shading, and composite multi-pass rendering
- Milestone 6: Multiplayer session support

## Special Thanks To

- [Microsoft GameInput (GDK) guidance](https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-overview)
- [Microsoft Direct3D 12 programming guidance](https://learn.microsoft.com/en-us/windows/win32/direct3d12/direct3d-12-graphics)
- [Microsoft PIX on Windows (profiling and diagnostics)](https://devblogs.microsoft.com/pix/)
- [Rust Style Guide](https://doc.rust-lang.org/style-guide/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [jeremychone-channel Rust Builder and TypeState examples](https://github.com/jeremychone-channel/rust-builder)
- [wgpu crate documentation](https://docs.rs/wgpu/latest/wgpu/)
- [Vulkan Specification (Khronos)](https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html)
- [Vulkan Guide (Khronos)](https://github.khronos.org/Vulkan-Site/guide/latest/)
- [Bevy main scheduling model reference](https://github.com/bevyengine/bevy/blob/main/crates/bevy_app/src/main_schedule.rs)
- [Fyrox executor/runtime loop reference](https://github.com/FyroxEngine/Fyrox/blob/master/fyrox-impl/src/engine/executor.rs)
- [Godot main loop/runtime reference](https://github.com/godotengine/godot/blob/master/main/main.cpp)

## License

Engine crate is licensed under MIT; see `anzu-engine/LICENSE`.

Third-party dependency/license notices are tracked in `THIRD_PARTY_NOTICES.md`.
