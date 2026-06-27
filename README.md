# Anzu

Anzu is a browser-first Rust/WASM engine project focused on a deterministic runtime core, WebGPU/WebGL rendering via wgpu, and a roadmap toward data-driven assets, networking, and secure remote content delivery.

Current project status (Milestone 3 complete):
- Browser-hosted runtime loop with fixed 60 Hz simulation tick.
- Full ECS implementation with multi-entity rendering and physics.
- Input system supporting keyboard and mouse events (gamepad support is planned).
- Deterministic simulation (input replay produces identical output).
- Working game: Triangle Man (Asteroids-like demo with player control, shooting, and asteroid spawning).
- Collision detection and rigid body physics with configurable restitution.
- Manifest-based asset loading pattern is implemented.
- Core architecture and backlog are documented in the docs set.

## Repository Layout

- `anzu-engine/` - Rust crate compiled to WebAssembly and loaded by browser shell.
- `static/` - Browser entry page, manifest, content assets, and generated wasm/js package.
- `documents/` - Architecture, developer, deployment, operations, and API documentation.
- `BACKLOG.md` - Top-down implementation roadmap with milestones/tasks.

## Quick Start

1. Build WASM package:

```bash
cd anzu-engine
wasm-pack build --target web --out-dir ../static/pkg --release
```

2. Serve browser assets:

```bash
cd static
python3 -m http.server 8000
```

3. Open `localhost:8000` and play Triangle Man (Asteroids-like game).

**Controls**:
- **W/A/S/D** - Thrust forward/left/reverse/right
- **Space/Mouse** - Fire bullets
- **Objective** - Destroy asteroids; avoid collisions

## Documentation Index

- `documents/architecture.md` - runtime and renderer layering contracts.
- `documents/developer-guide.md` - local build, test, and development workflow.
- `documents/deployment.md` - packaging and release checklist.
- `documents/operations.md` - runtime diagnostics and maintenance guidance.
- `documents/api.md` - module responsibilities and public API notes.
- `documents/user-guide.md` - user-facing run behavior and troubleshooting.

## Milestones

### Completed
- **Milestone 1** (Walking Skeleton): Browser WASM engine foundation with basic render loop
- **Milestone 2** (ECS Foundation): Multi-entity rendering, physics, collision detection, and lifecycle management
- **Milestone 3** (Input & Determinism): Input-driven movement, deterministic simulation, and Asteroids-like game logic

### In Progress / Upcoming
Full implementation roadmap is tracked in `BACKLOG.md`:
- Milestone 3.5: Generic input peripherals (gamepad/mouse mappings)
- Milestone 4: Spatial broadphase optimization
- Milestone 5: Performance profiling gates
- Milestone 6: Multiplayer session support

## License

Engine crate is licensed under MIT; see `anzu-engine/LICENSE`.
