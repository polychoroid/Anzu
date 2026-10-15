# Anzu

Anzu is a browser-first engine written in Rust and compiled to WebAssembly. It uses wgpu for WebGPU and WebGL rendering. The project roadmap includes data-driven assets, networking, and secure remote content delivery.

Live site: [https://polychoroid.github.io/Anzu/](https://polychoroid.github.io/Anzu/)

## Current Source State

- `anzu-engine` is a Rust library that uses `winit` for the window and event loop and `wgpu` for rendering.
- The active engine entry point starts `TextureDemoRom` directly. It does not select a ROM from `docs/manifest.json`.
- `TextureDemoRom` supplies scene content and a background-color command. Press `P` to change projection mode. Press `Escape` to exit.
- The engine source has separate `assets`, `execution`, `roms`, and `simulation` modules.
- Older generated engine code remains under `anzu-engine/src/abandoned_generated_code/`. The active crate does not compile those ROM implementations.
- The `anzu-runtime` crate contains an early in-process session and authority prototype. It is not a production multiplayer service.

Check the API and architecture documents for implemented interfaces and target design. Do not treat archived code or design recommendations as current features.

## Repository Layout

- `anzu-engine/` - Rust crate compiled to WebAssembly and loaded by browser shell.
- `docs/` - Browser entry page, manifest, content assets, and generated wasm/js package.
- `documents/` - Architecture, developer, deployment, operations, and API documentation.
- `BACKLOG.md` - Top-down implementation roadmap with milestones/tasks.

## Quick Start

1. Build the WebAssembly package:

```bash
cd anzu-engine
wasm-pack build --target web --out-dir ../docs/pkg --release
```

2. Start a web server for the browser assets:

```bash
cd docs
python3 -m http.server 8000
```

3. Open `localhost:8000`. The browser shell loads the generated engine package.

In the current engine build, press `P` to change projection mode. Press `Escape` to exit.

## Documentation Index

- `documents/architecture.md` - runtime and renderer layering contracts.
- `documents/developer-guide.md` - local build, test, and development workflow.
- `documents/deployment.md` - packaging and release checklist.
- `documents/operations.md` - runtime diagnostics and maintenance guidance.
- `documents/api.md` - module responsibilities and public API notes.
- `documents/user-guide.md` - user-facing run behavior and troubleshooting.
- `documents/runtime-execution-recommendations.md` - implementation recommendations and restart checklist for Milestone 4 runtime pipeline work.
- `documents/design-consideration-roms-as-separate-wasm.md` - proposal for separate WebAssembly ROM modules.
- `documents/websockets-recommendations.md` - forward-looking multiplayer and WebSocket recommendations.

## Roadmap

`BACKLOG.md` is the source of truth for milestone status and planned work. Roadmap items do not necessarily describe features in the active engine. See [Current Source State](#current-source-state) for the current runtime summary.

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
