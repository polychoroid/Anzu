# API Reference

## Scope

This document describes the active public modules in `anzu-engine`. It separates current source from proposed runtime contracts. It does not describe archived generated code as an active API.

## Active Modules

- `assets` exposes mesh types and `mesh_provider`. `MeshType` currently supports built-in primitives and a file handle. `BodyState` uses a built-in cube when it receives a file handle; file mesh loading is not implemented.
- The crate does not export `ecs` or `extraction` from `lib.rs`. Their source files are not part of the active public module tree.
- `execution` exposes `compute`, `render`, and `shared`. The current `compute.rs` file is empty.
- `execution::shared` exposes `BackgroundColor`, `BackgroundColorError`, `FrameScheduler`, `FrameStepPlan`, and `RuntimeState`.
- `execution::render` exposes the renderer types and builder states used by `lib.rs`, including `ProjectionMode`, `TexturedMesh`, `TexturedVertex`, and renderer construction functions.
- `roms` currently exposes `texture_demo::TextureDemoRom` and its scene-content and render-command types.
- `simulation` exposes `physics`, including `BodyState`, `MotionVector`, and `ForceVector`.
- `lib.rs` exposes `run()`. On WebAssembly, it also exposes `run_web()` as the wasm-bindgen start function.

Check Rust visibility in source before treating a type as a supported external API. Some public types are application implementation details and do not have compatibility guarantees.

## Current Runtime Behavior

- The application starts `TextureDemoRom` directly. It does not load `docs/manifest.json` or select a ROM by `rom_id`.
- The event handler updates runtime state and renders on redraw.
- Press `P` to change projection mode. Press `Escape` to exit.
- `RuntimeState` uses a fixed 16 ms step, limits work to five steps per frame, and clamps elapsed frame time to 250 ms.
- `BackgroundColor::new` checks that every color channel is in the range 0.0 to 1.0. It returns `BackgroundColorError` when a channel is outside this range.
- The current physics constructor loads built-in primitive meshes. File-based mesh loading is not implemented.

These statements describe the current source. The runtime rewrite and networking sections below describe target design only.

## Target Session and Authority Protocol

This section records a design contract for the runtime rewrite. The examples are illustrative and are not implemented public types in the browser WASM engine API described above.

### Command and Update Flow

The session sends typed domain commands to the simulation authority. The authority validates commands, applies them in authoritative order, advances the world, and publishes updates. The command channel is a transport boundary. It is not the simulation.

Commands describe domain actions, not transport details. A protocol may include commands such as:

```rust
enum SessionCommand {
	JoinPlayer { player_id: PlayerId, name: String },
	LeavePlayer { player_id: PlayerId },
	MoveActor { actor_id: ActorId, delta: Vec3 },
	Interact { actor_id: ActorId, target_id: ActorId },
	RequestWorldState { player_id: PlayerId },
}
```

The authority publishes authoritative snapshots or result events through a typed update stream. For example:

```rust
enum WorldUpdate {
	Snapshot { tick: u64, world: WorldState },
	Event { event: SimulationEvent },
	Correction { tick: u64, corrected_world: WorldState },
	Rejected { command_id: u64, reason: String },
}
```

These names and fields are protocol design examples only. The eventual implementation must define serialization, identity, ordering, error, and compatibility rules before exposing a stable wire format.

### Local and Remote Adapters

In local mode, in-process channels connect the session to the authority. In remote mode, a session adapter encodes domain commands and a server adapter decodes them. The server adapter also encodes updates for the session adapter. Both modes use the same domain contract; only the transport changes.

Start with a local channel implementation when implementing this design. It supports deterministic tests and provides a direct path to a remote adapter without putting transport details in simulation commands.

### Processing Loops

The session loop:

1. Reads the visible world view and queued actor input.
2. Predicts local state when prediction is enabled.
3. Sends commands to the authority.
4. Receives authoritative updates.
5. Corrects or reconciles its local view.

The authority loop:

1. Receives commands from sessions.
2. Validates and applies commands in authoritative order.
3. Advances the world according to simulation rules.
4. Emits events, rejections, and snapshots.
5. Publishes updates to the relevant sessions.

The authority owns world truth. The session owns local prediction and participant-facing policy. The transport carries messages between them and does not change either ownership boundary.
