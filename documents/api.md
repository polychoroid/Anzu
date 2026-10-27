# API Reference

## Scope

This document covers runtime integration outside `anzu-engine`. The 2026-10-16 brainstorm supersedes the earlier engine API inventory and direct-world snapshot examples. No stable wire API is implemented.

## Current Source Interfaces

- [Simulation contracts](../anzu-runtime/src/simulation.rs): `Simulation` declares `Command`, `Snapshot`, and `WorldState`. `SimulationIO` separately declares `Command` and `Update`, but is not connected to simulation methods.
- [Server](../anzu-runtime/src/server.rs): `Server<S: Simulation>` selects local or remote. Local delivery maps `InputSignal` through the simulation and calls `apply` and `step`. `RemoteServer` is empty. There is no update dequeue API.
- [Session](../anzu-runtime/src/session.rs): owns input adapter and server, but no session state or display adapter. Input mapping unwraps an optional result. A session tick forwards a timestamp to the server.
- [Winit app](../anzu-winit/src/lib.rs): creates a local `BoatTouching` session. App state and event-loop integration remain incomplete.
- [wgpu package](../anzu-render-wgpu/src/lib.rs): contains placeholder code, not a renderer implementation.

These are read-only source observations, not build or test results.

## Target Session and Authority Protocol

The [architecture](architecture.md#runtime-rewrite-direction) defines ownership. [Backlog Initiative 12](../BACKLOG.md#initiative-12-networking-and-session-features) tracks implementation. The contracts below are proposed, not implemented APIs.

### Separate IO and State

`SimulationIO` supplies shared command/update types independently of world storage, session storage, a GPU device, or a simulation instance.

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

For the detailed timing explanation and future discussion topics, see [Future Brainstorms](runtime-integration-brainstorms.md). For immediate contract tasks, see [the backlog checklist](../BACKLOG.md#next-step-contract-checklist). These are documentation plans, not runtime changes.

Acceptance checks cover local/remote reconstruction parity, commit-before-publish, actor/session exclusion, snapshot continuity, retries, malformed messages, bounded queues, unsupported input, and redraw-independent ticks. Browser checks cover initialization, resize, hidden-tab resume, and renderer failure without state rollback.
