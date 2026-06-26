# Architecture

## System Overview
Anzu uses a layered runtime design so gameplay evolution and rendering can scale independently.

Core layers:
- Platform shell: Browser and window/canvas integration (winit + wasm-bindgen).
- Runtime loop: Owns frame ordering and lifecycle (`update` then `render`).
- Simulation state: Source of truth for gameplay state and time evolution.
- Render extraction: Converts simulation state into GPU-ready draw parameters.
- Renderer/GPU backend: Owns device resources and submits command buffers.

Contract:
- Simulation mutates world state.
- Rendering reads state snapshots and must not contain gameplay business rules.
- Resource creation/destruction is centralized in renderer-owned lifecycle code.

## Rendering Pipeline
Frame flow:
1. Platform receives redraw event.
2. Runtime computes `delta_time` and runs simulation update.
3. Renderer acquires surface texture and encodes commands.
4. Renderer binds pipeline resources and submits draw calls.
5. Surface presents frame.

Current vertical slice target:
- One rotating triangle.
- One render pipeline.
- One vertex buffer and one uniform bind group for transform data.

Near-term rendering rules:
- Keep per-frame dynamic data in uniform buffers.
- Reuse pipeline and buffer objects across frames.
- Handle surface resize/lost/suboptimal events without panic.

## Resource Lifecycle
Lifecycle ownership:
- Initialization stage creates long-lived resources (pipeline, static buffers, bind group layouts).
- Frame stage updates transient data only (uniform writes, command encoder, render pass).
- Resize stage reconfigures surface-dependent resources.
- Shutdown stage allows window/event loop teardown without leaking resources.

Guidelines:
- Avoid resource re-creation in the hot path unless invalidated.
- Keep buffer usage flags minimal and explicit.
- Keep GPU state and simulation state represented as separate structs.

Residency and classification rules:
- Every runtime asset is classified as `critical`, `scaled_optional`, `reused`, or `streaming`.
- Runtime tracks residency state per handle (`Queued`, `Loading`, `Resident`, `EvictionPending`, `Evicted`).
- Quality fallback is mandatory for streamable visuals (fallback LOD/placeholder must exist).
- Eviction policy is class-specific and pressure-aware, not one global strategy.

Budget model:
- Budget hierarchy: `global` -> `cpu_wasm` and `gpu` -> per-class pools.
- Budget signals are dynamic and may change during a session.
- Pressure responses are staged: lower optional quality first, then evict non-critical content.
- Frame-critical logic must continue running when optional content is constrained.

Streaming subsystem:
- World content is partitioned into zones/cells with explicit dependencies.
- Cell activation is visibility-led with neighbor prefetch and unload hysteresis.
- Upload/stream work uses priority queues and bounded per-frame upload budgets.
- Renderer consumes resident handles only; streaming system drives transitions asynchronously.

## Performance Considerations
Runtime performance policy:
- Frame pacing: maintain stable pacing and avoid large dt spikes driving simulation instability.
- Allocation strategy: no avoidable heap allocations in per-frame hot path.
- Upload strategy: write only changed uniform/state data per frame.
- Batching: prefer fewer render passes and predictable state changes.

Profiling gates:
- Establish baseline metrics before adding complex systems (auth, networking, compute offload).
- Require measured bottlenecks before introducing compute pipelines.
- Keep fallback paths for browser backend variability (WebGPU/WebGL).

Future extension seams:
- Asset loading should produce content specs that instantiate simulation state.
- Networking should operate through transport abstractions and snapshot policies.
- Security should enforce deny-by-default and per-request authorization checks in server-facing components.
