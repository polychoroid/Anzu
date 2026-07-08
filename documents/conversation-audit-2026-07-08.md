# Conversation Audit - 2026-07-08

## Purpose
This document captures the main technical outcomes from the recent architecture and performance audit conversation, including:
- Confirmed design flaws.
- Poor implementation patterns in hot paths.
- Measured bottlenecks from reproducible stress tests.
- Agreed constraints and remediation direction.

## Scope And Constraints Agreed In Conversation
- Adapter boundary rule is strict:
  - Browser adapter: [anzu-engine/src/platform_browser.rs](../anzu-engine/src/platform_browser.rs)
  - Wasm-runtime adapter: [anzu-engine/src/renderer/wasm.rs](../anzu-engine/src/renderer/wasm.rs)
  - Both should be thin adapters, not owners of runtime/gameplay policy.
- Non-browser wasm hosts must remain feasible (wasm runtime is not browser-exclusive).
- WebGPU and WebGL fallback are optimized and evaluated independently.
- Current clamp excursions are warnings during instrumentation phase.
- Stress sequence starts with synthetic scenes before asset-pipeline-driven stress.
- Performance conclusions must be hypothesis-driven and reproducible.

## Key Design Flaws (Prioritized)

### 1) Critical: Adapter Boundary Violation In Wasm Runtime State
Evidence:
- [State struct in wasm runtime adapter](../anzu-engine/src/renderer/wasm.rs#L139)
- [build_batch_vertices](../anzu-engine/src/renderer/wasm.rs#L207)
- [update](../anzu-engine/src/renderer/wasm.rs#L1748)
- [render](../anzu-engine/src/renderer/wasm.rs#L1826)

Problem:
- The wasm adapter currently mixes host concerns with runtime policy and extraction behavior.
- It owns fixed-step loop orchestration, control-plane behavior, render extraction details, overlay construction, and input diagnostics in one place.

Impact:
- Weak portability to non-browser wasm and future native hosts.
- Harder testing and lower confidence in parity across hosts.
- High coupling creates high-risk refactors and slower iteration.

Target direction:
- Move fixed-step, simulation orchestration, control-policy transitions, and extraction policy into platform-neutral core modules.
- Keep wasm adapter focused on host integration only.

### 2) High: CPU-Centric Render Extraction Hot Path In Runtime Adapter
Evidence:
- [build_batch_vertices](../anzu-engine/src/renderer/wasm.rs#L207)

Problem:
- Full per-frame CPU transform and grouping of all render vertices in adapter-owned code.
- Material grouping builds dynamic vectors and batches each frame.
- Overlay generation is merged into the same extraction path.

Impact:
- Scaling risk for TriangleMan3D and large terrain scenes.
- High CPU pressure before GPU submission even starts.

Target direction:
- Move extraction policy to core and segment static vs dynamic paths.
- Preserve GPU-resident static geometry and cap per-frame transformed/uploaded dynamic data.

### 3) High: Full-Frame Dynamic Upload Pattern In Render Core
Evidence:
- [RenderCore::render](../anzu-engine/src/renderer/core.rs#L441)
- [queue.write_buffer usage](../anzu-engine/src/renderer/core.rs#L448)
- [surface acquire path](../anzu-engine/src/renderer/core.rs#L475)

Problem:
- Current path relies on large frame-time buffer writes for dynamic geometry.
- Pattern is acceptable for small 2D slices but becomes fragile as vertex counts rise.

Impact:
- Upload bandwidth and CPU staging cost become primary frame-time risk at scale.

Target direction:
- Introduce bounded dynamic upload budgets and static residency strategy.
- Keep render-core lifecycle robust for resize/lost/suboptimal handling while reducing per-frame data movement.

### 4) Medium: Trait-Object/Box Seams Need Boundary-Aware Re-evaluation
Evidence:
- [start and ROM resolver seam](../anzu-engine/src/lib.rs#L27)
- [Arc dyn Fn returning boxed trait object](../anzu-engine/src/lib.rs#L31)

Problem:
- Some dynamic seams are likely valid extension points.
- Others may be artifacts of mixed responsibilities and not true polymorphic boundaries.

Impact:
- Potentially unnecessary indirection/allocation in critical wiring paths.

Target direction:
- Re-assess dyn and Box after boundary normalization to avoid false positives.
- Keep trait objects where extension boundaries are real and stable.

### 5) Medium: Adapter Responsibilities Are Distributed Across Browser + Wasm Layers
Evidence:
- [browser app resumed path](../anzu-engine/src/platform_browser.rs#L116)
- [async state creation in browser shell](../anzu-engine/src/platform_browser.rs#L206)
- [browser event loop handling](../anzu-engine/src/platform_browser.rs#L238)

Problem:
- Separation exists but runtime behavior ownership remains blurred between adapter and engine code.

Impact:
- Limits confidence that behavior can be reproduced identically in other hosts.

Target direction:
- Make ownership contracts explicit: browser host concerns in browser adapter; shared runtime behavior in core.

## Poor Implementations Observed
- Monolithic runtime adapter state object controlling host integration, gameplay-adjacent policy, and extraction in one unit.
- Per-frame extraction workload bundled with overlay geometry in a single pass.
- Scaling-sensitive CPU-side work concentrated in adapter layer instead of explicit core subsystems.
- Limited structured performance output before this session, making longitudinal comparison hard.

## What Was Implemented During This Conversation

### A) Simulation Stress Harness (Component-Level)
Key additions in [anzu-engine/src/simulation.rs](../anzu-engine/src/simulation.rs):
- [run_stress_profile](../anzu-engine/src/simulation.rs#L1194)
- [stress_grid_manual_dense_sparse_csv](../anzu-engine/src/simulation.rs#L1496)

Outcomes:
- Dense/sparse synthetic profiles with percentile reporting (p50/p95/p99).
- Structured CSV-like output rows for repeatable comparison.

### B) Render Extraction Stress Harness (Component-Level)
Key additions in [anzu-engine/src/simulation.rs](../anzu-engine/src/simulation.rs):
- [run_render_extraction_profile](../anzu-engine/src/simulation.rs#L1323)
- [render_extraction_manual_dense_sparse_csv](../anzu-engine/src/simulation.rs#L1572)

Outcomes:
- Measured extraction loop cost using synthetic mesh-instance worlds.
- Structured CSV output for dense vs sparse extraction profiles.

## Measured Bottleneck Signals

### Simulation Stress (Dense Synthetic Grid)
Observed in manual run:
- Broadphase dominated stage time distribution.
- Candidate pair counts rose sharply in dense setups.

Representative historical signal from this conversation:
- candidate_pairs_peak about 32376 in dense 28x28 profile.
- broadphase p50 around 120 ms in that dense profile.

### Render Extraction Stress (New Structured Benchmark)
Observed from manual dense/sparse extraction CSV run:
- Dense:
  - p50_ns=851240
  - p95_ns=937599
  - p99_ns=1014792
  - transformed_vertices_peak=6144
- Sparse:
  - p50_ns=166617
  - p95_ns=198195
  - p99_ns=221118
  - transformed_vertices_peak=1536

Interpretation:
- Extraction cost scaled roughly with transformed vertex workload.
- Dense profile is about 5x p50 and about 4.7x p95 over sparse in this synthetic benchmark.

## Verification And Reproducibility Commands
Run from [anzu-engine](../anzu-engine):

- Simulation stress CSV:
  - `cargo test simulation::tests::stress_grid_manual_dense_sparse_csv -- --ignored --nocapture`
- Render extraction stress CSV:
  - `cargo test simulation::tests::render_extraction_manual_dense_sparse_csv -- --ignored --nocapture`

Both emit header plus scenario rows suitable for copy/paste into trend tracking.

## Documentation Gaps Still Open
- A final, single consolidated prioritized findings report covering:
  - Adapter-boundary defects.
  - Rust idiom issues beyond dyn and Box.
  - WGPU/GPUWeb extension risks and migration staging.
- Explicit migration map from current symbols to target modules for adapter/core split.
- Unified benchmark runner that emits simulation and extraction CSV under one scenario label and sample window.

## Recommended Next Remediation Order
1. Extract core runtime ownership out of [renderer/wasm.rs](../anzu-engine/src/renderer/wasm.rs) into platform-neutral modules.
2. Isolate render extraction into explicit core subsystem with static/dynamic geometry split.
3. Add upload-budget instrumentation in render core and cap dynamic bytes per frame.
4. Re-evaluate dyn and Box usage after boundary cleanup.
5. Add unified benchmark outputs for side-by-side simulation and extraction regression gates.
