# Anzu Engine Backlog

Virtual Console: A modular Rust/WASM game engine with data-driven architecture, remote ROM loading, authentication, and multiplayer support.

---

## Implementation Roadmap

### Phase 1: Minimal Runtime (Foundation)
Goal: Deliver a native prototype with core engine loop, ECS, and wgpu rendering.
Target: Single-threaded deterministic runtime, local test scenes, desktop validation.

### Phase 2: Data-Driven Engine (Game System)
Goal: ROM manifests, scene loading, data-driven behavior, local persistence.
Target: Load remote ROM definitions, move sprites via input, save/load local state.

### Phase 3: Networked Platform (Multiplayer)
Goal: Secure authentication, remote asset loading, multiplayer sync model.
Target: Authenticate user, fetch ROM from cloud storage, multiplayer state sync, live WebSocket transport.

### Phase 4: Full Virtual Console (Production)
Goal: WASM plugin modules, spatial audio, debug tools, cloud backends.
Target: Production-ready browser deployment, optional logic modules, complete observability.

### Cross-Cutting Implementation Policy (Applies to all unchecked tasks)
- Use typed `Result` errors for recoverable failures in runtime code paths.
- Avoid `unwrap`/`expect` in startup, rendering, networking, and asset-loading paths.
- Require measurable acceptance checks for each completed task (build/test/profile evidence).
- Keep deterministic simulation behavior isolated from I/O and rendering side effects.
- Establish final host/guest topology early; do not defer core service boundaries (input, render, audio, network, storage, ROM lifecycle) to late-stage retrofits.
- Keep physics response deterministic while supporting configurable restitution (elastic/inelastic) and bounded-world collisions.
- Enforce deny-by-default authz and per-request authorization checks for protected endpoints.
- Prefer documented extension seams (traits/modules/contracts) before adding complexity.
- Require ROM integrity verification (content hash minimum) before execution; never activate unverifiable ROM binaries.
- Treat asset loading and residency assumptions as day-0 architecture constraints, not post-content hardening work.
- Classify runtime resources (`critical`, `scaled_optional`, `reused`, `streaming`) and tie each class to explicit budget/eviction policy.
- Treat memory budgets as dynamic signals (CPU/WASM + GPU), with graceful quality fallback before hard failure.
- Keep streaming and upload work off the frame-critical path via priority queues and bounded per-frame upload budgets.
- Require fallback assets/LODs for streamable content so over-budget conditions degrade quality, not correctness.

## Epic: Runtime Execution Model

**Description**: Deterministic engine scheduler, ECS, world state, and frame lifecycle.

**Design constraint**: Simulation stage depends on a trait contract with swappable implementations rather than a mode enum.

### Milestone 1: Walking Skeleton—Browser-Hosted Engine with Moving Triangle
**Outcome**: A web client loads a WASM engine, initializes WebGPU, and renders a triangle that rotates each frame.

- [x] Task 1.1: Set up browser HTML shell with canvas element
- [x] Task 1.2: Configure Rust WASM build pipeline (wasm-pack, bundler target)
- [x] Task 1.3: Create wgpu Surface and initialize rendering context in WASM
- [x] Task 1.4: Implement basic event loop (winit or raw browser events)
- [x] Task 1.5: Create a simple triangle mesh with transform data
- [x] Task 1.6: Render triangle to canvas each frame (clear → render → present)
- [x] Task 1.7: Update triangle rotation based on elapsed time
- [x] Task 1.8: Test in browser and validate WebGPU output
- [x] Task 1.9: Document build steps and deployment (browser serve, CORS setup)
- [x] Task 1.10: Establish asset loading pattern (fetch manifests from server)
- [x] Task 1.11: Enforce frame contract `update(delta_time) -> render()` on every redraw
- [x] Task 1.12: Add triangle shader + render pipeline (vertex buffer, uniform buffer, bind group)
- [x] Task 1.13: Keep simulation state separate from GPU state (angle, angular velocity, elapsed time)
- [x] Task 1.14: Add acceptance checks (rotates for 10s, resize-safe, no startup panics)

**Demonstrates**: 
- WASM compilation and browser deployment works
- wgpu initializes correctly in WebGL/WebGPU context
- Frame timing and basic animation loop work
- Library choices validated (winit, wgpu, wasm-bindgen, bundler)

### Milestone 2: Add ECS Foundation and Multi-Entity Rendering
**Outcome**: Engine can spawn multiple entities (triangle, square, etc.), each with position/rotation components, updated by a simple scheduler.

- [x] Task 2.1: Implement minimal ECS with simulation trait contract boundary (Entity ID, Component trait, sparse storage)
- [x] Task 2.1a: Add compile-time boundary check: runtime depends on simulation trait interface, not concrete ROM simulation types
- [x] Task 2.2: Add Transform component (position, rotation, scale)
- [x] Task 2.3: Add Mesh component with vertex/index data
- [x] Task 2.4: Create simple scheduler to iterate entities and update transforms
- [x] Task 2.4a: Add collision/proximity components (AABB or circle bounds) for object interaction checks
- [x] Task 2.4b: Add deterministic interaction stage in scheduler (proximity + collision evaluation)
- [x] Task 2.4c: Emit typed entity interaction events (`Collision`, `ProximityEnter`, `ProximityExit`)
- [x] Task 2.4d: Add simple interaction response handler (state change or momentum impulse on contact)
- [x] Task 2.4e: Add deterministic interaction regression test (same entity ordering -> same event ordering)
- [x] Task 2.5: Spawn 3+ entities with different meshes and animations
- [x] Task 2.6: Render all entities in single batched call
- [x] Task 2.7: Add per-entity move/rotate logic
- [x] Task 2.8: Test performance with ~100 entities
- [x] Task 2.9: Add entity lifecycle (spawn/despawn) without leaks
- [x] Task 2.10: Implement flyweight pattern for shared mesh data (dedup vertex/index buffers across entities with same mesh)

**Demonstrates**:
- ECS patterns work for multi-entity games
- Scheduler ordering is correct
- Batching and GPU resource reuse work
- Entity spawn/despawn is leak-free
- Engine/ROM separation is enforced by trait/interface boundary rather than runtime mode branching

### Milestone 3: Input-Driven Movement and Deterministic Simulation
**Outcome**: Player can control a triangle with keyboard (WASD for movement, arrow keys for rotation). Simulation is deterministic (same input sequence = same output).

- [x] Task 3.1: Implement InputManager (capture keyboard events)
- [x] Task 3.2: Add input-to-action mapping (WASD → move, arrows → rotate)
- [x] Task 3.3: Create input event stage in scheduler
- [x] Task 3.4: Add fixed timestep simulation stage (60 Hz)
- [x] Task 3.5: Update transform based on input each frame
- [x] Task 3.6: Record input history and test replay (same result)
- [x] Task 3.7: Test determinism across multiple browser sessions
- [x] Task 3.8: Add frame-by-frame debug stepping (optional)

**Demonstrates**:
- Input abstraction works on browser
- Game logic can be driven by player actions
- Simulation is deterministic (foundation for multiplayer)
- Players feel responsive control

### Milestone 3.5: ROM-Owned Input and Peripheral Event Layer
**Outcome**: ROMs define their own input vocabulary and can receive keyboard, mouse, touch, gamepad, and future peripheral events through a generic engine event boundary.

- [x] Task 3.5.1: Define a ROM-facing generic input event shape that carries device/source, control identity, press/release state, and analog value data.
- [x] Task 3.5.2: Keep the browser adapter as a raw event forwarder that does not invent game actions or hardcode ROM input vocabularies.
- [x] Task 3.5.3: Let each ROM declare its own control schema and action mapping layer for keyboard and non-keyboard devices.
- [x] Task 3.5.4: Add one ROM example that interprets both keyboard input and a non-keyboard peripheral path without engine changes.
- [x] Task 3.5.5: Add regression checks to ensure new peripheral types can be introduced without modifying shared input enums.

**ROM input schema v1**:
```yaml
input_profile:
	profile_id: "triangle_man.default"
	actions:
		- action_id: "thrust_forward"
			kind: "digital"
		- action_id: "thrust_reverse"
			kind: "digital"
		- action_id: "turn_left"
			kind: "digital"
		- action_id: "turn_right"
			kind: "digital"
		- action_id: "fire"
			kind: "digital"
	bindings:
		- source: "keyboard"
			control: "KeyW"
			action_id: "thrust_forward"
			phase: "pressed"
		- source: "keyboard"
			control: "KeyS"
			action_id: "thrust_reverse"
			phase: "pressed"
		- source: "keyboard"
			control: "KeyA"
			action_id: "turn_left"
			phase: "pressed"
		- source: "keyboard"
			control: "KeyD"
			action_id: "turn_right"
			phase: "pressed"
		- source: "keyboard"
			control: "Space"
			action_id: "fire"
			phase: "pressed"
		- source: "gamepad"
			control: "south_button"
			action_id: "fire"
			phase: "pressed"
			device_index: 0
		- source: "mouse"
			control: "primary_button"
			action_id: "fire"
			phase: "pressed"
```

**Schema rules**:
- `action_id` is ROM-owned and may be any stable string; the engine does not maintain a shared action enum.
- `source` identifies the device family (`keyboard`, `mouse`, `touch`, `gamepad`, `sensor`, `custom`).
- `control` identifies the per-device control name or axis/button identifier as emitted by the adapter.
- `phase` defaults to `pressed` for digital inputs and may be `released`, `moved`, `changed`, or `held` when appropriate.
- `value` is optional for digital actions and required for analog axes or gesture-like inputs.
- `device_index` is optional and used when a ROM wants to distinguish multiple controllers of the same source.
- The engine should pass events through unchanged; ROMs may ignore unbound events, map one event to multiple actions, or map multiple controls to one action.

**Demonstrates**:
- ROMs own their control vocabulary
- Keyboard is just one device source among many
- Engine input handling stays generic and future-proof
- New peripherals do not require changing shared action enums

### Milestone 4: Hardened Simulation Pipeline and Spatial Broadphase
**Outcome**: A reusable tick pipeline composes world transforms and forces once per tick, profiles phase costs, and uses spatial indexing to keep collision checks efficient as entity counts grow.

- [x] Task 4.1: Define a tick pipeline contract that composes intent, applies transforms, runs physics, and reconciles results in explicit stages.
- [x] Task 4.2: Add per-phase profiling for sync, compose, broadphase, narrowphase, reconcile, and render boundary costs.
- [x] Task 4.3: Implement a spatial broadphase structure for candidate collision queries instead of all-pairs scanning.
- [x] Task 4.4: Keep collision response as narrowphase-only work after spatial and proximity filtering.
- [x] Task 4.5: Add regression tests for deterministic spatial query ordering and stable interaction results.
- [x] Task 4.6: Document the pipeline contract so ROMs can compose forces and transforms without extra per-system passes.

Progress note (2026-07-01): Slice A, Slice B, and Slice C are implemented in the scheduler with explicit stage timing/counter reporting, clarified compose/publish/reconcile boundaries, and a uniform-grid broadphase candidate feed. Validation included native + wasm `cargo check`, `cargo test simulation::tests:: -- --nocapture`, and `cargo test triangle_man::tests:: -- --nocapture`.

**Demonstrates**:
- Minimal tick passes and data movement
- Spatial indexing reduces collision work at scale
- Profiling identifies waste before new systems are added
- ROMs can plug into a hardened simulation pipeline

**Priority**: P0 (Foundation for scalable simulation)

### Milestone 4.5: Declarative Collision Outcomes and Fragmentation Rules
**Outcome**: Collision handling moves from role-specific hardcoded branches to a data-driven outcome model supporting damage, despawn/replace behavior, and physically consistent fragment spawning.

- [x] Task 4.5.1: Define collision outcome schema (`ignore`, `apply_damage`, `despawn`, `spawn_fragments`, `impulse_adjust`) keyed by role/tag pairs.
- [x] Task 4.5.2: Add deterministic damage/health integration path so collisions can reduce hit points without ad hoc role checks.
- [x] Task 4.5.3: Add despawn-and-replace flow allowing collision outcomes to spawn one or more fragment entities.
- [x] Task 4.5.4: Enforce linear momentum conservation when replacing a body with fragments; define bounded energy-loss policy via restitution.
- [x] Task 4.5.5: Add deterministic tests verifying identical input/collision ordering yields identical damage, despawn, and fragment outcomes.
- [x] Task 4.5.6: Add acceptance scenario: bullet-asteroid collision damages or fragments asteroid based on configured thresholds, with stable totals and no entity leaks.
- [x] Task 4.5.7: Add edge-case regression coverage for fragment spawn near world bounds and clamp fragment spawn positions to avoid first-tick out-of-bounds culling.

Status note: the live-body stress logs now show the broadphase ramp reaching a gameplay steady state rather than being pinned to the old low-30s plateau.

Progress note (2026-07-03): Added scheduler regression coverage for repeated-run deterministic interaction output under grid broadphase workloads, introduced declarative collision actions for Triangle Man (`ignore`, `apply_damage`, `despawn`, `spawn_fragments`, `impulse_adjust`), added deterministic health/damage integration for asteroid collisions, switched fragment replacement to restitution-bounded momentum retention, and added a collision acceptance test proving damage-before-fragment threshold behavior with stable entity totals.

Validation note (2026-07-03): Manual browser runs confirm asteroid health behavior works in both Firefox (WebGL path) and Chromium (WebGPU path), with observed runtime around ~75 fps.

**Demonstrates**:
- Collision behavior is configurable and scalable beyond hardcoded per-role logic
- Damage/despawn/fragment rules remain deterministic under fixed-timestep replay
- Fragmentation respects core physics invariants instead of producing arbitrary motion
- Fragment entities are independently simulated and survive edge-adjacent spawning scenarios

**Priority**: P0 (Closes gameplay collision-action gap)

### Milestone 4.6: Triangle Man 2 Platformer Scaffold
**Outcome**: A new ROM, `anzu.triangle_man_2`, boots a minimal platformer slice with one ground strip and one controllable triangle.

- [x] Task 4.6.1: Extract shared platformer helpers for mesh registration, render data, and locomotion state.
- [x] Task 4.6.2: Wire `triangle_man_2` through the ROM resolver and browser manifest.
- [x] Task 4.6.3: Implement first-slice movement with WASD, W jump, S crawl, traction, momentum, and ground contact.
- [ ] Task 4.6.4: Add browser acceptance coverage proving the scene boots and the triangle can crawl, move, and jump on the ground.

Progress note (2026-07-06): The first ROM slice is in place with shared platformer components, but browser-level acceptance still needs a focused run before the milestone is fully closed.

### Milestone 4.7: Triangle Man 3D Isometric Adventure
**Outcome**: A new ROM, `anzu.triangle_man_3d`, delivers an isometric adventure vertical slice with deterministic movement, camera framing, and interaction-ready scene composition.

- [ ] Task 4.7.1: Define ROM package scaffold and resolver wiring for `anzu.triangle_man_3d`.
- [ ] Task 4.7.2: Add isometric transform/extraction path compatible with existing render pipeline contracts.
- [ ] Task 4.7.3: Implement first scene with terrain plane, player avatar proxy, and collision-enabled walkable bounds.
- [ ] Task 4.7.4: Add input mapping for 8-direction movement projected into isometric world space.
- [ ] Task 4.7.5: Add deterministic camera follow behavior with bounded smoothing and fixed-step stability.
- [ ] Task 4.7.6: Add acceptance checks for spawn, traversal, collision boundaries, and stable fixed-tick behavior.

Planning note (2026-07-06): Milestone added to capture the next ROM line after Triangle Man 2 and to preserve deterministic runtime requirements while expanding to isometric 3D presentation.

### Milestone 5: Runtime Performance Baseline and Profiling Gates
**Outcome**: Runtime has explicit performance budgets and profiling checkpoints before major architectural expansion.

- [ ] Task 5.1: Define frame budget targets for browser MVP (frame time, startup time, memory ceiling)
- [ ] Task 5.2: Add startup and frame timing instrumentation (cold start, first frame, frame pacing)
- [ ] Task 5.3: Add lightweight render stats (draw calls, buffer uploads per frame)
- [ ] Task 5.4: Create performance regression checklist for each milestone
- [ ] Task 5.5: Require profiling evidence before introducing compute offload or complex scheduling
- [ ] Task 5.5a: Document and verify the frame boundary contract (`acquire -> encode -> submit -> present`), including surface `lost/suboptimal/reconfigure` handling expectations.
- [ ] Task 5.5b: Add boundary health instrumentation (`surface_reconfigure_count`, `present_fail_or_skip_count`, submit latency buckets, and frame pacing spike counters).

**Demonstrates**:
- Performance decisions are measured, not guessed
- Core loop remains stable as scope grows

**Priority**: P0 (Protects gameplay feel and iteration speed)

### Milestone 5.5: Material Definition and Registry Foundation
**Outcome**: Materials are defined as data and bound by ID so visual changes do not require vertex-structure changes.

- [x] Task 5.5.1: Define material definition shape (`material_id`, `blend_mode`, default parameters, fallback behavior)
- [x] Task 5.5.2: Implement renderer-side `MaterialRegistry` with deterministic fallback material resolution
- [x] Task 5.5.3: Bind entities to materials via `material_id` in the render path without changing geometry payload formats
- [x] Task 5.5.4: Route per-draw material lookup through render batches instead of hardcoded global blend state
- [ ] Task 5.5.5: Add acceptance checks proving entities can share geometry while rendering with different materials

**Demonstrates**:
- Material ownership is data-driven instead of hardcoded in geometry structs
- New visual styles can be introduced without renderer-wide vertex layout edits
- Missing/invalid materials degrade gracefully through deterministic fallback

**Priority**: P0 (Breaks geometry/effect coupling)

### Milestone 5.6: Material Parameters and Blend-Aware Pipelines
**Outcome**: Materials expose typed parameters and render through blend-aware pipelines selected per draw batch.

- [x] Task 5.6.1: Add typed material parameters (`base_color_tint`, `emissive_strength`, `metallic`, `roughness`, `specular_strength`)
- [x] Task 5.6.2: Implement per-batch material uniform packing with dynamic uniform buffer offsets
- [x] Task 5.6.3: Build and select blend-aware pipelines (`Opaque`, `Alpha`, `Additive`) from material definitions
- [x] Task 5.6.4: Emit and consume draw batches that carry material IDs and vertex ranges
- [ ] Task 5.6.5: Add profiling checks for material batch count, pipeline switches, and uniform upload cost

**Demonstrates**:
- Per-object visual controls vary without geometry duplication
- Blend behavior is material-driven instead of globally fixed
- PBR-lite shading knobs are available as material data, not shader constants

**Priority**: P0 (Enables scalable visual variety)

### Milestone 5.7: Composite Materials and Multi-Pass Composition
**Outcome**: Layered materials (base + emissive/outline/post) are composed via explicit passes without modifying core ECS or vertex types.

- [ ] Task 5.7.1: Define a compact composite material model supporting 2-3 chained passes
- [ ] Task 5.7.2: Implement deterministic render-graph pass ordering with explicit dependencies
- [ ] Task 5.7.3: Add emissive extraction + blur + add path for CRT-style glow/bloom
- [ ] Task 5.7.4: Bound intermediate targets and add quality fallback under memory/frame pressure
- [ ] Task 5.7.5: Add validation proving composite effects toggle per material without entity-structure changes

**Demonstrates**:
- Composite looks are achieved through pass composition, not vertex format growth
- Effect layering remains opt-in and bounded by runtime budgets
- Materials become a durable extension seam for ROM-specific rendering styles

**Priority**: P1 (Unlocks extensible visual effects)

### Milestone 5.75: Architecture Remediation—Adapter Boundary and Hot-Path Hardening
**Outcome**: Adapter files are reduced to host integration only, runtime/render extraction policy moves to platform-neutral core modules, and high-risk CPU/upload hot paths are instrumented with enforceable budgets.

- [ ] Task 5.75.1: Split responsibilities in `src/renderer/wasm.rs` so fixed-step orchestration, control-plane policy flow, and render extraction policy are owned by shared runtime/core modules.
- [ ] Task 5.75.2: Keep `src/platform_browser.rs` browser-only (window/canvas/event wiring) and remove any runtime-policy ownership from browser adapter paths.
- [ ] Task 5.75.3: Introduce a core render-extraction module with explicit static-vs-dynamic geometry paths and clear ownership contracts.
- [ ] Task 5.75.4: Add upload-budget instrumentation in render core (bytes/frame, dynamic write count, p50/p95/p99 upload cost) and wire warning thresholds.
- [ ] Task 5.75.5: Add regression checks proving core runtime behavior can run without browser-specific APIs (host parity contract checks at compile/test seam level).
- [ ] Task 5.75.6: Re-evaluate `dyn`/`Box` seams after boundary cleanup and replace incidental indirection where extension points are not real.
- [ ] Task 5.75.7: Add unified manual benchmark output that emits simulation-stage and render-extraction CSV metrics under one run label.

**Demonstrates**:
- Adapter-boundary violations are actively removed rather than documented only
- Runtime portability improves for non-browser wasm and future native hosts
- Extraction/upload bottlenecks are measured and budgeted before 3D scope expansion

**Priority**: P0 (Top priority: fixes confirmed design flaws before new feature expansion)

### Milestone 5.8: Engine Control Plane and Overlays (Core-First, Cross-Platform)
**Outcome**: Pause/menu, control, performance, and notification overlays are owned by engine runtime core (not ROM logic and not HTML-only), so browser and future desktop frontends share one control-plane implementation.

- [x] Task 5.8.1: Define shared control-plane command and overlay domain types in renderer core/shared module (`ControlCommand`, `OverlayPanel`, `OverlayVisibility`).
- [x] Task 5.8.2: Add shared configurable binding parser/default map for engine control commands (`Esc`, `F1`, `F2`, `F3`, `R`) with runtime override support.
- [x] Task 5.8.3: Intercept engine control commands before ROM input dispatch so pause/reset/control/perf/notification toggles are ROM-independent.
- [x] Task 5.8.4: Render overlays in-engine (main pass) so UI is not dependent on DOM dialogs.
- [x] Task 5.8.5: Move display/controller diagnostics from HTML diagnostics panel into engine control overlay.
- [x] Task 5.8.6: Provide minimal performance overlay (FPS) in-engine.
- [ ] Task 5.8.7: Provide notification overlay in-engine for control-plane and device events.
- [x] Task 5.8.8: Remove HTML-owned reset/game-over/controller HUD dialogs from `docs/index.html` after overlay parity.
- [x] Task 5.8.9: Define and document explicit input contexts (`GameplayContext`, `OverlayContext`) in control-plane contract, including command precedence and consumption rules.
- [x] Task 5.8.10: Enforce modal capture while overlay context is open (consume gameplay controls, allow overlay navigation/toggles, suppress pause-toggle side effects while overlay context is active).
- [x] Task 5.8.11: Add regression coverage for context gating across keyboard and controller paths (including pause suppression while overlay is visible and restored routing after overlay close).

Progress note (2026-07-03): Initial control-plane command/keybind handling is implemented and now anchored in shared renderer module `src/renderer/control_plane.rs` for cross-platform reuse. Runtime mutable control-plane state ownership has been moved into shared `ControlPlaneState` (core-first) and the wasm adapter now feeds key/controller/reset events into that shared state machine instead of keeping a wasm-local duplicate. In-engine overlay rendering has begun: pause menu, control, performance (with FPS bar), and notifications panels are now generated as geometry batches and rendered inside the existing GPU pass (not DOM-dependent).

Validation note (2026-07-03): Post-migration targeted suites remain green: `control_plane::tests` 8 passed, `simulation::tests` 5 passed, `triangle_man::tests` 13 passed. Overlay geometry is now wired and visible when toggles are active (F1: control, F2: perf, F3: notifications, Esc: pause).

Validation note (2026-07-05): Input-context modal capture and pause suppression are now enforced through shared control-plane + wasm routing gates. Added regression coverage for context restore and pause recovery after overlay close (`closing_overlay_restores_gameplay_context`, `pause_toggle_recovers_after_overlay_context_closes`). Targeted run: `cargo test renderer::control_plane::tests:: -- --nocapture` => 17 passed, 0 failed.

Validation note (2026-07-05): Input/display diagnostics are now engine-owned and rendered in the control scrim; HTML diagnostics HUD/toggle and remaining HTML game-over/reset overlays were removed from `docs/index.html`. Validation: `cargo check --target wasm32-unknown-unknown` passed after renderer diagnostics integration and shell cleanup.

Validation note (2026-07-05): Chromium and Firefox manual smoke runs confirm in-engine overlays are visible and functional in the same interaction order (pause -> control diagnostics scrim -> performance overlay) while gameplay continues rendering. Console output in both browsers shows expected no-gamepad warnings and control-context consume logs with no blocking runtime errors.

Acceptance note (input context): When overlay context is visible, gameplay input routing is blocked except explicit control-plane commands, and behavior remains deterministic and consistent across keyboard/controller paths.

**Demonstrates**:
- Control-plane behavior is engine-owned and frontend-agnostic
- Browser and desktop runtimes can reuse one command/binding model
- ROMs remain focused on gameplay while engine manages runtime UX controls

**Priority**: P0 (Required for desktop parity and maintainable runtime UX)

### Milestone 5.9: Font-Backed Engine Text Rendering
**Outcome**: Engine overlays and HUD text render from imported font data through an atlas-backed text path instead of hardcoded glyph tables, while keeping simulation determinism isolated from presentation work.

**Dependency note**: Build on Milestone 5.8 engine-owned overlays; do not reintroduce DOM-owned runtime UI for text presentation.

**Recommended references**:
- Preferred library: `glyphon` docs: <https://docs.rs/glyphon/latest/glyphon/>
- `glyphon` repository and examples: <https://github.com/grovesNL/glyphon>
- Concrete `glyphon` example (`hello-world.rs`): <https://github.com/grovesNL/glyphon/blob/main/examples/hello-world.rs>
- `wgpu` API reference: <https://docs.rs/wgpu/latest/wgpu/>
- `wgpu` graphics-work encapsulation guidance: <https://github.com/gfx-rs/wgpu/wiki/Encapsulating-Graphics-Work>
- Vulkan text-overlay reference sample (atlas/bitmap path): <https://github.com/SaschaWillems/Vulkan/tree/master/examples/textoverlay>

- [ ] Task 5.9.1: Define an engine-owned text rendering seam that separates string/layout preparation from render-pass encoding.
- [ ] Task 5.9.2: Replace the hardcoded glyph-table overlay path with atlas-backed font rendering for engine overlays and HUD text only.
- [ ] Task 5.9.3: Start with one embedded font and a constrained character-set policy suitable for current diagnostics, pause/menu, and notification overlays.
- [ ] Task 5.9.4: Keep text render-only so font loading, layout, and caching do not affect deterministic simulation behavior or replay outcomes.
- [ ] Task 5.9.5: Add profiling and observability for glyph atlas preparation cost, cache growth, first-use hitch behavior, and text draw-call/batch impact.
- [ ] Task 5.9.6: Add acceptance checks for readability and stable behavior across Chromium WebGPU, Firefox WebGL fallback, resize events, and browser scale-factor changes.

**First-slice exclusions**:
- No multi-font fallback or localization commitment in this slice.
- No complex-script shaping requirement in this slice.
- No ROM-authored text widget/layout system in this slice.
- No manifest-driven font streaming or residency policy in this slice.

**Demonstrates**:
- Engine text quality improves without expanding gameplay/simulation scope.
- Overlay text remains engine-owned and cross-platform.
- Font integration is measurable and bounded before broader UI/content ambitions.

**Priority**: P1 (Improves runtime UX after overlay ownership is established)

### Milestone 5.10: Topology-First Console/ROM Split (Full-Functionality MVP Gate)
**Outcome**: Runtime host/guest topology is established early with full service boundaries, ROM integrity verification, and manifest compatibility checks so future ROM/content growth does not force architectural rework.

- [ ] Task 5.10.1: Freeze ROM ABI v1 and host service contract (input, render extraction, audio events, network envelopes, storage calls, lifecycle hooks).
- [ ] Task 5.10.2: Add manifest v2 schema (`rom_url`, `abi_version`, `integrity`, `required_capabilities`, `storage_namespace`, optional fallback ROM metadata).
- [ ] Task 5.10.3: Implement ROM loader path (`fetch -> verify -> instantiate -> activate`) with hard-fail behavior on ABI/integrity mismatch.
- [ ] Task 5.10.4: Implement hash-addressed ROM cache/index (content hash keyed, not URL-only) with rollback-safe activation semantics.
- [ ] Task 5.10.5: Integrate storage namespace isolation per ROM for saves/cache metadata and validate quota/eviction recovery path.
- [ ] Task 5.10.6: Add deterministic replay checks proving host/guest boundary does not regress fixed-tick determinism.
- [ ] Task 5.10.7: Add acceptance scenario for one dynamic ROM loaded through full path with offline cache reload.

**Demonstrates**:
- Final topology is established before ROM complexity increases
- ROMs are integrity-protected from day one
- Core service boundaries are stable enough for long-lived iteration

**Priority**: P0 (Prevents late structural refactor risk)

### Milestone 5.10a: In-Game Multi-ROM Menu and Runtime ROM Switching
**Outcome**: Players can open an in-game ROM menu, view available ROMs, and switch/load multiple ROMs without leaving the runtime shell.

- [ ] Task 5.10a.1: Define ROM catalog model for menu consumption (`rom_id`, display name, version, source, integrity/hash status, capability flags).
- [ ] Task 5.10a.2: Add engine-owned ROM menu overlay panel integrated with control-plane navigation/input contexts.
- [ ] Task 5.10a.3: Implement ROM lifecycle transitions (`prepare`, `deactivate current`, `activate new`, `rollback on failure`) with deterministic state reset semantics.
- [ ] Task 5.10a.4: Enforce ROM integrity and compatibility checks before activation in menu-triggered loads.
- [ ] Task 5.10a.5: Add loading/error states in overlay UX (pending, success, failure, reason) with non-blocking fallback behavior.
- [ ] Task 5.10a.6: Add acceptance coverage for switching across at least 2 ROMs in one session, including failure recovery to previous active ROM.

**Demonstrates**:
- In-game ROM discovery/switching is a first-class runtime capability
- ROM activation remains safe (integrity/compatibility) under user-driven switching
- Control-plane UX supports feature growth without DOM-owned menu dependencies

**Priority**: P0 (Stakeholder feature + platform capability)

### Milestone 5.11: Early Audio Service Vertical Slice (Sine/Beep-Boop)
**Outcome**: Engine owns a cross-platform audio service boundary early, validated with a minimal synthesized tone path so resource and lifecycle constraints are known before content/audio complexity arrives.

- [ ] Task 5.11.1: Define `AudioService` contract (init/shutdown, voice lifecycle, one-shot events, per-frame budget counters).
- [ ] Task 5.11.2: Implement minimal host-owned audio backend (WebAudio in browser path) with explicit unlock/resume handling.
- [ ] Task 5.11.3: Add synthesized sine-wave generator and two one-shot cues (`beep`, `boop`) driven through ROM-facing audio events.
- [ ] Task 5.11.4: Add runtime safeguards (voice limits, clipping guard, fallback to silence on backend errors).
- [ ] Task 5.11.5: Add observability for callback underruns, active voice count, and audio timing drift versus fixed tick.
- [ ] Task 5.11.6: Add acceptance checks across Chromium and Firefox (start, cue playback, suspend/resume tab behavior, no crash on context loss).

**Demonstrates**:
- Audio ownership is engine-level and independent of ROM internals
- Early resource constraints are visible before full content audio integration
- ROMs can emit declarative audio intent without binding to backend specifics

**Priority**: P0 (Establishes early audio constraints and service topology)

### Milestone 6: Session Support & Basic Multiplayer (Shared Sessions)
**Outcome**: Multiple browser clients can join the same session and see each other's triangle positions updated in (near) real-time. This milestone proves session lifecycle, basic transport, and authoritative state propagation.

- [ ] Task 6.1: Define `SessionProtocol` (join, leave, broadcast, authoritative state update)
- [ ] Task 6.2: Implement a LocalSessionHost (in-process or Node/warp dev server) for rapid testing
- [ ] Task 6.3: Add client-side session join/leave APIs (WASM + JS glue)
- [ ] Task 6.4: Implement authoritative position sync for player-controlled entities (server authoritative with client updates)
- [ ] Task 6.5: Add basic latency compensation on clients (interpolation of remote entities)
- [ ] Task 6.6: Implement simple concurrency rules for entity interactions (e.g., collision or basic 'hit' event)
- [ ] Task 6.7: Test with multiple browser instances (2+) and verify state convergence
- [ ] Task 6.8: Add minimal security: session tokens and scoped auth for join requests
- [ ] Task 6.9: Document session API and example integration (how to host a session)

**Demonstrates**:
- Multi-user connection lifecycle (join/leave)
- Shared world state propagation and reconciliation
- Latency handling patterns (interpolation) and simple authoritative model
- Security posture for session join (token-based)

**Priority**: P0 (Enables early multi-user testing and session validation)

---

## Epic: Game Logic Execution

**Description**: Data-driven behavior engine with plugin extension points.

### Milestone 1: Event System and Rule Schema
- [ ] Task 1.1: Design event bus and message dispatch system
- [ ] Task 1.2: Implement rule schema format (JSON/RON)
- [ ] Task 1.3: Create rule interpreter for simple condition/action blocks
- [ ] Task 1.4: Add event trigger registration system
- [ ] Task 1.5: Test event propagation and ordering

### Milestone 2: Data-Driven State Machine Execution
- [ ] Task 2.1: Define state machine schema
- [ ] Task 2.2: Implement state transition interpreter
- [ ] Task 2.3: Support guard conditions and action sequences
- [ ] Task 2.4: Add behavior examples (patrol, attack, idle)
- [ ] Task 2.5: Test deterministic state transitions

### Milestone 3: Plugin Adapter for WASM or Scripting Fallback
- [ ] Task 3.1: Define plugin interface and module lifecycle
- [ ] Task 3.2: Add WASM module loader and instantiation
- [ ] Task 3.3: Implement Rhai scripting fallback for data-driven rules
- [ ] Task 3.4: Support safe function exports and callbacks
- [ ] Task 3.5: Test plugin hot-reload (optional)

**Priority**: P1 (Enables ROM-defined behaviors)

---

## Epic: Asset Pipeline

**Description**: ROM formats, asset dependency graph, streaming, caching, versioning.

### Milestone 1: Classified Asset Loading and Residency Baseline
**Outcome**: Engine loads manifest-defined textures/meshes with explicit resource classification, async staged loading, and residency-aware handles.

- [ ] Task 1.1: Extend manifest schema with runtime loading metadata (`resource_class`, `priority_tier`, `residency_hint`, `streaming_unit`, `fallback_chain`, `hash`)
- [ ] Task 1.2: Implement `AssetManager` residency model (`Queued`, `Loading`, `Resident`, `EvictionPending`, `Evicted`) with per-class memory accounting
- [ ] Task 1.3: Implement staged texture pipeline (`fetch` -> `decode` -> `upload` -> `activate`) with cancelation support
- [ ] Task 1.4: Implement staged mesh pipeline with placeholder LOD activation before full-quality readiness
- [ ] Task 1.5: Author example manifest that includes per-asset class/priority/fallback metadata
- [ ] Task 1.6: Integrate residency-aware manifest loading into scene instantiation
- [ ] Task 1.7: Bind active resident assets in render pass and fallback to lower LOD when high LOD is unavailable
- [ ] Task 1.8: Add typed error handling for missing assets, hash mismatch, and invalid class metadata
- [ ] Task 1.9: Define profiling gate for startup/frame/memory budgets and verify with multi-asset scene
- [ ] Task 1.10: Add cache/index layer keyed by content hash + version, not URL alone

**Demonstrates**:
- Manifest-driven loading is budget-aware
- Asset lifecycle and residency transitions are explicit
- Graceful fallback works under partial availability
- Startup/perf gates exist before scale-up

### Milestone 1.5: Real ROM Manifest and Content Templates
**Outcome**: Engine loads a ROM manifest with entry scene, controller maps, and entity templates. Runtime instantiates interactive entities from ROM content declarations.

- [ ] Task 1.5.1: Define ROM manifest schema v1 (`rom_id`, `version`, `entry_scene`, `assets`, `scenes`, `controller_maps`, `entity_templates`)
- [ ] Task 1.5.2: Add sprite/animation asset types and metadata conventions to manifest schema
- [ ] Task 1.5.2a: Define a later manifest-managed font asset shape (`font_id`, source/hash metadata, intended usage, fallback chain) for engine and ROM text once residency/versioning infrastructure exists.
- [ ] Task 1.5.3: Define scene reference records and controller-map reference records
- [ ] Task 1.5.4: Define entity template schema linking assets, interaction config, and controller maps
- [ ] Task 1.5.5: Implement ROM manifest parser validation for required fields and duplicate IDs
- [ ] Task 1.5.6: Integrate ROM entity-template instantiation into scene loading
- [ ] Task 1.5.7: Add ROM validator checks (missing references, empty IDs, hash presence for remote assets)
- [ ] Task 1.5.8: Add integration test loading a non-trivial ROM (multiple assets + interactive entities)
- [ ] Task 1.5.9: Add manifest compatibility/version gate checks

**Demonstrates**:
- ROM content is first-class (not example-only)
- Interactive object definitions can be authored as data
- Runtime content loading supports scenes + controls + entity templates
- Validation catches broken ROM definitions before runtime crashes

### Milestone 2: Dependency Graph and Asset Preloading
- [ ] Task 2.1: Build asset dependency resolver (e.g., mesh depends on texture)
- [ ] Task 2.2: Implement topological sort for correct load ordering
- [ ] Task 2.3: Add batch preload for manifest assets
- [ ] Task 2.4: Support explicit preload/lazy/stream directives with class-specific defaults
- [ ] Task 2.5: Test circular dependency detection
- [ ] Task 2.6: Measure load time, peak memory, and residency churn under preload
- [ ] Task 2.7: Add preload/stream progress reporting (bytes, queued jobs, resident set)

### Milestone 2.5: Cell Streaming and Adaptive LOD
**Outcome**: Runtime streams scene cells/zones and dynamically selects LOD based on visibility and memory pressure.

- [ ] Task 2.5.1: Define cell/zone schema and spatial partition metadata for scenes
- [ ] Task 2.5.2: Implement cell activation policy (visible set + neighbor ring prefetch)
- [ ] Task 2.5.3: Implement LOD selector using distance + pressure-driven quality caps
- [ ] Task 2.5.4: Add hysteresis policy to prevent load/unload thrash near cell boundaries
- [ ] Task 2.5.5: Validate seamless cell transitions with no hard visual pop/freeze
- [ ] Task 2.5.6: Benchmark transition latency, upload burst size, and resident memory stability

### Milestone 3: Budget Hierarchy, Eviction, and Versioned Delivery
- [ ] Task 3.1: Implement budget tree (`global` -> `cpu_wasm`/`gpu` -> per-class pools) with runtime tuning hooks
- [ ] Task 3.2: Implement class-specific eviction strategies and pressure callbacks (LOD drop, staged eviction, forced reclaim)
- [ ] Task 3.3: Support partial invalidation and live replacement by class/resource scope
- [ ] Task 3.4: Add detailed diagnostics (resident ratio, eviction reason, queue depth, upload latency)
- [ ] Task 3.5: Test dynamic budget changes and verify graceful degradation under pressure
- [ ] Task 3.6: Validate eviction safety for active scenes and in-flight dependencies

### Milestone 4: CDN-Aware Content Versioning and Fallback
**Outcome**: Runtime can consume versioned remote content with safe swaps and origin fallback policy.

- [ ] Task 4.1: Define versioned CDN URL policy and manifest compatibility contract
- [ ] Task 4.2: Implement stale detection using content hash/version and staged replacement
- [ ] Task 4.3: Add fallback chain support (primary CDN -> secondary mirror -> local cache)
- [ ] Task 4.4: Add cache-control and purge playbook checks to release process
- [ ] Task 4.5: Validate live update path without startup regressions or frame spikes

**Priority**: P0 (Enables ROM loading and asset management)

---

## Epic: Scene / World System

**Description**: Scene graph, entity templates, loading/unloading, serialization.

### Milestone 1: Scene Definition Format and Loader
- [ ] Task 1.1: Define scene schema with entity templates
- [ ] Task 1.2: Implement scene parser from JSON/RON
- [ ] Task 1.3: Support entity prefab instantiation from templates
- [ ] Task 1.4: Add scene graph traversal and querying
- [ ] Task 1.5: Test scene validation and error handling

### Milestone 2: Dynamic Scene Switching and Entity Instancing
- [ ] Task 2.1: Implement scene load/unload lifecycle
- [ ] Task 2.2: Support seamless entity spawn/despawn
- [ ] Task 2.3: Add cleanup and resource release on scene exit
- [ ] Task 2.4: Test frame consistency during transitions
- [ ] Task 2.5: Benchmark load times and memory usage

### Milestone 3: Save/Load and Partial Scene Restore
- [ ] Task 3.1: Implement world state serialization
- [ ] Task 3.2: Add scene-scoped save slots
- [ ] Task 3.3: Support partial scene revert (reload from save)
- [ ] Task 3.4: Test versioning and migration on load
- [ ] Task 3.5: Document save format stability guarantees

**Priority**: P1 (Enables game progression and state management)

---

## Epic: Authentication & Secure Data Access

**Description**: Browser token auth and secure ROM/asset retrieval.

### Milestone 1: Mock Auth Provider and Bearer Token Plumbing
- [ ] Task 1.1: Design AuthProvider trait and token struct
- [ ] Task 1.2: Implement MockAuthProvider for local testing
- [ ] Task 1.3: Add token acquisition and refresh flow
- [ ] Task 1.4: Integrate token into secure fetch wrapper
- [ ] Task 1.5: Test token expiry handling and refresh

### Milestone 2: Secure Fetch Wrapper for ROM and Assets
- [ ] Task 2.1: Implement HTTP fetch wrapper with Bearer header injection
- [ ] Task 2.2: Add CORS-compatible request construction
- [ ] Task 2.3: Support asset integrity checking (hash validation)
- [ ] Task 2.4: Add retry logic and timeout handling
- [ ] Task 2.5: Test with mocked remote endpoints

### Milestone 3: Azure-Compatible Auth Provider Interface
- [ ] Task 3.1: Design Azure-compatible token acquisition
- [ ] Task 3.2: Implement Azure AD / OAuth bridge
- [ ] Task 3.3: Support token caching and refresh via JS host
- [ ] Task 3.4: Add scope-based token requests
- [ ] Task 3.5: Test integration with Azure Blob Storage

**Priority**: P1 (Enables secure remote asset loading)

### Security Baseline Gate (Apply before production auth rollout)
- [ ] Task S1: Enforce deny-by-default access policy for protected engine endpoints
- [ ] Task S2: Validate authorization on every request (no client-only checks)
- [ ] Task S3: Define and test least-privilege policy for roles/attributes
- [ ] Task S4: Add auth/authz failure logging with stable event schema
- [ ] Task S5: Add authorization regression tests for IDOR and privilege escalation paths

**Reference baseline**: OWASP Authentication and Authorization Cheat Sheets

---

## Epic: Networking / Multiplayer Layer

**Description**: Transport abstraction, state sync, latency handling.

### Milestone 1: Abstract Transport and Local Loopback
- [ ] Task 1.1: Define NetworkTransport trait and message types
- [ ] Task 1.2: Implement LocalLoopbackTransport for testing
- [ ] Task 1.3: Add message serialization (bincode/serde)
- [ ] Task 1.4: Support async send/receive operations
- [ ] Task 1.5: Test deterministic message ordering

### Milestone 2: Client/Server Sync Model with Snapshots
- [ ] Task 2.1: Define snapshot replication model
- [ ] Task 2.2: Implement server-authoritative state updates
- [ ] Task 2.3: Add client-side prediction and rollback
- [ ] Task 2.4: Support latency interpolation for smooth movement
- [ ] Task 2.5: Test network lag and packet loss scenarios

### Milestone 3: Live WebSocket/WebRTC Adapter and Interpolation
- [ ] Task 3.1: Implement WebSocket transport adapter
- [ ] Task 3.2: Add WebRTC data channel fallback
- [ ] Task 3.3: Support encrypted messaging (TLS via browser)
- [ ] Task 3.4: Implement frame-based interpolation
- [ ] Task 3.5: Test live multiplayer with multiple clients

**Priority**: P2 (Enables multiplayer gameplay)

### Transport Performance Baseline (Pre-implementation)
- [ ] Task N1: Define transport abstraction boundary and message reliability classes
- [ ] Task N2: Build loopback latency/jitter benchmark harness
- [ ] Task N3: Compare WebSocket baseline with QUIC candidate using identical payload tests
- [ ] Task N4: Define snapshot interpolation buffer policy and test packet-loss behavior
- [ ] Task N5: Add transport-level observability (send rate, retransmit/skip, end-to-end latency)

**Reference baseline**: Gaffer On Games (timestep/snapshot interpolation), Tokio, Quinn

### GPU Offload Roadmap Gate
- [ ] Task G1: Maintain CPU/GPU workload inventory (what can move to compute)
- [ ] Task G2: Add profiling threshold for offload candidacy (CPU hotspot, data movement cost)
- [ ] Task G3: Prototype one compute workload only after threshold is met
- [ ] Task G4: Record throughput, latency, and power tradeoffs versus CPU path
- [ ] Task G5: Keep fallback CPU implementation until GPU path is verified stable

**Reference baseline**: gpuweb, wgpu examples, WebGPU Fundamentals

---

## Epic: Persistence System

**Description**: Save/load architecture for browser-local and cloud storage.

### Milestone 1: Local Save/Load with Browser Storage
- [ ] Task 1.1: Design SaveState schema
- [ ] Task 1.2: Implement IndexedDB or LocalStorage backend
- [ ] Task 1.3: Support multiple save slots
- [ ] Task 1.4: Add save compression (optional)
- [ ] Task 1.5: Test save/load roundtrip integrity

### Milestone 2: Cloud Sync Adapter Stub
- [ ] Task 2.1: Design cloud persistence trait
- [ ] Task 2.2: Implement Azure Blob backend stub
- [ ] Task 2.3: Add conflict resolution (client/server wins)
- [ ] Task 2.4: Support incremental sync
- [ ] Task 2.5: Test offline-first scenarios

### Milestone 3: Versioned State Snapshots and Restore
- [ ] Task 3.1: Implement snapshot versioning scheme
- [ ] Task 3.2: Add migration path for state format changes
- [ ] Task 3.3: Support snapshot diffing for bandwidth
- [ ] Task 3.4: Add rollback to older snapshot
- [ ] Task 3.5: Test version compatibility across releases

**Priority**: P1 (Enables save/load gameplay)

---

## Epic: Input System (Advanced)

**Description**: Unified action mapping across keyboard, mouse, touch, gamepad.

### Milestone 1: Base Input Manager and Action Map
- [ ] Task 1.1: Design input abstraction and ActionMap schema
- [ ] Task 1.2: Implement basic keyboard/mouse capture
- [ ] Task 1.3: Create action binding system
- [ ] Task 1.4: Add event deduplication and debouncing
- [ ] Task 1.5: Test input event routing

### Milestone 2: Keyboard/Mouse/Gamepad/Touch Support
- [ ] Task 2.1: Add gamepad input via browser Gamepad API
- [ ] Task 2.2: Implement touch event handling (mobile)
- [ ] Task 2.3: Support axis inputs (analog sticks, triggers)
- [ ] Task 2.4: Add haptic feedback hooks (optional)
- [ ] Task 2.5: Test multi-device simultaneous input

### Milestone 3: Remapping UI and Runtime Profile
- [ ] Task 3.1: Design input remap configuration format
- [ ] Task 3.2: Implement runtime keybinding changes
- [ ] Task 3.3: Add input profile persistence
- [ ] Task 3.4: Support accessibility input modes
- [ ] Task 3.5: Test profile switching in-game

**Priority**: P1 (Enables flexible game control)

---

## Epic: Audio System

**Description**: Web Audio playback, streaming, spatialization.

### Milestone 1: Audio Manager and Playback API
- [ ] Task 1.1: Design AudioManager and AudioSource trait
- [ ] Task 1.2: Implement Web Audio API integration
- [ ] Task 1.3: Support basic playback (play, stop, loop)
- [ ] Task 1.4: Add volume and pan controls
- [ ] Task 1.5: Test audio load and playback timing

### Milestone 2: Streamed Audio Asset Support
- [ ] Task 2.1: Add streaming audio loader
- [ ] Task 2.2: Support audio format handling (WAV, OGG, MP3)
- [ ] Task 2.3: Implement buffer management for streams
- [ ] Task 2.4: Add seek and playback position tracking
- [ ] Task 2.5: Test streaming reliability and memory usage

### Milestone 3: Simple Spatial Audio in Engine
- [ ] Task 3.1: Implement 3D audio positioning
- [ ] Task 3.2: Add distance-based attenuation
- [ ] Task 3.3: Support directional panning
- [ ] Task 3.4: Add Doppler effect (optional)
- [ ] Task 3.5: Test spatial audio in test scenes

**Priority**: P2 (Enhances game immersion)

---

## Epic: Task / Async System

**Description**: Non-blocking WASM execution and job scheduling.

### Milestone 1: Browser Async Task Queue and Asset Loading
- [ ] Task 1.1: Design task queue and job scheduler
- [ ] Task 1.2: Implement async/await integration
- [ ] Task 1.3: Add priority upload/stream task queue (`critical_stream`, `normal_decode`, `background_cleanup`)
- [ ] Task 1.4: Support task cancellation
- [ ] Task 1.5: Test non-blocking load during gameplay with bounded per-frame upload bytes

### Milestone 2: Job Scheduling for Background Loads
- [ ] Task 2.1: Implement priority scheduling with starvation protection and per-tier budgets
- [ ] Task 2.2: Add time-slicing to prevent frame drops
- [ ] Task 2.3: Support job dependencies
- [ ] Task 2.4: Add load balancing hints and upload batch coalescing
- [ ] Task 2.5: Test frame rate stability under load and queue backpressure scenarios

### Milestone 3: Runtime Task Diagnostics and Throttling
- [ ] Task 3.1: Add task queue metrics and statistics
- [ ] Task 3.2: Implement per-frame time budget and throttling
- [ ] Task 3.3: Support task profiling and visualization
- [ ] Task 3.4: Add adaptive quality scaling based on load
- [ ] Task 3.5: Test performance under memory pressure

**Priority**: P1 (Enables smooth async loading)

---

## Epic: Debugging & Observability

**Description**: Logging, profiling, debug overlays, runtime inspection.

### Milestone 1: Structured Logging and Tracing Hooks
- [ ] Task 1.1: Implement tracing/log integration
- [ ] Task 1.2: Add event logging for gameplay events
- [ ] Task 1.3: Support runtime log level filtering
- [ ] Task 1.4: Add performance markers (perf.mark)
- [ ] Task 1.5: Test log output in browser console

### Milestone 2: Debug Overlay and Frame Stats
- [ ] Task 2.1: Design debug overlay UI (FPS, memory, draw calls)
- [ ] Task 2.2: Implement frame time graph visualization
- [ ] Task 2.3: Add entity/component statistics display
- [ ] Task 2.4: Support toggle via keyboard shortcut
- [ ] Task 2.5: Test overlay performance impact

### Milestone 3: Runtime Inspection Tool/Profiling Markers
- [ ] Task 3.1: Add entity inspector (list entities, view components)
- [ ] Task 3.2: Implement system profiler (per-stage timings)
- [ ] Task 3.3: Support asset browser and memory viewer
- [ ] Task 3.4: Add remote debug protocol (optional)
- [ ] Task 3.5: Test inspection during game execution

**Priority**: P2 (Enables effective debugging)

---

## Epic: Platform Abstraction Layer

**Description**: Separation of browser and native behavior.

### Milestone 1: Dual-Target Build Works (Native + WASM produce identical output)
**Outcome**: Engine compiles to both native binary and WASM. Same scene produces identical frames.

- [ ] Task 1.1: Set up Cargo.toml with platform features and target selection
- [ ] Task 1.2: Create PlatformAdapter trait (window, input, rendering surface)
- [ ] Task 1.3: Implement BrowserPlatform adapter using web-sys and wasm-bindgen
- [ ] Task 1.4: Implement NativePlatform adapter using winit and native wgpu
- [ ] Task 1.5: Unify wgpu initialization behind adapter (same Surface, Device, Queue setup)
- [ ] Task 1.6: Configure feature flags for compile-time platform selection
- [ ] Task 1.7: Build and run native binary on desktop (moving triangle)
- [ ] Task 1.8: Build and deploy WASM to browser (same moving triangle)
- [ ] Task 1.9: Compare frame output bit-for-bit (validate determinism)
- [ ] Task 1.10: Document build commands for both targets

**Demonstrates**:
- Code can target both platforms without duplication
- Rendering is deterministic across platforms
- Build pipeline supports both targets
- Feature flags work correctly

### Milestone 2: Unify Common Runtime Logic Under Adapter Boundary
- [ ] Task 2.1: Extract platform-agnostic engine core (ECS, scheduler, physics)
- [ ] Task 2.2: Move I/O adapters (asset loading, persistence) behind traits
- [ ] Task 2.3: Add platform-specific window management (resize, fullscreen)
- [ ] Task 2.4: Support platform-specific debug output (console vs browser console)
- [ ] Task 2.5: Test that both platforms share 99%+ of logic
- [ ] Task 2.6: Validate that platform-specific code is <10% of total

### Milestone 3: Browser-First Build and Desktop Dev Harness
- [ ] Task 3.1: Create native dev harness for rapid iteration
- [ ] Task 3.2: Add WASM build tooling and npm integration
- [ ] Task 3.3: Support both targets in single cargo workspace
- [ ] Task 3.4: Document platform-specific limitations and capabilities
- [ ] Task 3.5: Test build reproducibility across clean environments

**Priority**: P0 (Enables platform flexibility)

---

## Epic: Data Serialization Strategy

**Description**: Define ROM/scene/config schema, format tradeoffs.

### Milestone 1: Load Scene Definition from JSON and Render It
**Outcome**: Engine loads a JSON file defining a scene (entities, meshes, transforms, animations). Renders the exact scene from that file.

- [ ] Task 1.1: Design minimal scene schema (entities, components as JSON)
- [ ] Task 1.2: Create JSON parser using serde
- [ ] Task 1.3: Map JSON structure to engine ECS entities
- [ ] Task 1.4: Define entity template format (reusable archetypes)
- [ ] Task 1.5: Write example scene JSON (triangle + square with animations)
- [ ] Task 1.6: Implement scene loader that reads JSON and instantiates entities
- [ ] Task 1.7: Verify scene renders identically to hard-coded version
- [ ] Task 1.8: Add error reporting for malformed JSON
- [ ] Task 1.9: Test with multiple scene files

**Demonstrates**:
- JSON serialization format works end-to-end
- Data-driven content is achievable
- Scene definitions are portable
- Parser and schema are correct

### Milestone 2: Add Schema Validation and Optional Binary Variant
- [ ] Task 2.1: Create JSON schema (.schema.json) for validation
- [ ] Task 2.2: Implement manifest versioning (v1.0, v2.0, etc.)
- [ ] Task 2.3: Design and implement binary format (bincode or similar)
- [ ] Task 2.4: Support format auto-detection (JSON vs binary)
- [ ] Task 2.5: Test schema evolution (add/remove/rename fields)
- [ ] Task 2.6: Benchmark: JSON vs binary load time and size
- [ ] Task 2.7: Validate that binary only outweighs JSON for >100KB scenes

### Milestone 3: Snapshot Serialization for Save/Multiplayer Rollback
- [ ] Task 3.1: Define snapshot serialization format (world state dump)
- [ ] Task 3.2: Implement deterministic entity serialization
- [ ] Task 3.3: Support incremental snapshots (diffs)
- [ ] Task 3.4: Add snapshot compression (optional)
- [ ] Task 3.5: Test roundtrip fidelity (load → modify → save → load)
- [ ] Task 3.6: Validate bit-exact reproducibility after restore

**Priority**: P0 (Foundation for data exchange)

---

## Epic: State Synchronization Model

**Description**: Deterministic state updates, snapshotting, rollback.

### Milestone 1: Deterministic Simulation Stage with Fixed Update Step
- [ ] Task 1.1: Implement fixed timestep update
- [ ] Task 1.2: Ensure deterministic physics and logic
- [ ] Task 1.3: Add input buffering for fixed step
- [ ] Task 1.4: Support determinism testing (replay)
- [ ] Task 1.5: Validate bit-exact reproducibility

### Milestone 2: Snapshot Manager for Save/Multiplayer
- [ ] Task 2.1: Implement snapshot capture from world state
- [ ] Task 2.2: Add snapshot comparison and diffing
- [ ] Task 2.3: Support snapshot compression
- [ ] Task 2.4: Implement snapshot versioning
- [ ] Task 2.5: Test snapshot-based save/load

### Milestone 3: Rollback/Rewind Support for Lockstep
- [ ] Task 3.1: Implement input history buffer
- [ ] Task 3.2: Add fast-forward replay from snapshot
- [ ] Task 3.3: Support deterministic recomputation
- [ ] Task 3.4: Validate correctness after rollback
- [ ] Task 3.5: Test rollback under network jitter

**Priority**: P2 (Enables advanced multiplayer)

---

## Epic: Developer Tooling

**Description**: Build pipeline, asset preprocessing, debugging, testing strategy.

### Milestone 1: Build/Install Doc and Dev Shell
- [ ] Task 1.1: Document native build steps
- [ ] Task 1.2: Document WASM build and deployment
- [ ] Task 1.3: Create setup script for dev environment
- [ ] Task 1.4: Add troubleshooting guide
- [ ] Task 1.5: Test build on clean system

### Milestone 2: Asset Manifest Generator and Validation Tools
- [ ] Task 2.1: Create asset manifest generator tool
- [ ] Task 2.2: Implement manifest validator CLI
- [ ] Task 2.3: Add asset dependency checker
- [ ] Task 2.4: Support asset preview/inspection
- [ ] Task 2.5: Test with real ROM examples

### Milestone 3: Test Harness for Runtime and ROM Regression
- [ ] Task 3.1: Create unit test framework for engine systems
- [ ] Task 3.2: Add integration tests for ROM loading
- [ ] Task 3.3: Implement regression test suite
- [ ] Task 3.4: Support automated ROM validation
- [ ] Task 3.5: Add CI/CD integration

**Priority**: P1 (Enables productive development)

---

## Epic: Memory & Resource Management

**Description**: WASM memory budgeting, GPU lifecycle, asset lifetime.

### Milestone 1: Explicit GPU Resource Ownership and Cleanup
- [ ] Task 1.1: Design GPU resource handle lifecycle
- [ ] Task 1.2: Implement RAII cleanup for textures/buffers
- [ ] Task 1.3: Add resource leak detection
- [ ] Task 1.4: Support resource destruction hooks
- [ ] Task 1.5: Test cleanup on scene exit

### Milestone 2: Asset Lifetime Tracking and Residency Control
- [ ] Task 2.1: Implement generational handles with residency markers (`last_access_frame`, `priority_tier`, `eviction_queued`)
- [ ] Task 2.2: Add classification-aware eviction callbacks (`reason`, `bytes_freed`, `fallback_applied`)
- [ ] Task 2.3: Support manual vs automatic lifetime
- [ ] Task 2.4: Add lifetime/residency diagnostics by class and scene scope
- [ ] Task 2.5: Test memory bounds enforcement with dynamic budget changes

### Milestone 3: Memory Budget Monitoring and Pressure Response
- [ ] Task 3.1: Track CPU/WASM and GPU memory usage separately with sampled history
- [ ] Task 3.2: Implement budget alerts and pressure-level state machine (`Normal`, `Constrained`, `Critical`)
- [ ] Task 3.3: Integrate profiler views for per-class usage, residency ratio, and eviction churn
- [ ] Task 3.4: Add pressure reports including queue depth and LOD downgrade counts
- [ ] Task 3.5: Test pressure scenarios for graceful degradation and recovery hysteresis

**Priority**: P1 (Ensures stability under constraints)

---

## Priority Summary

**P0 (Immediate Foundation)**
- Runtime Execution Model
- Asset Pipeline
- Platform Abstraction Layer
- Data Serialization Strategy

**P1 (Necessary Next Steps)**
- Game Logic Execution
- Scene / World System
- Authentication & Secure Data Access
- Input System (Advanced)
- Task / Async System
- Developer Tooling
- Memory & Resource Management
- Persistence System

**P2 (Later Enhancements)**
- Networking / Multiplayer Layer
- Audio System
- Debugging & Observability
- State Synchronization Model

---

## Initiative: GPU-Driven Neural Skeletal Animation (Rust + wgpu)

**Goal**: Run neural-network-driven skeletal animation fully on GPU (compute + vertex skinning) with minimal CPU-GPU traffic and scalable multi-character support.

### Suggested Development Order (Dependencies)
1. NS-01 -> NS-02 -> NS-03 (authoritative data model first)
2. NS-04 -> NS-05 (GPU resource model before shaders)
3. NS-06 -> NS-07 (render baseline before GPU NN)
4. NS-08 -> NS-09 (compute inference then transform compose)
5. NS-10 -> NS-11 (multi-character scale + profiling)
6. NS-12 in parallel after NS-06 (debug/visualization)

### Epic NS-01: GLB/glTF Import and Canonicalization
**Feature NS-01.1: GLB ingest pipeline**
- [ ] Task NS-01.1.1: Add GLB loader module to parse meshes, skins, joints, node hierarchy, and animations from binary glTF
- [ ] Task NS-01.1.2: Validate required vertex attributes exist (`POSITION`, `NORMAL`, `JOINTS_0`, `WEIGHTS_0`)
- [ ] Task NS-01.1.3: Implement strict error types for missing/incompatible skin data

**Feature NS-01.2: Coordinate and unit normalization**
- [ ] Task NS-01.2.1: Define canonical engine space (handedness, up axis, meters scale)
- [ ] Task NS-01.2.2: Normalize imported transforms to canonical space during asset build/import
- [ ] Task NS-01.2.3: Add import report output (joint count, mesh count, scale factors, warnings)

**Feature NS-01.3: Offline asset packing**
- [ ] Task NS-01.3.1: Create packed runtime structs for static mesh/skeleton data
- [ ] Task NS-01.3.2: Emit contiguous arrays for GPU upload (no pointer-linked runtime graph)
- [ ] Task NS-01.3.3: Cache packed output with content hash to avoid repeated conversion

### Epic NS-02: Skeletal Data Structures and Hierarchy
**Feature NS-02.1: Runtime skeleton model**
- [ ] Task NS-02.1.1: Define flat joint table (`parent_index`, `inverse_bind`, `rest_pose`)
- [ ] Task NS-02.1.2: Define joint remap table between glTF node order and skin joint order
- [ ] Task NS-02.1.3: Add max joint limits and validation gates for target hardware profiles

**Feature NS-02.2: Transform math contracts**
- [ ] Task NS-02.2.1: Standardize transform representation (`quat + translation`, optional uniform scale)
- [ ] Task NS-02.2.2: Implement CPU reference compose path for validation (not per-frame runtime path)
- [ ] Task NS-02.2.3: Add quaternion normalization and NaN guard utilities shared by CPU/GPU tests

### Epic NS-03: Neural Network Representation
**Feature NS-03.1: MLP schema and serialization**
- [ ] Task NS-03.1.1: Define layer schema (`in_dim`, `out_dim`, activation)
- [ ] Task NS-03.1.2: Define flat buffer layout for weights/biases per layer (aligned to 16-byte boundaries)
- [ ] Task NS-03.1.3: Implement NN asset serializer/deserializer with versioned header

**Feature NS-03.2: Input/Output contracts**
- [ ] Task NS-03.2.1: Define per-character input vector schema (velocity, facing, prior latent/state, optional env probes)
- [ ] Task NS-03.2.2: Define output schema mapping to joint local transforms (quat + translation per joint)
- [ ] Task NS-03.2.3: Add shape checks that fail pipeline creation when model/joint dimensions mismatch

### Epic NS-04: GPU Buffer Layout and Resource Management
**Feature NS-04.1: Buffer inventory and binding model**
- [ ] Task NS-04.1.1: Define storage buffers for skeletons, NN weights, per-character state, and output transforms
- [ ] Task NS-04.1.2: Define static vertex/index buffers for skinned meshes uploaded once at load
- [ ] Task NS-04.1.3: Define per-frame transient buffers (if needed) with ring-buffer allocator policy

**Feature NS-04.2: Layout correctness and alignment**
- [ ] Task NS-04.2.1: Add Rust-side `repr(C)` structs that mirror WGSL struct layout rules
- [ ] Task NS-04.2.2: Add compile-time/static assertions for stride/alignment and padding expectations
- [ ] Task NS-04.2.3: Add integration test that writes sentinel values and verifies shader reads

**Feature NS-04.3: Bind groups and pipeline layout**
- [ ] Task NS-04.3.1: Create separate bind group layouts for inference, transform compose, and skinning render
- [ ] Task NS-04.3.2: Implement resource lifetime ownership map (who creates, updates, destroys)
- [ ] Task NS-04.3.3: Add hot-reload-safe resource rebuild path for shader/layout changes

### Epic NS-05: Rendering Baseline with GPU Skinning
**Feature NS-05.1: Skinned mesh vertex shader path**
- [ ] Task NS-05.1.1: Implement WGSL vertex skinning using 4-joint blend weights
- [ ] Task NS-05.1.2: Compute skinned normals correctly (rotation-only or inverse-transpose path)
- [ ] Task NS-05.1.3: Add fallback path for meshes with fewer than 4 influences

**Feature NS-05.2: Minimal animated render scene**
- [ ] Task NS-05.2.1: Render one character using static test joint transforms from buffer
- [ ] Task NS-05.2.2: Add camera/light controls to inspect skinning artifacts
- [ ] Task NS-05.2.3: Capture reference screenshots for regression comparison

### Epic NS-06: Compute Shader NN Inference (WGSL)
**Feature NS-06.1: Layer kernels**
- [ ] Task NS-06.1.1: Implement WGSL kernel for dense layer mat-vec (or small mat-mat for batched chars)
- [ ] Task NS-06.1.2: Implement WGSL activation functions (`relu`, `tanh`, optional `gelu`)
- [ ] Task NS-06.1.3: Chain multiple layers with explicit intermediate buffers or ping-pong buffers

**Feature NS-06.2: Dispatch and synchronization**
- [ ] Task NS-06.2.1: Define workgroup sizes from target device limits and benchmark candidates
- [ ] Task NS-06.2.2: Insert required compute-to-render synchronization barriers in command encoding
- [ ] Task NS-06.2.3: Add per-dispatch timing markers for profiling

**Feature NS-06.3: CPU parity harness**
- [ ] Task NS-06.3.1: Implement CPU reference inference for same model format
- [ ] Task NS-06.3.2: Run parity tests GPU vs CPU with tolerance thresholds per layer
- [ ] Task NS-06.3.3: Log max absolute and relative error per output block

### Epic NS-07: Bone Transform Generation and Hierarchy Resolve
**Feature NS-07.1: Local output to model-space matrices**
- [ ] Task NS-07.1.1: Convert NN outputs to normalized local transforms per joint
- [ ] Task NS-07.1.2: Resolve hierarchy (parent-child compose) on GPU compute pass
- [ ] Task NS-07.1.3: Multiply by inverse bind matrices to produce final skinning palette

**Feature NS-07.2: Stability and plausibility guards**
- [ ] Task NS-07.2.1: Clamp extreme translations/rotations to configured per-joint limits
- [ ] Task NS-07.2.2: Add optional temporal smoothing buffer for jitter reduction
- [ ] Task NS-07.2.3: Add root-motion extraction output channel for gameplay integration

### Epic NS-08: End-to-End GPU Animation Pipeline
**Feature NS-08.1: Frame graph integration**
- [ ] Task NS-08.1.1: Encode pass order: character state update -> NN inference -> hierarchy compose -> render skinning
- [ ] Task NS-08.1.2: Keep per-frame CPU writes limited to compact character input/state buffer updates
- [ ] Task NS-08.1.3: Add validation mode that can read back a tiny subset of outputs for debugging only

**Feature NS-08.2: System modularization**
- [ ] Task NS-08.2.1: Split modules: `asset_loading`, `nn_runtime`, `animation_gpu`, `render_skinning`
- [ ] Task NS-08.2.2: Define trait contracts between modules to avoid tight coupling
- [ ] Task NS-08.2.3: Add dependency-injection-friendly initialization for testability

### Epic NS-09: Multi-Character Batching and Scaling
**Feature NS-09.1: Character indexing strategy**
- [ ] Task NS-09.1.1: Add SoA layout for per-character inputs/states to improve coalesced access
- [ ] Task NS-09.1.2: Add character index indirection table for active/inactive pooling
- [ ] Task NS-09.1.3: Ensure all shader addressing is index-driven (no per-character pipeline changes)

**Feature NS-09.2: Batched inference and skinning**
- [ ] Task NS-09.2.1: Dispatch inference for N characters per frame with configurable batch size
- [ ] Task NS-09.2.2: Support multiple skeleton archetypes via offsets/ranges in shared buffers
- [ ] Task NS-09.2.3: Add frustum/visibility gate to skip skinning for fully off-screen characters

**Feature NS-09.3: Scaling validation**
- [ ] Task NS-09.3.1: Add benchmark scenarios for 1, 10, 50, 100+ characters
- [ ] Task NS-09.3.2: Record frame time split (CPU encode, compute, render)
- [ ] Task NS-09.3.3: Add automated threshold alerts when scaling regresses

### Epic NS-10: Debugging and Visualization Tooling
**Feature NS-10.1: Skeleton debug view**
- [ ] Task NS-10.1.1: Render bone lines/joint axes overlay pass
- [ ] Task NS-10.1.2: Toggle between rest pose, NN output pose, and final skinned pose
- [ ] Task NS-10.1.3: Color joints by constraint violation or output magnitude

**Feature NS-10.2: GPU introspection aids**
- [ ] Task NS-10.2.1: Add optional buffer dump tool for selected character and frame
- [ ] Task NS-10.2.2: Add debug labels/markers for command encoder passes
- [ ] Task NS-10.2.3: Add shader compile error surfacing with file/line mapping in logs

### Epic NS-11: Performance Optimization
**Feature NS-11.1: Memory layout tuning**
- [ ] Task NS-11.1.1: Compare AoS vs SoA for NN IO and transform buffers
- [ ] Task NS-11.1.2: Align buffer strides to avoid bank conflicts/misaligned loads
- [ ] Task NS-11.1.3: Minimize duplicated data across passes (reuse intermediates where safe)

**Feature NS-11.2: Dispatch and occupancy tuning**
- [ ] Task NS-11.2.1: Sweep workgroup sizes and measure occupancy/latency
- [ ] Task NS-11.2.2: Fuse lightweight passes where it reduces memory traffic
- [ ] Task NS-11.2.3: Gate optional high-cost features behind quality levels

**Feature NS-11.3: CPU-GPU transfer minimization checks**
- [ ] Task NS-11.3.1: Add per-frame upload byte counter and target budget
- [ ] Task NS-11.3.2: Assert static mesh/skeleton/weight buffers are not re-uploaded per frame
- [ ] Task NS-11.3.3: Add CI regression test for unexpected buffer write growth

### Epic NS-12: Testing and Validation Strategy
**Feature NS-12.1: Unit tests (CPU-side contracts)**
- [ ] Task NS-12.1.1: Test glTF parsing and canonicalization edge cases
- [ ] Task NS-12.1.2: Test transform compose, quaternion normalization, and hierarchy traversal math
- [ ] Task NS-12.1.3: Test NN serialization/deserialization and shape validation

**Feature NS-12.2: GPU parity and correctness tests**
- [ ] Task NS-12.2.1: Add deterministic GPU-vs-CPU inference test vectors
- [ ] Task NS-12.2.2: Add skinning correctness test with known simple rig pose
- [ ] Task NS-12.2.3: Add tolerance-based snapshot test for joint matrices across frames

**Feature NS-12.3: Visual validation**
- [ ] Task NS-12.3.1: Define golden animation clips and expected pose checkpoints
- [ ] Task NS-12.3.2: Add screenshot/video diff checks for key camera angles
- [ ] Task NS-12.3.3: Add debug HUD with per-pass timings, joint counts, and active character count

### Potential Pitfalls (Track as Risk Issues)
- [ ] Risk NS-R1: Coordinate system mismatch between Blender/glTF/import/runtime causes mirrored or rotated skeletons
- [ ] Risk NS-R2: WGSL/Rust layout mismatch (`std430`-style assumptions, padding) corrupts transforms
- [ ] Risk NS-R3: Quaternion drift without normalization creates exploding poses over time
- [ ] Risk NS-R4: Inverse bind matrix ordering mismatch yields subtle skinning distortions
- [ ] Risk NS-R5: Precision loss (`f16`/`f32`) destabilizes NN outputs for deep/wide models
- [ ] Risk NS-R6: Hidden CPU sync points (buffer mapping/readback) stall frame pipeline
- [ ] Risk NS-R7: Bone hierarchy resolve becomes bottleneck for large rigs without optimized pass design
- [ ] Risk NS-R8: Divergent branches in shader constraints/smoothing hurt occupancy

### Iteration Path (Start Simple -> Scale)
1. Single character, single rig, fixed test inputs, static NN weights, no environment inputs
2. Single character, dynamic per-frame inputs, GPU NN inference, basic constraints
3. Multiple characters with same rig, batched inference and skinning
4. Multiple rigs/archetypes with shared pipelines and offset indexing
5. Add optional environment features, temporal smoothing, quality tiers, and aggressive perf tuning

### Optional Stretch Goals
- [ ] Stretch NS-S1: Mixed precision inference (`f16` weights/intermediates) with automatic fallback to `f32`
- [ ] Stretch NS-S2: GPU IK post-pass for feet/hands grounding after NN pose output
- [ ] Stretch NS-S3: Motion matching or latent-space controller feeding NN inputs
- [ ] Stretch NS-S4: Runtime model hot-swap and A/B blend between two NN policies
- [ ] Stretch NS-S5: Async streaming of NN models and skeleton LOD sets based on distance/perf budget

---

## Technical Risks

- **WASM Constraints**: Single-threaded, async-only, no file I/O. Must use browser fetch and JS host calls for all I/O.
- **WebGPU Compatibility**: Feature set varies across browsers. Must target WebGL-compatible limits and handle surface format differences.
- **Auth Token Storage**: Browser storage must avoid insecure persistence. Tokens must be passed from JS host.
- **Remote Asset Integrity**: ROM content can be tampered with. Need hash validation and trusted URLs.
- **Serialization Cost**: Browser parsing overhead impacts load times. Prefer JSON/RON for config, binary only if profiling justifies.
- **Browser Networking**: Limited to fetch/WebSocket with CORS/TLS. No raw sockets or custom transport.

---

## Development Notes

- Keep each task small and independently testable.
- Avoid embedding game logic in Rust; encode behaviors in data and interpreters.
- Start with JSON/RON; evaluate binary formats only after profiling identifies bottleneck.
- Design all systems for platform abstraction: core logic is platform-agnostic, adapters handle environment specifics.
- Maintain determinism in simulation layer for multiplayer and replay support.
- Regularly sync backlog with commit history and release notes.
