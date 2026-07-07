# Operations

## Runtime Monitoring

Primary operational visibility currently comes from browser console logs.

- Startup errors: canvas lookup, window creation, renderer initialization.
- Manifest loader status: success summary or typed warning/error message.
- Render loop failures: surfaced to console and runtime exits event loop.
- Material/render diagnostics: per-batch material routing and blend-mode behavior can be validated visually and through render warnings.
- Simulation profiling: per-phase fixed-tick summaries for sync, composition, broadphase, narrowphase, reconcile, and render boundary costs.
- Collision scaling: candidate pair counts, collision counts, and despawn counts during stress runs.
- Fragmentation diagnostics: fragment spawn counts and first-tick despawn behavior near world bounds.

Recommended checks during runtime verification:
- Confirm no repeated surface/device errors in console.
- Confirm redraw/update/render loop remains active after window resize.
- Confirm manifest fetch resolves from served static root.
- Confirm material fallback behavior is stable when unknown material IDs are encountered.
- Confirm additive/alpha/opaque material passes appear as expected (no global blend regression).
- Confirm spatial broadphase reduces collision candidate pairs relative to all-pairs scanning.
- Confirm phase timings remain bounded under stress runs and do not concentrate in narrowphase unexpectedly.
- Confirm bullet->asteroid collisions produce expected fragment count and fragment entities persist at least one tick near boundaries.

Budget and streaming checks:
- Confirm periodic memory samples are emitted for CPU/WASM and GPU pools.
- Confirm residency stats are visible by class (`critical`, `scaled_optional`, `reused`, `streaming`).
- Confirm queue depth and upload latency remain within expected limits during movement across scenes.
- Confirm pressure-level transitions (`Normal`, `Constrained`, `Critical`) are logged with reason.
- Confirm fallback LOD activation occurs before optional high-LOD eviction causes visible stalls.

## Debugging

Common failure modes:
- `manifest.json` not found or blocked by incorrect serving path.
- Browser not supporting selected GPU backend path.
- Stale generated `docs/pkg` files after code changes.

Debug workflow:
1. Rebuild with wasm-pack release output.
2. Hard-refresh browser page.
3. Inspect console messages for startup and render diagnostics.
4. Validate static server root is `docs`.
5. Compare `pairs_checked` and phase timing logs before/after any simulation architecture change.

Memory pressure workflow:
1. Reproduce with constrained memory settings or high-content scenes.
2. Verify pressure transition logs and class-specific eviction reasons.
3. Confirm optional quality downgrade precedes forced eviction.
4. Validate recovery hysteresis prevents rapid load/unload thrash.

## Maintenance

- Keep Rust dependencies current with regular `cargo update` reviews.
- Re-run format/lint/build checks before merging.
- Keep docs synchronized whenever runtime contracts or commands change.
- Track milestone acceptance criteria in backlog as implementation evolves.
- Keep release playbooks in sync with CDN versioning/cache purge policy.
- Record baseline metrics per milestone (startup, frame pacing, resident memory, eviction churn).
