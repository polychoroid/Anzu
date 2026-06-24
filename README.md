# Anzu

Anzu is a browser-first Rust/WASM engine project focused on a deterministic runtime core, WebGPU/WebGL rendering via wgpu, and a roadmap toward data-driven assets, networking, and secure remote content delivery.

Current project status:
- Browser-hosted runtime loop is in place.
- Rotating triangle render slice works in WASM.
- Manifest-based asset loading pattern is implemented.
- Core architecture and backlog are documented in the docs set.

## Repository Layout

- `anzu-engine/` - Rust crate compiled to WebAssembly and loaded by browser shell.
- `docs/` - Architecture, developer, deployment, operations, and API documentation.
- `BACKLOG.md` - Top-down implementation roadmap with milestones/tasks.

## Quick Start

1. Build WASM package:

```bash
cd anzu-engine
wasm-pack build --target web --out-dir static/pkg --release
```

2. Serve browser assets:

```bash
cd anzu-engine/static
python3 -m http.server 8000
```

3. Open:
- http://localhost:8000/index.html

## Documentation Index

- `docs/architecture.md` - runtime and renderer layering contracts.
- `docs/developer-guide.md` - local build, test, and development workflow.
- `docs/deployment.md` - packaging and release checklist.
- `docs/operations.md` - runtime diagnostics and maintenance guidance.
- `docs/api.md` - module responsibilities and public API notes.
- `docs/user-guide.md` - user-facing run behavior and troubleshooting.

## Milestones

Primary runtime roadmap is tracked in `BACKLOG.md`.
Milestone 1 (browser-hosted rotating triangle skeleton) is mostly complete, including asset manifest loading scaffolding.

## License

Engine crate is licensed under MIT; see `anzu-engine/LICENSE`.
