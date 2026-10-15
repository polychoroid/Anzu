# Operations

## Runtime Monitoring

The browser console provides the current runtime diagnostics.

- Check it for canvas, window, device, and rendering errors.
- Confirm that the texture demo appears after startup.
- Resize the window and confirm that rendering continues.
- Press `P` and confirm that the projection mode changes.

The active runtime does not yet expose frame or stage timing distributions, collision/broadphase work counts, asset-residency metrics, upload-queue metrics, or pressure metrics. Similar profiler code under `anzu-engine/src/abandoned_generated_code` is archived and is not evidence that the active runtime is instrumented.

### Performance Baseline Status

Performance monitoring is a current implementation priority, not a completed runtime capability. Until Feature 9.1 and Feature 9.6 are implemented, use this status when reviewing performance claims:

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
- The server cannot find `index.html` or generated package files because it uses the wrong root path.
- The browser does not support the selected GPU backend.
- The generated files in `docs/pkg` do not match the current code.

Debug workflow:
1. Build the release package with `wasm-pack`.
2. Hard-refresh the browser page.
3. Inspect the console for startup and rendering diagnostics.
4. Confirm that the static server root is `docs`.
5. Compare `pairs_checked` and phase-timing logs before and after each simulation architecture change.

## Planned Memory-Pressure Workflow

The active runtime does not implement this workflow. Use it as a test plan after memory budgets, asset eviction, and pressure telemetry exist.

1. Reproduce the issue with constrained memory settings or high-content scenes.
2. Check pressure-transition logs and class-specific eviction reasons.
3. Confirm that the runtime lowers optional quality before forced eviction.
4. Confirm that recovery hysteresis prevents rapid load and unload cycles.

## Maintenance

- Keep Rust dependencies current with regular `cargo update` reviews.
- Re-run format/lint/build checks before merging.
- Keep docs synchronized whenever runtime contracts or commands change.
- Track milestone acceptance criteria in backlog as implementation evolves.
- Keep release playbooks in sync with CDN versioning and cache-purge policy when CDN deployment is in use.
- Record baseline metrics when the runtime exposes them, including startup time, frame pacing, resident memory, and eviction churn.
