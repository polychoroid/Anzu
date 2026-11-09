# Anzu

Anzu is a browser-first engine written in Rust and compiled to WebAssembly. It uses wgpu for WebGPU and WebGL rendering. The project roadmap includes data-driven assets, networking, and secure remote content delivery.

Live site: [https://polychoroid.github.io/Anzu/](https://polychoroid.github.io/Anzu/)

## Current Source State

- `anzu-runtime` defines simulation, protocol, local-server, session, input, display, and asset interfaces. `LocalServer` queues inputs, advances one simulation step per tick, and stores published update batches.
- `SessionData` owns protocol state and input/display adapters, but it does not dequeue server batches or apply updates. The current display therefore does not show simulation updates.
- `anzu-winit` starts a `winit` event loop, creates a local `You Touched My Boats` session, and sends a tick event every 16 ms. Each tick advances one simulation step and requests a redraw.
- `anzu-render-wgpu` has an early GLTF asset loader and `wgpu` surface renderer. It draws one instance and does not yet provide complete multi-instance or surface-recovery behavior.
- The `You Touched My Boats` ROM maps keyboard input to actions, but action application only logs them. Its update application is unfinished.
- `anzu-terminal` has a terminal input loop, but its display backend is unfinished. `RemoteServer` and `PredictiveRemoteServer` are empty declarations; no remote transport exists.
Check the API and architecture documents for implemented interfaces and target design. Do not treat historical notes or design recommendations as current features.

## Repository Layout

- `anzu-runtime/` - runtime interfaces, local server, session, input, display, and asset types.
- `anzu-winit/` - browser app, event loop, and `winit` platform adapters.
- `anzu-render-wgpu/` - early GLTF loader and `wgpu` renderer.
- `anzu-terminal/` - terminal input loop and unfinished display adapter.
- `roms/` - domain-specific simulation packages.
- `docs/` - Browser entry page, manifest, content assets, and generated wasm/js package.
- `documents/` - Architecture, developer, deployment, operations, and API documentation.
- `BACKLOG.md` - Top-down implementation roadmap with milestones/tasks.

## Quick Start

The checked-in browser page imports generated files from `docs/pkg/`. Serving these files does not rebuild them from the Rust source.

1. Start a static file server:

```bash
python3 -m http.server 8000 --directory docs
```

2. Open `http://localhost:8000/` in a browser.

This serves the checked-in `anzu_winit` WebAssembly package, which can differ from current source. See the [developer guide](documents/developer-guide.md) for workspace checks and the current limitations. There is no verified release workflow yet.

## Documentation Index

- [Runtime Integration Brainstorms](documents/runtime-integration-brainstorms.md) - resume context, scheduling explanation, and future contract discussions.
- `documents/architecture.md` - runtime and renderer layering contracts.
- `documents/developer-guide.md` - local build, test, and development workflow.
- `documents/deployment.md` - packaging and release checklist.
- `documents/operations.md` - runtime diagnostics and maintenance guidance.
- `documents/api.md` - module responsibilities and public API notes.
- `documents/user-guide.md` - user-facing run behavior and troubleshooting.
- `documents/runtime-execution-recommendations.md` - historical runtime recommendations. The current architecture and roadmap take precedence.
- `documents/design-consideration-roms-as-separate-wasm.md` - proposal for separate WebAssembly ROM modules.
- `documents/websockets-recommendations.md` - forward-looking multiplayer and WebSocket recommendations.

## Roadmap

[BACKLOG.md](BACKLOG.md) defines the current user-outcome sequence. The architecture document records the target split between compute proposals, authoritative commit, filtered session publication, actor-partitioned session state, and display extraction. These boundaries are not fully implemented.

Pub/sub is an architectural analogy, not a microservice deployment plan. WebSocket over TLS is proposed for the first remote transport. Prediction, interpolation, and separate-WASM loading are later work, not prerequisites.

## Generative AI

Generative artificial intelligence (AI) tools may assist with research, planning, documentation, and development. AI output is a proposal and must be reviewed and validated before acceptance. Previously agent-generated source code has been removed; no AI-generated source code is currently retained in the project.

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

The browser distribution includes a license file at [docs/LICENSE](docs/LICENSE). Check each active crate manifest for its package license metadata.

Third-party dependency/license notices are tracked in `THIRD_PARTY_NOTICES.md`.
