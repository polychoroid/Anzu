# Anzu

Anzu is a browser-first engine written in Rust and compiled to WebAssembly. It uses wgpu for WebGPU and WebGL rendering. The project roadmap includes data-driven assets, networking, and secure remote content delivery.

Live site: [https://polychoroid.github.io/Anzu/](https://polychoroid.github.io/Anzu/)

## Current Source State

- `anzu-runtime` contains simulation, server, input, and session prototypes. `SimulationIO` declares shared command/update types but is not integrated yet.
- `anzu-winit` is the browser frontend and creates a local session. Its event-loop and app-state integration remain incomplete. Native `target_arch` paths may support development-workspace runs.
- `anzu-render-wgpu` is a placeholder package, not a completed backend.
- Local committed-update delivery, session state, display extraction, and remote transport are the next integration work. They are not implemented features.
- Everything under `anzu-engine` is excluded from this development stage. Earlier generated browser artifacts are not evidence that the new runtime is complete.

Check the API and architecture documents for implemented interfaces and target design. Do not treat archived code or design recommendations as current features.

## Repository Layout

- `anzu-runtime/` - runtime contracts, local authority prototype, and session logic.
- `anzu-winit/` - browser app and winit platform input integration.
- `anzu-render-wgpu/` - planned rendering backend, currently a placeholder.
- `anzu-terminal/` - terminal application and adapters.
- `roms/` - domain-specific simulation packages.
- `docs/` - Browser entry page, manifest, content assets, and generated wasm/js package.
- `documents/` - Architecture, developer, deployment, operations, and API documentation.
- `BACKLOG.md` - Top-down implementation roadmap with milestones/tasks.

## Quick Start

Legacy browser-artifact workflow only: the commands below target the excluded engine, not the new runtime integration. They have not been revalidated in this documentation pass. A working `anzu-winit` build/deployment recipe and a remote host package are still pending. See the [current development direction](documents/architecture.md#runtime-rewrite-direction).

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

- [Runtime Integration Brainstorms](documents/runtime-integration-brainstorms.md) - resume context, scheduling explanation, and future contract discussions.
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

[BACKLOG.md](BACKLOG.md#current-priority-session-and-authority-integration) defines the current sequence. The 2026-10-16 vision separates compute proposals, authoritative commit, filtered session publication, actor-partitioned session state, and display extraction. Sessions own input/display adapters. Local and remote connections share IO without requiring a simulation on remote clients.

Pub/sub is an architectural analogy, not a microservice deployment plan. WebSocket over TLS is proposed for the first remote transport. Prediction, interpolation, and separate-WASM loading are later work, not prerequisites.

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
