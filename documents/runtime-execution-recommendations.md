# Runtime Execution Model: Implementation Recommendations

This document is a development handoff for continuing Runtime Execution Model work (Milestone 4 in the backlog).

## Objective

Implement a hardened, deterministic tick pipeline with explicit stages, measurable per-phase costs, and scalable collision processing.

Primary targets:

- `BACKLOG.md` Milestone 4.1 through 4.6
- Deterministic simulation ordering
- Broadphase-first collision pipeline
- Profiling gates for each stage

## Attribution

This plan is informed by both current Anzu implementation details and external engineering guidance.

Internal references:

- `BACKLOG.md` (Milestone 4 and profiling requirements)
- `documents/architecture.md` (runtime/render separation and phase framing)
- `anzu-engine/src/simulation.rs` (current scheduler behavior and collision path)
- `anzu-engine/src/platform_browser.rs` (frame boundary: update then render)

External references:

- Rust Style Guide: <https://doc.rust-lang.org/style-guide/>
- Rust API Guidelines: <https://rust-lang.github.io/api-guidelines/>
- Microsoft GameInput guidance: <https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-overview>
- Direct3D 12 graphics guidance: <https://learn.microsoft.com/en-us/windows/win32/direct3d12/direct3d-12-graphics>
- PIX on Windows profiling guidance: <https://devblogs.microsoft.com/pix/>
- wgpu API reference: <https://docs.rs/wgpu/latest/wgpu/>
- Vulkan Specification: <https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html>
- Vulkan Guide: <https://github.khronos.org/Vulkan-Site/guide/latest/>
- Bevy scheduling reference: <https://github.com/bevyengine/bevy/blob/main/crates/bevy_app/src/main_schedule.rs>
- Fyrox executor/runtime loop reference: <https://github.com/FyroxEngine/Fyrox/blob/master/fyrox-impl/src/engine/executor.rs>
- Godot main loop reference: <https://github.com/godotengine/godot/blob/master/main/main.cpp>

## Current Baseline (Observed)

Current scheduler behavior in `anzu-engine/src/simulation.rs` is:

1. Integrate transform velocity into position/rotation.
2. Build per-body working set from world state.
3. Run out-of-bounds filtering.
4. Run proximity + SAT narrowphase in an all-pairs loop.
5. Apply positional correction and impulse response.
6. Write resolved body state back to world.
7. Process lifecycle TTL/despawn and emit frame report.

Current frame boundary in browser loop (`anzu-engine/src/platform_browser.rs`):

- `state.update(delta_seconds)` then `state.render()` on redraw.

This is a solid base, but stage boundaries are implicit and broadphase is not yet separated from all-pairs scanning.

## Recommended Stage Contract

Define explicit simulation pipeline stages in the scheduler contract:

1. `Sync`:
   - Snapshot world state into deterministic working sets.
   - Validate required components and defaults.

2. `Compose`:
   - Apply intent and forces exactly once per tick.
   - Integrate velocities into predicted transforms.

3. `Broadphase`:
   - Build/query spatial index (uniform grid first).
   - Emit deterministic candidate pairs only.

4. `Narrowphase`:
   - Run SAT/contact generation only for candidates.
   - Emit typed interaction events in stable order.

5. `Resolve`:
   - Apply impulse and positional correction.
   - Keep sensor/hitbox behavior policy-driven.

6. `Reconcile`:
   - Apply ROM rules (spawn/despawn/replace/damage policy).
   - Finalize frame report and scrub stale pair tracking.

7. `Publish`:
   - Write world state once from resolved buffers.
   - Keep render side read-only from simulation outputs.

## Determinism Rules

- Canonical pair ordering: `(min(entity_a, entity_b), max(entity_a, entity_b))`.
- Stable iteration order for broadphase cells and pair emission.
- No hash map iteration order dependence in event sequencing.
- Reconcile stage must not mutate ordering semantics established by physics stages.
- Any random behavior must be seeded and tick-derived.

## Broadphase Recommendation

Start with a uniform grid broadphase for simplicity and predictability.

Suggested model:

- Cell key: integer coordinates from quantized world position.
- Insert each body into overlapped cells using bounds/proximity radius.
- Candidate generation:
  - Build local pair sets per occupied cell.
  - Canonicalize pair IDs.
  - Deduplicate via ordered set.
- Sort candidates before narrowphase to preserve deterministic event order.

Why first:

- Easy to profile and reason about.
- Works well for medium-density 2D scenes.
- Lower implementation risk than sweep-and-prune for current scope.

## Profiling Requirements (Milestone 4.2)

Capture per-phase timings each tick:

- `sync_ns`
- `compose_ns`
- `broadphase_ns`
- `narrowphase_ns`
- `resolve_ns`
- `reconcile_ns`
- `publish_ns`

Also log work metrics:

- body count
- candidate pair count
- narrowphase checks
- collisions resolved
- proximity enter/exit counts

Roll up summaries every N frames (for example 60).

Instrumentation mode guidance:

- Always-on mode should keep lightweight counters only.
- High-resolution phase timing should be sampled (for example every N ticks) or gated behind a debug flag.
- Track instrumentation overhead as a first-class metric so profiling does not become a hidden bottleneck.

## Efficiency Safeguards (Must-Have)

### Hot-path allocation policy

- Do not allocate in per-tick hot loops when capacity can be reused.
- Pre-size and reuse scheduler-owned scratch buffers for:
   - body working sets
   - broadphase cell buckets/candidate storage
   - transformed polygon vertex scratch
   - dedupe and sort staging arrays

### Candidate post-processing policy

- Keep deterministic pair ordering, but avoid tree-heavy structures on the critical path when possible.
- Preferred flow:
   1. generate canonical pair IDs into contiguous buffers
   2. perform deterministic sort once
   3. run linear unique pass
- Add explicit broadphase-postprocess timing counters so dedupe/sort cost cannot hide inside broadphase totals.

### Data layout evolution policy

- Keep ECS ownership boundaries, but allow contiguous simulation-side arrays for numeric hot fields.
- If body counts rise enough that copy-in/copy-out dominates, add a dedicated migration slice to SoA-like working buffers before adding algorithmic complexity.

## Render Boundary (wgpu/Vulkan-Aligned)

Keep simulation and rendering contracts separate:

- Simulation stages finish before render extraction begins.
- Renderer reads final world snapshot only.
- No gameplay mutation in render code paths.

At frame level, preserve explicit boundaries:

1. Acquire surface texture
2. Encode commands from immutable frame snapshot
3. Submit queue
4. Present

These boundaries make profiling and future backend behavior (WebGPU/WebGL fallback) easier to reason about.

## Stepwise Implementation Plan

### Slice A: Stage API and Report Shape

- Add a stage contract type and expand `SchedulerFrameReport` with phase timings and counters.
- Keep behavior unchanged in this slice.

Acceptance:

- Existing tests pass.
- Timings/counters are present and populated.
- No additional avoidable heap allocations in the baseline tick path.
- Instrumentation overhead remains under agreed budget in baseline scenes.

### Slice B: Extract Compose and Publish Boundaries

- Separate integration/composition from publish-back-to-world writes.
- Keep same collision logic temporarily.

Acceptance:

- Deterministic event ordering tests still pass.
- No behavior regressions in Triangle Man loops.

### Slice C: Broadphase Grid + Candidate Feed

- Add uniform grid candidate generation.
- Replace all-pairs scan with candidate iteration.

Acceptance:

- Candidate count is lower than all-pairs under stress scenes.
- Narrowphase results match previous baseline for same seeds.
- Broadphase postprocess (dedupe + ordering) timing is reported separately and does not dominate narrowphase at target densities.

### Slice C.5: Data Movement Reduction (Conditional)

- Introduce contiguous simulation-side arrays for hot numeric fields if profiling shows copy traffic dominates tick cost.
- Preserve ECS boundaries by converting at stage boundaries only.

Acceptance:

- Reduced copy/writeback cost in phase timings.
- No determinism regressions in event ordering and reconcile outcomes.

### Slice D: Narrowphase/Resolve Split

- Distinguish contact generation from resolution logic.
- Keep sensor/hitbox policies unchanged.

Acceptance:

- Collision and proximity events remain stable.
- Fragment/edge regression tests remain green.

### Slice E: Reconcile Contract Hardening

- Keep reconcile as explicit final simulation mutation stage.
- Ensure spawn/despawn replacements are deterministic.

Acceptance:

- Existing collision-outcome tests pass.
- No leaked entities after despawn-and-replace flows.

### Slice F: Documentation + Profiling Gate

- Update docs and backlog checkboxes with measured evidence.
- Record before/after timing snapshots.

Acceptance:

- Milestone 4.1/4.2/4.3 evidence is documented.

## Tomorrow Start Checklist (Fast Resume)

1. Re-run baseline checks:

```bash
cd /workspaces/Anzu/anzu-engine
cargo check
cargo check --target wasm32-unknown-unknown
cargo test --target wasm32-unknown-unknown --no-run
wasm-pack build --target web --out-dir ../static/pkg --release
```

2. Implement Slice A only (stage report scaffolding).
3. Add one regression test asserting stage timing fields are emitted.
4. Commit as isolated change before broadphase work.

## Suggested Commit Sequence

1. `runtime: add explicit stage timing/counter report scaffolding`
2. `runtime: split compose and publish phases`
3. `runtime: add deterministic uniform-grid broadphase`
4. `runtime: separate narrowphase and resolve stages`
5. `runtime: harden reconcile stage and update docs`

## Out-of-Scope for This Pass

- Multithreaded scheduler redesign
- Compute offload for collision/physics
- Material/render graph expansion
- Network sync model changes

Keep this pass focused on deterministic execution model hardening and profiling visibility.
