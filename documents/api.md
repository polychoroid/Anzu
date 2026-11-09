# API Reference

## Scope

This document covers the active runtime workspace. The 2026-10-16 brainstorm supersedes the earlier API inventory for a removed implementation and its direct-world snapshot examples. No stable wire API is implemented.

## Current Source Interfaces

- [Simulation contract](../anzu-runtime/src/simulation.rs): `Simulation<P>` defines an `Action`, input adaptation, action application, asset listing, update publication, and a simulation step. It has no snapshot or `SimulationIO` contract.
- [Server](../anzu-runtime/src/server.rs): `LocalServer<P, S>` owns a simulation, queued input signals, and published update batches. Each tick maps queued signals, applies supported actions, calls `step`, and queues a returned update batch. `Server::dequeue_update_batch` exposes those batches. `RemoteServer` and `PredictiveRemoteServer` are empty declarations.
- [Session](../anzu-runtime/src/session.rs): `SessionData` owns input and display adapters, a server, and protocol state. It queues mapped input and requests rendering from `State::project_display`. It does not dequeue server updates or call `State::apply_update`; the protocol state therefore does not receive published batches through this path.
- [Winit app](../anzu-winit/src/lib.rs) and [app handler](../anzu-winit/src/app.rs): `run` starts a `winit` event loop. A 16 ms interval sends tick events. The app builds a local `BoatTouching` session, attaches a window, loads display assets, routes keyboard events, resizes the display, and requests redraws. Its native path is a development-workspace path, not a desktop product package.
- [Winit adapters](../anzu-winit/src/input.rs) and [display adapter](../anzu-winit/src/display.rs): keyboard events map to input signals. The display adapter selects WebGPU or WebGL in a browser and a native backend for non-WASM builds. Browser backend creation is asynchronous; native creation blocks.
- [wgpu package](../anzu-render-wgpu/src/lib.rs): exports a GLTF loader and a `wgpu` renderer. The loader reads positions and `TEXCOORD_0`, creates material base-color textures and samplers, and supplies a white fallback for materials without a texture. The WGSL shader samples the base-color texture and multiplies it by the material factor. The current source is not yet pipeline-valid: the shader and material bind group use texture bindings 1 and 2, but `Material::layout()` declares only binding 0. The renderer also uploads texture pixels in the render path and draws one selected instance; it is not a complete multi-instance renderer or robust surface-recovery path.
- [ROM package](../roms/you%20touched%20my%20boats/src/lib.rs): `BoatTouching` provides the local simulation and `GameState` protocol types. It maps keyboard signals to actions and advances a tick counter. `GameState::apply_update` is unfinished, and its update queue is not populated by the current simulation behavior.
- [Terminal app](../anzu-terminal/src/main.rs): creates the same local ROM and server, polls terminal input, and requests redraws. Its display backend methods are unfinished.

These are source observations, not build or test results. The local input-to-server-to-step path exists, but published batches are not applied to session state. The browser draw path is also incomplete and can fail when it receives no instances or an asset without a material.

## Target Session and Authority Protocol

The [architecture](architecture.md#runtime-rewrite-direction) defines target ownership. The [roadmap](../BACKLOG.md) records user outcomes. The contracts below are proposed, not implemented APIs.

### Separate IO and State

The target `SimulationIO` concept would supply shared command/update types independently of world storage, session storage, a GPU device, or a simulation instance. No such type exists in the active source.

The recommended shape is a simulation with associated IO, a server-connection trait with associated IO, and compatible session state. A remote client must not need a `Simulation` implementation just to name protocol types.

Decision, 2026-10-17: the platform host hardcodes the concrete local or remote connection type. Runtime switching is not required, so the target design does not retain a local/remote `Server` enum. A session generic over the connection contract supports both concrete implementations through static dispatch. Exact trait signatures remain to be defined.

`WorldState` and `SessionState` can have different layouts. Reusing update instructions requires compatible application semantics, not identical memory layout. Filtered state can require a projected update contract. Snapshots and connection-control messages also need representation beyond the two existing associated types.

### Command and Update Flow

1. Input mapping produces a domain command, or a signal for host-side mapping if that alternative is selected.
2. The connection queues the command with authenticated session and authorized actor scope.
3. Computation proposes changes against stable state and fixed simulation time.
4. A 1:1 ordered queue delivers proposals to the world state database.
5. The state owner validates and commits accepted updates.
6. It publishes committed events through permission filtering to 1:n session channels.
7. The session applies updates to actor-partitioned state without repeating expensive computation.

Commands express intent; proposed updates are not committed facts. Queue acceptance is not execution acknowledgement. The database is a logical state owner, not a selected persistence product.

### Local and Remote Adapters

| Adapter | Required responsibilities |
|---|---|
| Local | Simulation/state coordination, shared IO, in-memory command/update queues |
| Remote client | Shared IO, codec, connection lifecycle, bounded send/receive queues; no simulation instance |
| Remote host | Authority, authenticated routing, actor authorization, filtered publication, transport |

A new host package is planned. Shared domain types must be usable without linking simulation implementation code. A separate protocol package is optional. See [transport recommendations](websockets-recommendations.md).

### Connection Operations

- Queue command: report acceptance for delivery, saturation, or failure. Define ownership of a rejected command.
- `dequeue_update()`: immediately return the next complete committed batch if available. Distinguish an empty queue from closed or failed transport.
- Status and initialization: expose readiness and snapshot/recovery progress without blocking the platform loop.

Async transport work fills the receive queue. Local and remote modes preserve compatible ordering, scope, and failure semantics.

### Message and Recovery Requirements

Define these before stabilizing message types or serialization:

- Protocol version and bounded message/batch sizes.
- Authenticated `session_id` binding and authorized `actor_id` selection. Payload identifiers alone do not establish permission.
- Command identity and retry acknowledgement policy. Reconnect must not silently execute retried commands twice.
- World revision and subscription sequence. Filtered revisions must not look like lost messages.
- Batch boundaries, duplicate/gap handling, and stale compute-result policy.
- Permitted snapshot contents and snapshot-to-stream handoff without missing updates.
- Queue saturation, slow-consumer recovery, disconnect, resynchronization, and permission changes.

Update operations versus final values, shared actor data, reduced session representations, and compute state reads remain open.

### Session and Display Contracts

Session owns input and display adapters plus permitted state. The display adapter is an extraction-only trait. It reads a stable view of the session's permitted state and produces backend-independent data with resource handles. Actor authorization determines which actor state enters the session; extraction may compose data for multiple permitted actors into one display output, including separate views such as split screen. It does not own a surface or submit GPU work.

Hard partitioning selects the `session_id` delivery channel. Soft partitioning selects actor-specific state within that session. Session queries exclude actor partitions the session is not permitted to access. This is a state-disclosure boundary, not a one-actor-per-display rule. A deployment may bind separate sessions to separate renderer surfaces, such as one local session and monitor per actor. Both partition layers work with authentication, authorization, and TLS.

App owns session and renderer. The renderer owns resource preparation, frame acquire, encoding, submission, and present. The session's permitted actor set and the composition of those actors into display views are separate decisions. Whether a display adapter receives session scope at construction or a view selection per extraction remains undecided.

### Processing Loops

Authority time is not defined by redraw count. A local app can service an elapsed-time fixed-step scheduler from the same callback as rendering. This does not require another thread. Scheduler ownership, catch-up limits, and local suspension policy remain open. A remote session tick cannot advance shared authority.

The session processes input and available committed batches, then extracts a stable permitted view. Prediction and interpolation are low-priority future consideration after the baseline and remote recovery are validated. Current contracts must not require predictive state, rollback/replay buffers, or a simulation in remote clients.

For timing details and future discussion topics, see [Future Brainstorms](runtime-integration-brainstorms.md). These are documentation plans, not runtime changes.

Acceptance checks cover local/remote reconstruction parity, commit-before-publish, actor/session exclusion, snapshot continuity, retries, malformed messages, bounded queues, unsupported input, and redraw-independent ticks. Browser checks cover initialization, resize, hidden-tab resume, and renderer failure without state rollback.
