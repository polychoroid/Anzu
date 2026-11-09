# Operations

## Current Integration Scope (2026-11-04)

The active app runs a local `You Touched My Boats` session through `anzu-winit`. A 16 ms interval requests one server tick and then a redraw. The ROM currently logs actions, and the session does not apply published updates. The app has no remote host or network transport.

### Planned Session and Authority Checks

- Track compute, commit, publication, delivery, session application, extraction, upload, and rendering separately.
- Record queue depth, update age, bytes, command rejection, snapshot recovery, and connection failures without logging unauthorized payloads.
- Confirm committed batches are published only after application and preserve session/actor scope throughout processing.
- Check snapshot/update continuity, duplicate commands, missing batches, bounded queues, and slow-consumer recovery.
- Test wrong-session delivery and actor-scoped query exclusion alongside authentication, authorization, and TLS controls.
- Keep remote authority timing independent of redraws. A failed surface frame must not roll back committed state.

These are acceptance requirements, not currently available diagnostics. Durable commit/publication recovery is conditional on a future persistence requirement.

## Runtime Monitoring

The browser console provides the current logs and errors. The checked-in page loads generated files from `docs/pkg/`, which may not match the active source.

- Check it for canvas, window, device, and rendering errors.
- Confirm that the browser app creates its canvas and GPU backend.
- Resize the window and check for surface configuration errors. The current resize path has no zero-size guard.
- Confirm that the current GLTF asset appears. The renderer uses only the first supplied instance.

The active runtime does not expose frame or stage timing distributions, collision work counts, asset-residency metrics, upload-queue metrics, or pressure metrics. Previously documented profiling code was removed and does not provide active instrumentation.

### Performance Baseline Status

Performance measurement remains planned, not a completed runtime capability. Use this status when reviewing performance claims until the roadmap's benchmark work is implemented and measured:

- Do not claim that the active runtime has a measured frame-time baseline.
- For each performance-sensitive change, record the workload, target, backend, and available external measurements. State which internal stage data is unavailable.
- Do not optimize cache behavior, alignment, or copy volume by inspection alone. Record the hypothesis and define how the result will be measured once instrumentation exists.
- Do not set performance budgets from unmeasured assumptions. First capture repeatable baseline distributions.

## Planned Verification

Use these checks only after the related profiling, collision, or streaming systems exist:

- Measure phase timings and collision candidate counts during stress tests.
- Measure memory, residency, queue depth, upload latency, and pressure transitions.
- Confirm that optional asset quality decreases before the runtime evicts non-critical assets.
- Confirm that collision and fragment outcomes match their tests.

### Performance Evidence

Use this method as soon as the baseline instrumentation and benchmark output exist:

1. Start with synthetic workloads. Then test representative asset-driven workloads.
2. Record the target, backend, workload parameters, warm-up frames, and sample count.
3. Report p50, p95, and p99 timings for each measured stage.
4. Record work metrics, such as body count, candidate-pair count, and transformed-vertex count.
5. Measure or report instrumentation overhead. State whether profiling is sampled or enabled for every frame.
6. Keep the benchmark command and raw output with the result. Re-run archived measurements against active code before using them as a baseline.
7. Compare before and after with the same workload. Report whether the difference exceeds measurement noise before describing it as an improvement or regression.

## Debugging

Common failure modes:
- The static server cannot find `index.html` or generated package files because it uses the wrong root path.
- The browser does not support the selected GPU backend.
- The generated files in `docs/pkg/` do not match the current code.

Debug workflow:
1. Confirm that the static server root is `docs/`.
2. Confirm that `docs/pkg/anzu_winit.js` and `docs/pkg/anzu_winit_bg.wasm` exist.
3. Hard-refresh the browser page.
4. Inspect the console for startup and rendering errors.
5. Rebuild only with a validated source build command. No release build recipe is verified yet.

## Planned Memory-Pressure Workflow

The active runtime does not implement this workflow. Use it as a test plan after memory budgets, asset eviction, and pressure telemetry exist.

1. Reproduce the issue with constrained memory settings or high-content scenes.
2. Check pressure-transition logs and class-specific eviction reasons.
3. Confirm that the runtime lowers optional quality before forced eviction.
4. Confirm that recovery hysteresis prevents rapid load and unload cycles.

## Maintenance

- Review Rust dependency updates in the workspace manifests.
- Run format, lint, and build checks before merging when the active workspace supports them.
- Keep docs synchronized whenever runtime contracts or commands change.
- Track milestone acceptance criteria in backlog as implementation evolves.
- Define CDN versioning and cache-purge policy before a CDN release process is selected.
- Record baseline metrics when the runtime exposes them, including startup time, frame pacing, resident memory, and eviction churn.
