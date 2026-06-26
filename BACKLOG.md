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
- Keep physics response deterministic while supporting configurable restitution (elastic/inelastic) and bounded-world collisions.
- Enforce deny-by-default authz and per-request authorization checks for protected endpoints.
- Prefer documented extension seams (traits/modules/contracts) before adding complexity.
- Classify runtime resources (`critical`, `scaled_optional`, `reused`, `streaming`) and tie each class to explicit budget/eviction policy.
- Treat memory budgets as dynamic signals (CPU/WASM + GPU), with graceful quality fallback before hard failure.
- Keep streaming and upload work off the frame-critical path via priority queues and bounded per-frame upload budgets.
- Require fallback assets/LODs for streamable content so over-budget conditions degrade quality, not correctness.

---

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

- [ ] Task 3.1: Implement InputManager (capture keyboard events)
- [ ] Task 3.2: Add input-to-action mapping (WASD → move, arrows → rotate)
- [ ] Task 3.3: Create input event stage in scheduler
- [ ] Task 3.4: Add fixed timestep simulation stage (60 Hz)
- [ ] Task 3.5: Update transform based on input each frame
- [ ] Task 3.6: Record input history and test replay (same result)
- [ ] Task 3.7: Test determinism across multiple browser sessions
- [ ] Task 3.8: Add frame-by-frame debug stepping (optional)

**Demonstrates**:
- Input abstraction works on browser
- Game logic can be driven by player actions
- Simulation is deterministic (foundation for multiplayer)
- Players feel responsive control

### Milestone 4: Session Support & Basic Multiplayer (Shared Sessions)
**Outcome**: Multiple browser clients can join the same session and see each other's triangle positions updated in (near) real-time. This milestone proves session lifecycle, basic transport, and authoritative state propagation.

- [ ] Task 4.1: Define `SessionProtocol` (join, leave, broadcast, authoritative state update)
- [ ] Task 4.2: Implement a LocalSessionHost (in-process or Node/warp dev server) for rapid testing
- [ ] Task 4.3: Add client-side session join/leave APIs (WASM + JS glue)
- [ ] Task 4.4: Implement authoritative position sync for player-controlled entities (server authoritative with client updates)
- [ ] Task 4.5: Add basic latency compensation on clients (interpolation of remote entities)
- [ ] Task 4.6: Implement simple concurrency rules for entity interactions (e.g., collision or basic 'hit' event)
- [ ] Task 4.7: Test with multiple browser instances (2+) and verify state convergence
- [ ] Task 4.8: Add minimal security: session tokens and scoped auth for join requests
- [ ] Task 4.9: Document session API and example integration (how to host a session)

**Demonstrates**:
- Multi-user connection lifecycle (join/leave)
- Shared world state propagation and reconciliation
- Latency handling patterns (interpolation) and simple authoritative model
- Security posture for session join (token-based)

**Priority**: P0 (Enables early multi-user testing and session validation)

### Milestone 5: Runtime Performance Baseline and Profiling Gates
**Outcome**: Runtime has explicit performance budgets and profiling checkpoints before major architectural expansion.

- [ ] Task 5.1: Define frame budget targets for browser MVP (frame time, startup time, memory ceiling)
- [ ] Task 5.2: Add startup and frame timing instrumentation (cold start, first frame, frame pacing)
- [ ] Task 5.3: Add lightweight render stats (draw calls, buffer uploads per frame)
- [ ] Task 5.4: Create performance regression checklist for each milestone
- [ ] Task 5.5: Require profiling evidence before introducing compute offload or complex scheduling

**Demonstrates**:
- Performance decisions are measured, not guessed
- Core loop remains stable as scope grows

**Priority**: P0 (Protects gameplay feel and iteration speed)

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
