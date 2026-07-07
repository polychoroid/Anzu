# Design Consideration: ROMs as Separate WASM Assemblies

Date: 2026-07-06
Status: Accepted (topology-first)
Owner: Runtime/Platform

## Problem Framing

We want to ship a host "console" runtime as one wasm module and load game ROMs as separate wasm modules on demand.

Target host responsibilities:
- Input management
- Display/render loop
- Audio output/mixing
- Networking transport
- ROM loading and lifecycle
- Local storage and cache management

Target ROM responsibilities:
- Game-specific rules and simulation behavior
- Scenario logic and content-specific intent mapping

## Current-State Evidence (In Repo)

The current architecture compiles ROMs into the same wasm module and resolves them at startup with a static closure map.

Evidence:
- `anzu-engine/src/lib.rs`
  - Hardcoded ROM resolver map (`anzu.triangle_man`, `anzu.triangle_man_2`)
- `anzu-engine/src/platform_browser.rs`
  - Loads `manifest.json`, reads `rom_id`, then resolves through in-process closure
- `anzu-engine/src/rom.rs`
  - `RomPackage` trait is in-process and uses host references/trait objects
- `anzu-engine/src/renderer/core.rs`
  - `RenderRomPackage` returns static in-process render payload (`&'static` shader/vertex data)
- `anzu-engine/src/renderer/wasm.rs`
  - Runtime `State` owns both `World` and `SimulationModel` directly

Implication:
Current ROM contract is not plugin-style across wasm module boundaries.

## Recommendation

Adopt an explicit host/guest ABI and treat ROMs as sandboxed guests.

Host (console wasm):
- Owns frame lifecycle, rendering backend, input adapters, audio graph, network sockets, storage/caching, and ROM lifecycle.

Guest (ROM wasm):
- Owns ROM logic only.
- Interacts with host through typed messages/buffers, not shared references.

Key architecture shift:
- Replace trait-object coupling with a serialized ABI contract that supports versioning and compatibility checks.

## Strategy Update (2026-07-06)

Direction confirmed:
- MVP should include full core platform functionality as early as possible, not a reduced-scope architecture that requires later rework.
- Topology (ownership boundaries and service graph) must be established before ROM complexity grows.
- ROM integrity/hash protection is required from the start to support future commercial distribution.
- Asset loading, streaming, and cache/residency behavior must be designed and implemented early to avoid assumption debt.

Design intent:
- Spend early engineering credits on durable boundaries and contracts so later iterations focus on gameplay/content quality rather than structural refactors.

Non-goals for this phase:
- Deferring major host services (audio/network/storage/integrity) to a later retrofit.
- Shipping a "toy ABI" that is known to break once real ROMs arrive.

## Required Contracts

### 1) ROM ABI

Define a stable ABI with:
- ABI version
- Capability negotiation
- Lifecycle calls (`init`, `tick`, `serialize`, `deserialize`, `shutdown`)
- Host service requests (audio events, storage I/O, network envelopes)
- Structured error codes

### 2) Frame Boundary Ownership

Preserve explicit host frame boundaries:
1. Acquire surface texture
2. Execute fixed simulation staging
3. Extract render data from stable snapshot
4. Encode commands
5. Submit queue
6. Present
7. Emit profiling/diagnostics

### 3) Manifest v2

Extend startup metadata beyond `rom_id` to include:
- `rom_url`
- `abi_version`
- `integrity` (hash/signature material)
- `required_capabilities`
- `storage_namespace`
- Optional `fallback_rom_id`

### 4) Integrity and Trust Chain

ROM distribution contract:
- Every ROM artifact must include a cryptographic digest (at minimum SHA-256) validated by host before activation.
- Manifest must bind ROM identity to expected digest and ABI version.
- Host must hard-fail mismatches and never execute unverifiable ROM payloads.
- Cache entries must be keyed by content hash, not only by URL/version strings.

Forward-compatible extension:
- Allow future signed manifests or signed ROM metadata without breaking v2 field semantics.

## Browser Storage Guidance (2026)

Use a layered approach:

1. IndexedDB
- Primary structured storage for save data, metadata, compatibility tables.

2. Cache API
- Request/response artifact cache for ROM binaries and static packs.
- Version cache names and implement cleanup.

3. OPFS (Origin Private File System)
- Larger binary assets and high-write workloads.
- Useful for file-like or streaming content.

4. Storage API
- Use `navigator.storage.estimate()` for budget signals.
- Request persistence via `navigator.storage.persist()` when needed.

5. localStorage
- Keep for tiny preferences only.
- Avoid for ROM binaries or large save payloads.

Notes:
- Browser eviction policies are origin-scoped and browser-specific.
- Plan for `QuotaExceededError` and full-origin eviction recovery.

## Level of Work

Assessment: High effort, feasible.

Estimated effort for selected strategy (full-functionality MVP):

1. Topology-first MVP with full host services (ABI, loader, integrity/hash, storage/caching, audio, networking, asset pipeline, diagnostics)
- About 12-18 weeks with 2-3 engineers.

2. Post-MVP expansion (authoring SDK maturity, stronger policy controls, operational tooling, ROM catalog UX)
- About 6-10 additional weeks with 2-3 engineers.

Confidence:
- High on effort class (major architecture slice)
- Medium on exact timeline (depends on strictness of security posture, browser support floor, and networking scope)

## Full-Functionality MVP Scope (Required)

The MVP is complete only when all of the following are present:

1. Host/guest ABI and lifecycle
- Versioned ROM ABI contract with capability negotiation.
- Deterministic fixed-tick host loop with ROM `tick` integration.

2. ROM loading and integrity
- On-demand ROM fetch.
- Pre-execution digest verification.
- Hash-addressed cache and rollback-safe activation.

3. Asset loading and residency
- Manifest-driven asset dependency model.
- Cache + storage-backed residency with clear fallback behavior.
- Bounded per-frame upload policy and failure handling.

4. Storage and persistence
- IndexedDB/Cache/OPFS strategy implemented.
- Namespace isolation per ROM.
- Quota and eviction recovery paths.

5. Audio service boundary
- Host-owned audio graph and ROM audio event API.

6. Networking service boundary
- Host-owned transport API with reconnect/backpressure semantics.

7. Observability and safety gates
- ABI compatibility checks.
- Determinism replay checks.
- Integrity verification telemetry.
- Browser compatibility matrix and startup/offline smoke tests.

## Work Breakdown

1. ABI and ROM SDK foundation: 3-5 weeks
- Define stable host/guest contract
- Versioning and capability model
- Determinism invariants for fixed-step execution

2. Runtime loader/lifecycle: 2-3 weeks
- Fetch, instantiate, validate compatibility
- Failure containment, reset, unload policies

3. Storage and cache subsystem: 1-3 weeks
- Artifact cache versioning
- Save-data namespacing per ROM
- Eviction/recovery behavior

4. Audio boundary: 1-2 weeks
- Host-owned WebAudio graph
- ROM emits declarative audio events

5. Networking boundary: 2-4 weeks
- Host-owned websocket transport
- ROM uses transport-agnostic message API
- Reconnect/backpressure semantics

6. Tooling, CI, release hardening: 2-3 weeks
- ABI conformance tests
- Multi-browser/offline validation
- Packaging and integrity checks

## Alternatives

1. Single wasm binary plus data-driven ROM packs
- Lowest effort, fastest ship
- No true code isolation per ROM

2. JS plugin ROMs
- Easier dynamic loading
- Weaker determinism/perf envelope for this Rust-centric runtime

3. Separate wasm ROM modules (proposed)
- Best long-term isolation/extensibility
- Highest upfront architecture cost

## Key Risks

1. ABI churn causing ROM breakage and slow authoring.
2. Determinism regressions if host/guest ownership is unclear.
3. Startup latency without staged fetch/compile/cache.
4. Storage eviction and offline instability without robust recovery.
5. Security exposure from remote ROM loading without strict integrity/capability controls.

## Suggested Next Steps

1. Freeze ABI v1 and Manifest v2 schemas before runtime implementation.
2. Implement host skeleton with all service boundaries (input, render, audio, network, storage) even if initial implementations are minimal behind the same contract.
3. Implement ROM integrity pipeline (digest verification and hash-addressed cache) before dynamic ROM execution is enabled.
4. Ship one end-to-end ROM through the full path (fetch -> verify -> instantiate -> tick -> persist -> reload offline).
5. Add CI gates for ABI conformance, deterministic replay, integrity failures, and storage eviction recovery.

## Decision Summary

This is a quarter-scale architecture initiative, not a sprint-level refactor. The selected strategy is to build the final topology early with full-functionality MVP constraints, so later effort is focused on gameplay and content iteration rather than structural migration.
