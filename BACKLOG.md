# Anzu Engine Backlog

This backlog is the active plan for a simulation and visualization system.

## Status Baseline
- Implementation status: Runtime prototype in progress; no backlog feature is complete based on `anzu-runtime/src/main.rs`.
- Scope note: Use only behavior in `anzu-runtime/src/main.rs` as completion evidence for this status reset. Do not credit archived engine code or plans.
- Tracking rule: Mark a feature complete only when its stated outcome and acceptance criteria are implemented and validated.

## Priority Override: Establish Performance Baselines (Start Now)
- Decision: implement runtime performance monitoring before further performance-sensitive engine or rendering work.
- Working sequence:
  1. Feature 9.1: Frame boundary instrumentation and health counters.
  2. Feature 9.6: Unified benchmark output for simulation and extraction phases.
  3. Resume the current geometry-loading priority sequence.
- Rationale: active runtime diagnostics do not currently provide a repeatable frame/stage performance baseline. Archived profiler code is not active implementation evidence.
- Scope boundary: first establish repeatable CPU timing and workload counters. Add GPU timing only where the active backend supports it. Keep low-cost counters separate from sampled or opt-in detailed timing.
- Acceptance gate: performance-sensitive changes must compare against a recorded baseline on the same representative workload, and report timing distributions, work counts, and measurement overhead.

## Priority Override: Blender Geometry Loading (Next Up)
- Decision: prioritize loading arbitrary geometry exported from Blender and add streaming residency support for large scenes.
- Why:
  - Generated object paths are not meeting runtime quality expectations.
  - Importing external geometry is now the fastest path to useful content iteration.
  - Large-scene domain profile requirements need bounded storage -> RAM -> GPU resource residency flow.
- Working sequence (ordered):
  1. Feature 11.5: Externalized ROM content manifests with schema validation and runtime loading.
  2. Feature 11.6: ROM-to-engine asset provider registration (mesh, texture, material).
  3. Feature 10.1: Asset class taxonomy (critical, scaled_optional, reused, streaming).
  4. Feature 10.2: Residency state machine (queued, loading, resident, eviction pending, evicted).
  5. Feature 10.3: Streaming queues with bounded per-frame upload budgets.
  6. Feature 9.3: Upload budget and pipeline-switch telemetry.
  7. Feature 10.4: Budget-aware quality fallback before correctness-impacting eviction.
  8. Feature 10.5: Residency and eviction observability per asset class.
  9. Feature 11.7: Startup-only asset staging and pack-once GPU upload contract.
  10. Feature 2.2: ObjectSpec domain with predefined catalog templates.
  11. Feature 2.3: ObjectRuntime domain with resolved resource handles and lifecycle state.
  12. Feature 2.6: Extractor emits frame packet from ECS/object snapshot.
  13. Feature 2.7: Render consumes frame packet only, without direct ECS schema coupling.
  14. Feature 5.5: Unified import pipeline that emits display and physics assets.
- Scope boundary for this reprioritization:
  - Target format for first pass: Blender-exported GLB assets loaded through ROM manifests.
  - Near-term extension: bounded resource residency path for large scenes under Feature 10.x.
  - Non-goals for first pass: editor hot-reload and per-object material authoring UI.
- Dependency note:
  - Feature 1.5b remains planned, but is sequenced after this asset-loading and resource residency track so body-count expansion builds on real imported geometry and stable budgets.

## Personas
- Developer
- Simulation User

## Initiative 1: Runtime and Frame Lifecycle
- [ ] Feature 1.1: Initialize runtime in browser host with stable frame loop
  - Outcome: The browser host starts with a canvas-backed runtime, produces the first frame without a crash, and maintains stable pacing across resize, tab visibility changes, and short suspend/resume events.
  - Scope:
    - In scope: browser canvas bootstrap, wasm-ready app-shell state, explicit redraw pacing, bounded delta handling, fixed-step accumulation with a max-step cap, safe surface reconfiguration, and an explicit acquire -> encode -> submit -> present frame contract.
    - Non-goals: configurable clear color, baseline triangle or texture content changes, gameplay simulation ownership changes, and the full state-decomposition migration from Features 1.8 through 1.12.
  - Constraints and assumptions:
    - The primary acceptance target is the wasm browser host; native behavior must continue to compile.
    - The current render output is considered placeholder continuity and is not the completion metric.
    - The scheduler should use a clamped delta cap of 0.25s to avoid runaway catch-up after tab suspension.
    - The implementation should not introduce full simulation ownership changes yet; it should only add a scheduler seam and a renderer-facing frame entry point.
  - Milestones:
    - M1: Extract shell ownership and introduce runtime readiness state.
    - M2: Introduce runtime pacing and fixed-step scheduling.
    - M3: Move render backend/resource ownership behind a renderer-facing frame path and harden surface lifecycle handling.
  - File-by-file implementation contract:
    - [anzu-engine/src/lib.rs](anzu-engine/src/lib.rs)
      - Action: modify
      - Why: keep this file as the shell adapter and route browser events into the runtime boundary.
      - Structs/enums: `enum AppBootstrapState { Booting, Ready, Failed(String) }`
      - Functions: `fn resumed(&mut self, event_loop: &ActiveEventLoop)`, `fn window_event(...)`, `fn user_event(...)`
      - Ownership notes: App owns shell state only; it should not own frame pacing or GPU backend lifecycle directly.
    - [anzu-engine/src/execution.rs](anzu-engine/src/execution.rs)
      - Action: modify
      - Why: expose the runtime/render modules used by the browser host and keep the execution module as the public boundary.
      - Functions: `pub mod shared`, `pub mod render`
      - Ownership notes: this module is the public execution entry point; it should not contain business logic.
    - [anzu-engine/src/execution/shared.rs](anzu-engine/src/execution/shared.rs)
      - Action: create/modify
      - Why: host the runtime scheduler and timing state so the frame loop is testable independent of the GPU backend.
      - Structs/enums: `struct RuntimeState`, `struct FrameScheduler`, `struct FrameStepPlan`
      - Functions: `fn handle_redraw(&mut self, delta_seconds: f32) -> anyhow::Result<()>`, `fn fixed_update(&mut self)`, `fn resize(&mut self, width: u32, height: u32) -> anyhow::Result<()>`, `fn begin_frame(&mut self, delta_seconds: f32) -> FrameStepPlan`
      - Ownership notes: RuntimeState owns pacing decisions and fixed-step accumulation; it delegates rendering to the backend layer.
    - [anzu-engine/src/execution/render.rs](anzu-engine/src/execution/render.rs)
      - Action: create/modify
      - Why: move surface/device/queue/config and long-lived GPU resources out of the shell and into the renderer-facing layer.
      - Structs/enums: `struct RenderBackendState`, `struct RenderResources`, `enum FrameAcquireOutcome { Ready, Reconfigure, Skipped, Fatal }`
      - Functions: `fn new(window: Arc<Window>) -> anyhow::Result<(RenderBackendState, RenderResources)>`, `fn resize(&mut self, width: u32, height: u32) -> anyhow::Result<()>`, `fn acquire_frame(&mut self) -> anyhow::Result<FrameAcquireOutcome>`, `fn render_frame(&mut self, resources: &RenderResources) -> anyhow::Result<()>`
      - Ownership notes: RenderBackendState owns surface lifecycle and the acquire/encode/submit/present path; RenderResources owns persistent GPU assets.
    - [documents/architecture.md](documents/architecture.md)
      - Action: reference only
      - Why: keep terminology aligned with the runtime/shell/render boundaries while avoiding unrelated documentation churn.
  - Execution flow and ownership boundaries:
    - App owns platform shell concerns only: create window, bootstrap runtime, forward events, and transition readiness state.
    - RuntimeState owns pacing decisions and fixed-step accumulation.
    - RenderBackendState owns surface-state transitions and the frame submission path.
    - RenderResources owns long-lived GPU resources that persist across frames.
    - The renderer-facing frame path should be the only place where acquire/encode/submit/present is executed for this feature.
  - Validation plan:
    - Unit tests for `FrameScheduler::begin_frame` covering zero delta, clamped delta, accumulator carryover, and max-step limiting.
    - Native compile/test pass for the touched runtime module.
    - wasm compile check with `cargo check --target wasm32-unknown-unknown`.
    - Manual browser smoke check for first-frame boot, resize, and suspend/resume stability.
  - Risks and open questions:
    - Risk: redraw ownership can be duplicated if both shell and renderer request redraws. The plan should keep redraw scheduling in the shell/runtime boundary and not in render code.
    - Risk: browser suspend/resume may produce large deltas. The plan should clamp them and limit catch-up steps.
    - Open question resolved by this plan: the feature should use a minimal renderer split now, not a full extraction stack; later features can expand into `RenderExtractionState` and `SimulationState`.
- [ ] Feature 1.2: Clear frame to configurable background color
  - Outcome: The runtime clears each frame with a configurable background color instead of a hardcoded literal, with ROM-driven runtime updates and explicit validation/error behavior.
  - Scope:
    - In scope: a typed background-color config model, runtime setter path, ROM-issued clear-color updates, render-pass integration using runtime-owned clear state, and unit tests for defaults/validation/update flow.
    - Non-goals: material-system integration, per-entity background effects, post-processing passes, and full ECS extraction ownership migration.
  - Constraints and assumptions:
    - Color representation uses linear RGBA with `f64` channels in [0.0, 1.0] aligned to `wgpu::Color`.
    - Invalid channel input should return an error; this feature does not silently clamp.
    - ROM integration should use `src/roms/texture_demo.rs` as the first profile seam.
    - ROM commands are ROM-local and translated in `lib.rs` before applying to runtime state.
    - Legacy code in `src/abandoned_generated_code/` is guidance-only and not copied as architecture.
    - The clear color source should be runtime/render-state owned and consumed in the frame path, not a shell-local literal.
    - Command application is deterministic: accepted commands apply at next-frame boundary, not immediately mid-frame.
  - Milestones:
    - M1: Introduce clear-color domain types and validation at execution runtime boundary.
    - M2: Wire clear color through frame rendering path and remove hardcoded literal clear.
    - M3: Add ROM-issued runtime update path and tests/manual verification.
  - File-by-file implementation contract:
    - [anzu-engine/src/execution/shared.rs](anzu-engine/src/execution/shared.rs)
      - Action: modify
      - Why: runtime state already owns frame scheduling and is the best near-term seam for frame-level configuration.
      - Structs/enums:
        - `struct BackgroundColor { r: f64, g: f64, b: f64, a: f64 }`
        - `enum BackgroundColorError { ChannelOutOfRange { channel: &'static str, value: f64 } }`
      - Functions/methods:
        - `fn default_background_color() -> BackgroundColor`
        - `fn set_background_color(&mut self, color: BackgroundColor) -> anyhow::Result<()>`
        - `fn background_color(&self) -> BackgroundColor`
      - Ownership notes: `RuntimeState` remains owner of mutable frame-control values exposed to render.
    - [anzu-engine/src/lib.rs](anzu-engine/src/lib.rs)
      - Action: modify
      - Why: current frame clear is hardcoded in shell render path; this must become runtime-driven until later decomposition features migrate rendering ownership.
      - Structs/enums:
        - `enum RomRenderCommand { SetBackgroundColor(BackgroundColor) }` (shell-facing translated command)
        - Update `State` usage to read clear color from runtime-owned state.
      - Functions/methods:
        - `fn render(&mut self) -> Result<(), anyhow::Error>` updated to use runtime clear value.
        - `fn update(&mut self)` routes ROM-local commands through translation and queues at most one pending clear-color update.
      - Ownership notes: app shell reads and applies runtime clear state; it does not own background-color policy.
    - [anzu-engine/src/roms/texture_demo.rs](anzu-engine/src/roms/texture_demo.rs)
      - Action: modify
      - Why: this profile is the requested first ROM seam for background-color control.
      - Structs/enums:
        - `enum TextureDemoRenderCommand { SetBackgroundColor { r: f64, g: f64, b: f64, a: f64 } }`
      - Functions/methods:
        - `fn initial_render_command() -> Option<TextureDemoRenderCommand>`
        - `fn next_render_command(...) -> Option<TextureDemoRenderCommand>` (signature adapted to active runtime hook)
      - Ownership notes: ROM owns scenario intent type; shell translates into runtime-owned validated state.
    - [anzu-engine/src/execution.rs](anzu-engine/src/execution.rs)
      - Action: modify
      - Why: expose background-color runtime types for shell translation and runtime mutation.
      - Functions/modules:
        - Re-export `BackgroundColor` and `BackgroundColorError` from execution boundary.
  - ROM contract (tightened):
    - Command source: ROM-local command type per profile (starting with `TextureDemoRenderCommand`).
    - Translation boundary: `lib.rs` translates ROM-local command payloads into runtime `BackgroundColor` values.
    - Cardinality: ROM emits at most one background-color command per update/frame tick.
    - Scheduling: shell queues at most one pending background-color value and applies it at next-frame boundary before render clear is encoded.
    - Default behavior: if ROM emits no initial command, runtime keeps engine default background color.
    - Validation behavior: runtime rejects invalid channels, keeps previous valid color, and continues frame processing.
    - Determinism rule: multiple same-tick emissions are treated as contract violation in tests; runtime may ignore extras after first.
  - Execution flow and ownership boundaries:
    - ROM emits optional `TextureDemoRenderCommand::SetBackgroundColor` values.
    - Shell translates ROM-local payload into runtime `BackgroundColor` and queues one pending update.
    - Runtime applies queued update at next-frame boundary, validates channel ranges, and stores accepted value in runtime-owned state.
    - Render path reads runtime-owned clear color and maps it into `wgpu::LoadOp::Clear(wgpu::Color { ... })`.
    - If validation fails, runtime returns an error result for the update path and preserves previous valid clear color without aborting frame rendering.
  - Validation plan:
    - Unit tests:
      - default color is deterministic and fully opaque.
      - invalid channels (<0.0 or >1.0) produce `Err` and preserve prior value.
      - valid runtime setter updates are reflected in frame clear state reads.
      - command scheduling applies updates on next frame only.
      - command cardinality enforces at-most-one emission per tick in ROM contract tests.
    - Build checks:
      - `cd /workspaces/Anzu/anzu-engine && cargo test`
      - `cd /workspaces/Anzu/anzu-engine && cargo check --target wasm32-unknown-unknown`
    - Manual browser smoke:
      - verify startup clear color comes from ROM-provided value.
      - trigger at least one runtime color update and verify next frame clear reflects it.
  - Risks and open questions:
    - Risk: in current architecture `lib.rs` still hosts the render pass; later migration features must relocate this path into execution render backend without duplicating clear-color ownership.
    - Risk: if ROM command cadence is per-frame, unnecessary state writes may occur; setter should be idempotent where practical.
    - Open question resolved by this plan: ROM uses ROM-local command types with shell translation, while runtime remains source of truth for validated clear-color state.
- [ ] Feature 1.3: Render baseline triangle geometry
  - Outcome: The runtime renders a centered, CCW triangle using engine mesh-provider geometry and a color-only pipeline, with per-vertex color attributes and no texture sampling in the active path.
  - Scope:
    - In scope: consume triangle geometry from `assets::mesh_provider`, keep indexed draw path, introduce baseline color-only shader/pipeline state, and remove active texture loading/bind-group usage from the frame path.
    - Non-goals: texture sampling and UV-driven material behavior (Feature 1.4), full simulation/extraction ownership migration (Features 1.8 through 1.12), and generalized multi-mesh scene composition.
  - Constraints and assumptions:
    - Baseline geometry source is `src/assets/mesh_provider.rs` (`Primitive::Triangle`) instead of hardcoded local geometry constants.
    - Triangle remains fully visible in NDC, centered, and uses CCW winding.
    - Draw contract remains indexed for parity with mesh-provider indices and future shape reuse.
    - Shader is color-only for this feature, but color source is per-vertex attribute (not a hardcoded fragment literal).
    - Baseline per-vertex colors are fixed to: A=`[1.0, 0.5, 0.0, 1.0]` (orange), B=`[0.0, 0.5, 0.5, 1.0]` (teal), C=`[0.29, 0.0, 0.51, 1.0]` (indigo).
    - Winding enforcement for Feature 1.3 tests uses exact index-order validation (`[0, 1, 2]`) instead of signed-area checks.
    - Active shader source of truth for this feature is `src/execution/shaders/shader.wgsl`; duplicate active shader paths are out of scope.
    - Pipeline/resource responsibilities begin moving into `src/execution/render.rs` in this feature, while app-shell/event-loop ownership remains in `src/lib.rs`.
    - Targeted mesh-provider cleanup is allowed only where required by the new baseline triangle contract.
  - Milestones:
    - M1: Establish mesh-provider baseline triangle data path and validation seam.
    - M2: Introduce color-only per-vertex render pipeline in execution render module.
    - M3: Wire shell to execution render path, remove active texture path, and validate wasm/native behavior.
  - File-by-file implementation contract:
    - [anzu-engine/src/assets.rs](anzu-engine/src/assets.rs)
      - Action: modify
      - Why: expose small, testable mesh accessors needed by render-path conversion without spreading geometry assumptions.
      - Structs/enums:
        - `struct Mesh` remains canonical mesh container.
      - Functions/methods:
        - `fn indices(&self) -> Option<&[u16]>` (new helper for read-only indexed draw integration).
      - Ownership notes: `Mesh` remains data-only; it does not own GPU conversion concerns.
    - [anzu-engine/src/assets/mesh_provider.rs](anzu-engine/src/assets/mesh_provider.rs)
      - Action: modify
      - Why: make triangle retrieval explicit and add minimum invariants for baseline rendering.
      - Structs/enums:
        - `enum Primitive` remains unchanged in responsibility.
      - Functions/methods:
        - `fn get_primitive_mesh(primitive: &Primitive) -> Mesh` (existing path retained).
        - `fn triangle_mesh() -> Mesh` (new convenience seam for Feature 1.3 usage).
        - `fn validate_triangle_mesh(mesh: &Mesh) -> anyhow::Result<()>` (new guard for point/index cardinality and bounds).
      - Ownership notes: provider owns static geometry definitions; validation errors are surfaced to render setup.
    - [anzu-engine/src/execution/render.rs](anzu-engine/src/execution/render.rs)
      - Action: modify
      - Why: begin render ownership migration by centralizing baseline triangle GPU resource setup and draw encoding.
      - Structs/enums:
        - `struct BaselineVertex { position: [f32; 3], color: [f32; 4] }`
        - `struct BaselineGeometryGpu { vertex_buffer: wgpu::Buffer, index_buffer: wgpu::Buffer, index_count: u32 }`
        - `struct BaselinePipelineGpu { render_pipeline: wgpu::RenderPipeline }`
        - `struct BaselineTriangleRenderer { geometry: BaselineGeometryGpu, pipeline: BaselinePipelineGpu }`
      - Functions/methods:
        - `fn from_mesh_triangle(mesh: &crate::assets::Mesh) -> anyhow::Result<Vec<BaselineVertex>>`
        - `fn create_baseline_triangle_renderer(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> anyhow::Result<BaselineTriangleRenderer>`
        - `fn encode_baseline_triangle_pass(renderer: &BaselineTriangleRenderer, encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView, clear_color: crate::execution::BackgroundColor)`
      - Ownership notes: execution render module owns pipeline/shader/buffer setup and draw encoding for baseline geometry.
    - [anzu-engine/src/execution/shaders/shader.wgsl](anzu-engine/src/execution/shaders/shader.wgsl)
      - Action: modify
      - Why: convert active shader path to per-vertex color IO and remove texture bindings from baseline pipeline.
      - Functions/modules:
        - `@vertex fn vs_main(...)`
        - `@fragment fn fs_main(...)`
      - Ownership notes: shader contract must match `BaselineVertex` layout exactly.
    - [anzu-engine/src/lib.rs](anzu-engine/src/lib.rs)
      - Action: modify
      - Why: keep shell responsibilities while delegating baseline pipeline/geometry setup and pass encoding into execution render seam.
      - Structs/enums:
        - `State` drops active texture bind-group ownership for this feature.
        - `State` stores `BaselineTriangleRenderer` (from execution render module).
      - Functions/methods:
        - `fn new(...) -> anyhow::Result<Self>` uses `create_baseline_triangle_renderer(...)`.
        - `fn render(&mut self) -> Result<(), anyhow::Error>` calls `encode_baseline_triangle_pass(...)`.
      - Ownership notes: app shell remains orchestrator (window/events/present timing) and does not define triangle GPU setup details.
  - Execution flow and ownership boundaries:
    - `State::new` requests triangle mesh from mesh provider through execution render setup entrypoint.
    - Execution render setup validates mesh cardinality/bounds, converts to `BaselineVertex`, and allocates GPU buffers/pipeline.
    - Per frame, shell resolves runtime background color and delegates draw encoding to `encode_baseline_triangle_pass`.
    - Shell submits and presents; execution render code remains responsible for render-pass draw details.
    - No texture resource creation or bind-group usage occurs on the active 1.3 path.
  - Validation plan:
    - Unit tests:
      - `mesh_provider_triangle_is_centered_and_ccw` validates canonical triangle points and exact index order `[0, 1, 2]`.
      - `mesh_provider_triangle_indices_are_in_bounds` rejects malformed index/cardinality cases.
      - `baseline_vertex_conversion_assigns_expected_colors` verifies deterministic A/B/C color mapping.
      - `baseline_triangle_pipeline_builds_with_color_shader` validates shader/vertex layout compatibility.
      - `baseline_triangle_pass_draws_indexed_triangle` validates indexed draw contract for one triangle.
    - Build checks:
      - `cd /workspaces/Anzu/anzu-engine && cargo test`
      - `cd /workspaces/Anzu/anzu-engine && cargo check --target wasm32-unknown-unknown`
    - Manual browser smoke:
      - verify exactly one centered CCW triangle is visible with per-vertex color interpolation.
      - verify configured background color from Feature 1.2 still applies behind triangle.
      - verify no texture-dependent rendering artifacts remain on the active baseline path.
      - evidence mode: text-only manual observation is sufficient for this feature (no screenshot requirement).
  - Risks and open questions:
    - Risk: partial render migration could duplicate responsibilities between shell and execution render modules if boundaries are not enforced in signatures.
    - Risk: mismatched shader IO vs vertex layout will fail pipeline creation at runtime; tests should cover layout assumptions early.
    - Open question resolved by refinement: use mesh-provider geometry + indexed draw + per-vertex color shader with fixed orange/teal/indigo mapping, execution shader as single active source, and active texture path removed in Feature 1.3.
- [ ] Feature 1.4: Apply texture to baseline geometry
  - Outcome: The runtime renders the baseline triangle with ROM-owned texture content and UV mapping while preserving indexed draw, runtime clear-color behavior, and shell-orchestration boundaries.
  - Scope:
    - In scope: ROM-owned texture identifiers and ROM-local asset resolution, UV-enabled baseline geometry contract, textured shader and pipeline path, texture upload and sampler bind-group setup, startup texture selection, and validation for UV and texture contracts.
    - Non-goals: runtime texture switching, engine-owned texture catalogs, multi-material generalization, and model draw-path integration beyond baseline triangle.
  - Constraints and assumptions:
    - Texture assets and texture/model definitions are owned by ROM content under `anzu-engine/src/roms/files`, including `anzu-engine/src/roms/files/textures`.
    - ROM resolves texture identifiers to asset sources; engine only performs generic decode, upload, and bind behavior.
    - Baseline blend behavior for this feature is texture sample multiplied by per-vertex color.
    - Initial sampler policy is `Linear` + `ClampToEdge`, with future extensibility.
    - Active shader source of truth remains `src/execution/shaders/shader.wgsl`.
  - Milestones:
    - M1: Extend geometry contract with UV coordinates and ROM-owned texture identifier ownership.
    - M2: Add textured render pipeline and shader bindings in execution render boundary.
    - M3: Wire shell startup path to ROM-resolved textured content and validate native + wasm behavior.
  - File-by-file implementation contract:
    - [anzu-engine/src/assets.rs](anzu-engine/src/assets.rs)
      - Action: modify
      - Why: extend `Mesh` contract to carry optional UV data aligned to vertex indices.
    - [anzu-engine/src/assets/mesh_provider.rs](anzu-engine/src/assets/mesh_provider.rs)
      - Action: modify
      - Why: provide canonical UVs for `Primitive::Triangle` with existing indexed CCW invariants.
    - [anzu-engine/src/roms/texture_demo.rs](anzu-engine/src/roms/texture_demo.rs)
      - Action: modify
      - Why: define ROM texture identifier enum, startup textured content payload, and ROM-local identifier-to-asset resolution targeting `anzu-engine/src/roms/files/textures`.
    - [anzu-engine/src/execution/render.rs](anzu-engine/src/execution/render.rs)
      - Action: modify
      - Why: add textured vertex layout, textured mesh validation, texture decode/upload and sampler bind-group setup, textured renderer creation, and textured pass encoding.
    - [anzu-engine/src/execution/shaders/shader.wgsl](anzu-engine/src/execution/shaders/shader.wgsl)
      - Action: modify
      - Why: extend shader IO for UV and texture bindings; fragment multiplies texture sample by vertex color.
    - [anzu-engine/src/lib.rs](anzu-engine/src/lib.rs)
      - Action: modify
      - Why: replace color-only renderer wiring with textured renderer wiring while preserving shell lifecycle orchestration and runtime background-color flow.
    - [anzu-engine/src/roms/files/textures](anzu-engine/src/roms/files/textures)
      - Action: reference only
      - Why: ROM-packaged texture catalog ownership boundary; engine does not own this catalog.
    - [anzu-engine/src/roms/files/objects](anzu-engine/src/roms/files/objects)
      - Action: reference only
      - Why: ROM-packaged model/object data ownership boundary (runtime use expanded in later features).
  - Execution flow and ownership boundaries:
    - ROM emits startup textured content contract: primitive + colors + UVs + ROM texture identifier mapped to assets packaged in `anzu-engine/src/roms/files/textures`.
    - Shell translates ROM content into execution render input without deciding texture policy or asset catalog membership.
    - Execution render consumes ROM-provided texture payload, decodes image bytes, uploads GPU texture, creates sampler and bind group, and builds the textured pipeline.
    - Shell remains responsible for frame orchestration (`acquire -> encode -> submit -> present`) while delegating textured pass encoding details to execution render.
  - Validation plan:
    - Unit tests:
      - UV cardinality and index-bounds validation for textured mesh contract.
      - Textured vertex-layout/shader compatibility and textured pipeline creation.
      - ROM texture identifier resolution and startup-command determinism.
    - Build checks:
      - `cd /workspaces/Anzu/anzu-engine && cargo test`
      - `cd /workspaces/Anzu/anzu-engine && cargo check --target wasm32-unknown-unknown`
    - Manual browser smoke:
      - verify one centered textured triangle is visible.
      - verify Feature 1.2 runtime background color behavior remains unchanged.
  - Risks and open questions:
    - Risk: policy leakage if engine starts owning texture-catalog logic. Mitigation: keep content mapping in ROM layer only.
    - Risk: shader/layout mismatch for UV bindings can fail pipeline creation. Mitigation: add layout-compat tests early.
    - Open question resolved by refinement: ROM owns texture definitions and identifiers; engine remains content-agnostic for catalog policy.
- [ ] Feature 1.5: Animate baseline transform at fixed timestep
  - Outcome: The texture demo now animates a single ROM-owned tetrahedron at the fixed-step boundary. The ROM owns the body state and transform closure, the runtime consumes the body state on each fixed step, and the renderer applies a view-projection matrix so the object stays centered and visible.
  - Implemented behavior:
    - `TextureDemoRom` owns the body state and projection mode.
    - `P` toggles between `Perspective` and `Isometric` projection modes at runtime.
    - `render.rs` builds the projection matrix and combines it with the runtime transform before upload.
    - The isometric view is framed on the tetrahedron's world-space center so it renders in the middle of the screen.
  - Validation:
    - `cd /workspaces/Anzu/anzu-engine && cargo test --lib -- --nocapture`
    - Result: 37 tests passed, 0 failed.
  - Notes:
    - The old clarification questions are closed by implementation: motion is ROM-provided, the primitive is the tetrahedron, and projection mode is user-toggleable.
- [ ] Feature 1.5b: Add N-body animation support for multiple bodies
  - Outcome: The runtime animates several ROM-owned bodies at the same fixed-step boundary, each body keeps its own transform closure and frame counter, and the renderer emits one indexed draw per body using shared geometry and texture resources.
  - Scope:
    - In scope: multi-body startup state, per-body transform evaluation, one uniform slot per body, render loop iteration over all body transforms, and ROM scene layout with multiple Platonic solids distributed across the screen.
    - Non-goals: full physics solver integration, collision response, generalized scene-graph abstraction, or per-body material differentiation.
  - Constraints and assumptions:
    - The current single-body implementation in `RuntimeState`, `TextureDemoRom`, and `render.rs` remains the baseline contract that this feature extends.
    - `BodyState` remains owned by the runtime and is not duplicated by value. The runtime holds body entries and per-body step counters rather than cloning state.
    - This feature should not introduce a new simulation abstraction; it should extend the established `BodyState` + `RuntimeState` pattern.
    - The startup scene should display one instance of each Platonic solid with distinct phase offsets and a shared texture, producing a clear visual distinction without introducing per-body asset routing yet.
  - Milestones:
    - M1: Replace the single-body ownership path with a body collection and confirm transform evaluation for every entry in the same fixed-step pass.
    - M2: Add per-body uniform slots and a render pass loop that writes transform data and draws each body in order.
    - M3: Extend ROM startup content to emit the full multi-body set and wire shell/runtime integration to the new body collection seam.
  - File-by-file implementation contract:
    - [anzu-engine/src/execution/shared.rs](anzu-engine/src/execution/shared.rs)
      - Action: modify
      - Why: the runtime already contains `BodyEntry`, `set_bodies`, `body_transforms`, and `apply_animation_step`; this feature should finish the multi-body contract and make the body collection the canonical runtime representation.
      - Structs/enums:
        - `struct BodyEntry { state: BodyState, step_index: u64 }`
      - Functions/methods:
        - `fn set_bodies(&mut self, bodies: Vec<BodyState>)`
        - `fn bodies(&self) -> &[BodyState]`
        - `fn body_count(&self) -> usize`
        - `fn body_transforms(&self) -> Vec<Matrix4<f32>>`
        - `fn apply_animation_step(&mut self)`
      - Ownership notes: `RuntimeState` owns the body collection and per-body step counters; it does not own GPU or renderer state.
    - [anzu-engine/src/execution/render.rs](anzu-engine/src/execution/render.rs)
      - Action: modify
      - Why: enable one uniform buffer and one draw call per body while leaving the shared mesh and texture resources on the textured renderer.
      - Structs/enums:
        - `struct BodyUniformSlot { uniform_buffer: wgpu::Buffer, uniform_bind_group: wgpu::BindGroup }`
      - Functions/methods:
        - `fn create_body_uniform_slots(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, count: usize) -> Vec<BodyUniformSlot>`
        - `fn encode_multi_body_textured_mesh_pass(...)` writes each transform and issues one `draw_indexed` per body within a single pass.
      - Ownership notes: geometry, pipeline, sampler, and texture resources stay on `TexturedMeshRenderer`; only per-body transform data is reuploaded each frame.
    - [anzu-engine/src/roms/texture_demo.rs](anzu-engine/src/roms/texture_demo.rs)
      - Action: modify
      - Why: the ROM should own the full initial body collection and return it once at startup for the runtime to ingest.
      - Structs/enums:
        - `struct TextureDemoRom { bodies: Option<Vec<BodyState>>, ... }`
      - Functions/methods:
        - `fn take_bodies(&mut self) -> Vec<BodyState>`
        - `fn initial_bodies() -> Vec<BodyState>`
        - `fn startup_body_specs() -> Vec<(Primitive, Vec<[f32; 4]>)>`
      - Ownership notes: ROM is the source of the scenario layout and transform closures; shell and runtime simply consume the resulting body list.
    - [anzu-engine/src/lib.rs](anzu-engine/src/lib.rs)
      - Action: modify
      - Why: app shell should build the runtime body collection once, allocate one uniform slot per body, and pass the transform list into the multi-body render entry point.
      - Functions/methods:
        - `State::new(...)` seeds `runtime_state.set_bodies(rom.take_bodies())` and creates `body_uniform_slots` sized to the body count.
        - `State::render(...)` calls the multi-body render helper and passes the transform list and slot list into the render path.
      - Ownership notes: shell remains the orchestrator; it does not define body motion policy or mesh semantics.
  - Execution flow and ownership boundaries:
    - `TextureDemoRom::new()` constructs the startup body collection with distinct transform closures.
    - `State::new()` transfers the body collection into `RuntimeState` and matches it to the number of per-body render slots.
    - Each fixed-step tick increments each body’s step index and produces a transform snapshot for all bodies.
    - The renderer writes one transform per body to its uniform slot and issues a draw call for that body in the same pass.
    - Shared mesh, pipeline, and texture resources are created once; only body transform data changes each frame.
  - Validation plan:
    - Unit tests:
      - `runtime_state_sets_multiple_bodies_and_exposes_transform_count`
      - `runtime_state_body_transforms_match_body_count`
      - `texture_demo_rom_take_bodies_returns_expected_body_count`
      - `texture_demo_rom_bodies_have_distinct_phase_offsets`
      - `render_pass_iterates_over_all_body_uniform_slots_without_mismatches`
    - Build checks:
      - `cd /workspaces/Anzu/anzu-engine && cargo test`
      - `cd /workspaces/Anzu/anzu-engine && cargo check --target wasm32-unknown-unknown`
    - Manual browser smoke:
      - verify five bodies are visible simultaneously with divergent motion patterns.
      - verify the projection toggle and background-color behavior remain intact.
  - Risks and open questions:
    - Risk: body-transform count mismatches between runtime and render slots can cause incorrect draws or hidden bodies. Mitigation: assert that the counts match before rendering and log an error early.
    - Risk: body overlap or poor projection placement can hide some objects. Mitigation: distribute bodies across X/Z offsets and validate in smoke testing.
    - Open question resolved by this plan: the initial multi-body scene is a shared-geometry/shared-texture system with distinct phase offsets rather than a richer per-body asset or material model.
- [ ] Feature 1.6: Handle surface resize, reconfigure, and present lifecycle safely
  - Follow-up scope addition:
    - Replace panic-path surface creation and setup calls with typed error boundaries and recoverable fallback behavior.
    - Acceptance criterion: no unwrap or expect in the surface acquire/reconfigure path in [anzu-engine/src/lib.rs](anzu-engine/src/lib.rs) and the future render-backend boundary.
- [ ] Feature 1.7: Maintain explicit frame contract acquire -> encode -> submit -> present
- [ ] Feature 1.8: Split app shell state from runtime orchestration state
- [ ] Feature 1.9: Split simulation-owned state from render backend state
- [ ] Feature 1.10: Add explicit render extraction state between simulation and renderer
- [ ] Feature 1.11: Keep long-lived render resources separate from per-frame transient state
- [ ] Feature 1.12: Migrate `lib.rs` to platform-shell ownership only (no gameplay mutation ownership)

### Initiative 1 State Decomposition Plan (lib.rs alignment)

```mermaid
flowchart LR
	A[AppState: platform shell adapter] --> B[RuntimeState: frame contract and control-plane state]
	B --> C[SimulationState: world state, body state, solver]
	B --> D[RenderExtractionState: frame packet]
	D --> E[RenderBackendState: surface, device, queue, config]
	E --> F[RenderResources: pipelines, buffers, textures, bind groups]
	E --> G[FrameState: transient encoder, view, pass]
```

### Initiative 1 BDD User Stories

1. Story: Platform shell ownership
Given the runtime starts on native or wasm
When the application initializes
Then `lib.rs` owns platform shell concerns only
And simulation mutation logic is outside app shell state

2. Story: Runtime orchestration boundary
Given a redraw or fixed-step trigger
When frame processing begins
Then `RuntimeState` enforces acquire -> encode -> submit -> present ordering
And stage ownership remains explicit and traceable

3. Story: Simulation ownership boundary
Given simulation ticks execute
When state mutation occurs
Then only `SimulationState` mutates gameplay/world state
And renderer paths consume read-only snapshot data

4. Story: Render extraction contract
Given a stabilized simulation snapshot
When extraction runs
Then `RenderExtractionState` emits a frame packet
And renderer input does not require direct ECS schema coupling

5. Story: Render backend lifecycle safety
Given surface resize, loss, or suboptimal present events
When render backend handles the event
Then `RenderBackendState` reconfigures safely without gameplay-state side effects
And surface lifecycle concerns stay isolated from simulation stages

6. Story: Resource lifecycle separation
Given long-lived pipelines, buffers, and textures
When frame rendering executes
Then `RenderResources` persist across frames
And `FrameState` contains transient per-frame objects only

7. Story: Migration acceptance
Given the state decomposition migration is complete
When reviewers inspect runtime boundaries
Then each state type has a single primary responsibility
And architecture terms match the core glossary in `documents/architecture.md`

## Initiative 2: ECS, Objects, and Extraction Boundaries
- [ ] Feature 2.1: ECS world with stable entity identity and component storage
- [ ] Feature 2.2: ObjectSpec domain with predefined catalog templates
  - Contract: `ObjectSpec` is an identifier-only catalog template resolved at profile load or entity spawn. It does not embed mutable `BodyState`, `DisplayState`, light, or material values.
  - Ownership: ROM manifests own template identifiers and content policy; engine registries resolve those identifiers into stable resource handles.
- [ ] Feature 2.3: ObjectRuntime domain with resolved resource handles and lifecycle state
  - Contract: `ObjectRuntime` binds entity identity, its originating object-spec identifier, resolved engine resource handles, and lifecycle state.
  - Ownership: simulation, display, and light state remain separate ECS components; `ObjectRuntime` does not become their mutation owner.
- [ ] Feature 2.4: BodyState and DisplayState split per object
  - Contract: `BodyState` and `DisplayState` are distinct per-entity components. Simulation owns their mutation, while render extraction reads a stabilized snapshot.
- [ ] Feature 2.5: Light attachments with explicit emitter specs
  - Contract: light attachments are optional ECS component data referencing explicit emitter specs. They do not own renderer or GPU resources.
- [ ] Feature 2.6: Extractor emits frame packet from ECS/object snapshot
  - Contract: extraction emits drawable and light records that are distinct from `DisplayState` and other ECS component schemas.
- [ ] Feature 2.7: Render consumes frame packet only, without direct ECS schema coupling
  - Contract: extracted frame records are the renderer's only object-state input; renderer code does not query ECS components directly.

## Initiative 3: Physics Foundation
- [ ] Feature 3.1: Canonical BodyState schema with motion, force, and mass properties
- [ ] Feature 3.2: Physics mesh handle on body state
- [ ] Feature 3.3: Collision shape defaults and validation policy
- [ ] Feature 3.4: Inverse mass and inverse inertia construction policy
- [ ] Feature 3.5: Precomputed mass properties from collision geometry at asset-build/load time
- [ ] Feature 3.6: Broadphase plus narrowphase staged collision pipeline
- [ ] Feature 3.7: Deterministic collision event ordering

## Initiative 4: Hybrid Physics Runtime (GPU + CPU Control Plane)
- [ ] Feature 4.1: GPU-resident body state buffers across ticks
- [ ] Feature 4.2: Compute pipeline for integration and interaction solving
- [ ] Feature 4.3: Compact interaction event stream from GPU to CPU
- [ ] Feature 4.4: ECS updates from authoritative simulation outputs
- [ ] Feature 4.5: Dual backend validation path (CPU reference and GPU runtime)
- [ ] Feature 4.6: Physics queue and dispatch orchestration in simulator runtime module

## Initiative 5: Mesh and Asset Strategy
- [ ] Feature 5.1: Dual-mesh model (physics mesh and display mesh)
- [ ] Feature 5.2: Shared physics mesh residency model using shape handles
- [ ] Feature 5.3: Instancing without duplicating shared physics mesh vertices
- [ ] Feature 5.4: Copy-on-change policy for topology mutations (fracture/remesh)
- [ ] Feature 5.5: Unified import pipeline that emits display and physics assets
- [ ] Feature 5.6: Asset handles in ECS instead of embedded vertex blobs

## Initiative 6: Materials, Lights, and Render Composition
- [ ] Feature 6.1: Material registry with data-defined material ids
  - Contract: the existing `execution::material::MaterialDefinition` is the seed for the engine-owned registry. This feature must not introduce a competing generic `Material` domain type without an explicit migration.
- [ ] Feature 6.2: Multi-material object rendering via submesh ranges
- [ ] Feature 6.3: Emitter model supporting directional, point, spot, area, environment, and surface emission
  - Contract: `EmitterType` contains exactly `Directional`, `Point`, `Spot`, `Area`, `Environment`, and `Surface` for this feature's acceptance scope.
  - Ownership: emitter specs are simulation/ECS data; extracted light records translate them for renderer consumption without transferring mutation ownership.
- [ ] Feature 6.4: Surface response model separated from emitter records
  - Contract: emitter records contain emission data only. Material definitions own surface-response properties independently.
- [ ] Feature 6.5: Pipeline keying and variant cache
- [ ] Feature 6.6: Composite render passes for emissive and post-process effects
- [ ] Feature 6.7: Low-poly visual strategy support through shading-first enhancement

## Initiative 7: Input and Control Plane
- [ ] Feature 7.1: Profile-owned input vocabularies and mappings
- [ ] Feature 7.2: Engine-owned context routing (gameplay and overlay contexts)
- [ ] Feature 7.3: Escape overlay toggle with deterministic pause behavior
- [ ] Feature 7.4: In-engine diagnostics/performance overlay
- [ ] Feature 7.5: Notification overlay for runtime/control-plane events
- [ ] Feature 7.6: Pause, single-step, and replay workflow for deterministic debugging

## Initiative 8: Simulation Domain Profiles
- [ ] Feature 8.1: Domain-profile registration model (games as one profile)
- [ ] Feature 8.2: Platformer profile bootstrap and acceptance scene
- [ ] Feature 8.3: Isometric 3D profile bootstrap and acceptance scene
- [ ] Feature 8.4: Multi-profile runtime switching in one session
- [ ] Feature 8.5: Engine-provided default simulation and physics model catalog
  - Outcome: The engine exposes a built-in catalog of simulation/physics models (for example arcade, rigid-body baseline, and deterministic reference modes) that ROM developers can select without writing custom solvers.
  - Scope:
    - In scope: model identifiers, registration/discovery APIs, startup model selection in ROM profile config, and deterministic fallback behavior when no custom model is supplied.
    - Non-goals: replacing advanced custom models, full editor UI for model tuning, and cross-network model negotiation.
  - Acceptance criteria:
    - ROM profile can select one engine-default model by identifier at load time.
    - Runtime boots and simulates using selected model with deterministic configuration handling.
    - Invalid model ids fail at load boundary with actionable diagnostics.
- [ ] Feature 8.6: ROM-supplied custom simulation and physics model interface
  - Outcome: ROM developers can provide custom simulation/physics implementations through an engine-defined interface, and the runtime executes them alongside engine lifecycle/control-plane constraints.
  - Scope:
    - In scope: trait/interface contract, lifecycle hooks (init/tick/extract), registration path, capability/limits validation, and fallback to engine-default models.
    - Non-goals: arbitrary unsafe host access, bypassing determinism checks, and direct mutation of render backend internals.
  - Constraints and assumptions:
    - Engine owns scheduling, safety, and deterministic stepping boundaries.
    - ROM custom models own domain logic but must conform to engine contracts for inputs/outputs/state serialization.
  - Acceptance criteria:
    - ROM can register a custom model and run simulation ticks through engine scheduler.
    - Engine can run A/B parity checks between custom and default reference paths when configured.
    - Failure to satisfy interface contracts produces deterministic startup errors with clear diagnostics.

## Initiative 9: Performance, Determinism, and Quality Gates
- [ ] Feature 9.1: Frame boundary instrumentation and health counters
  - Outcome: active runtime emits repeatable frame-level and workload measurements that establish a baseline for later optimization.
  - Initial measurements: frame duration, fixed-tick count, dropped/clamped time, and available simulation/extraction work counts.
  - Instrumentation policy: keep low-cost counters available continuously; sample or explicitly enable high-resolution stage timings. Measure and report instrumentation overhead.
  - Acceptance criteria:
    - Baseline output identifies target, backend, workload, warm-up, and sample count.
    - Timing summaries report p50, p95, and p99 with workload counts.
    - CPU stage timing is distinct from GPU timing. Unsupported GPU timing is reported as unavailable, not inferred from CPU submission time.
    - A repeat run on the same target and workload produces comparable results, with noise and limitations stated.
- [ ] Feature 9.2: Deterministic replay regression suite
- [ ] Feature 9.3: Upload budget and pipeline-switch telemetry
  - Outcome: runtime emits per-frame streaming and pipeline-churn telemetry so budget tuning is evidence-driven.
  - Metrics (required):
    - `upload_bytes_per_frame`
    - `upload_jobs_per_frame`
    - `decode_jobs_per_frame`
    - `activation_jobs_per_frame`
    - `gpu_resident_bytes`
    - `eviction_count_per_frame`
    - `pipeline_switch_count_per_frame`
    - `visible_missing_asset_count`
  - Acceptance criteria:
    - Metrics are available in both native and wasm builds.
    - A 5-minute traversal run produces stable metric output without missing samples.
    - Budget overruns emit structured warnings with asset class and queue tier labels.
- [ ] Feature 9.4: Startup-to-first-frame budget checks
- [ ] Feature 9.5: Warning-free CI gate for runtime-critical modules
- [ ] Feature 9.6: Unified benchmark output for simulation and extraction phases
  - Outcome: benchmark runs produce comparable, machine-readable simulation and render-extraction measurements.
  - Acceptance criteria:
    - Output includes phase timing distributions, work counts, target/backend, workload parameters, warm-up, sample count, and instrumentation mode.
    - Before/after results use the same workload and record raw output.
    - Performance claims separate algorithmic changes from layout/copy/allocation changes and report measurement overhead.

## Initiative 10: Asset Streaming and Residency Management
- [ ] Feature 10.1: Asset class taxonomy (critical, scaled_optional, reused, streaming)
- [ ] Feature 10.2: Residency state machine (queued, loading, resident, eviction pending, evicted)
- [ ] Feature 10.3: Streaming queues with bounded per-frame upload budgets
  - Default budgets (v1):
    - wasm: `max_upload_bytes_per_frame = 4 MiB`
    - native: `max_upload_bytes_per_frame = 16 MiB`
    - `max_upload_jobs_per_frame = 32`
    - `max_decode_jobs_per_frame = 2` (wasm) / `4` (native)
    - `max_activation_jobs_per_frame = 32`
  - Queue tiers and priority:
    - `critical_stream`: visible-now assets only.
    - `normal_decode`: near-field prefetch assets.
    - `background_cleanup`: eviction and compaction tasks.
  - Acceptance criteria:
    - Upload and decode work never exceeds configured per-frame caps.
    - Traversal under load maintains simulation tick continuity without upload-triggered stalls.
    - Queue starvation protection guarantees at least one `critical_stream` job admission per frame when pending.
- [ ] Feature 10.4: Budget-aware quality fallback before correctness-impacting eviction
  - Fallback policy order:
    1. Reduce optional texture resolution.
    2. Reduce optional mesh LOD.
    3. Disable non-critical material features.
    4. Evict `scaled_optional` before `reused` before `critical`.
  - Acceptance criteria:
    - Under forced memory pressure, fallback activates before evicting `critical` assets.
    - Quality recovery occurs automatically when pressure clears and budgets permit.
    - Fallback transitions are deterministic for identical traversal inputs.
- [ ] Feature 10.5: Residency and eviction observability per asset class
  - Required counters by class (`critical`, `scaled_optional`, `reused`, `streaming`):
    - resident count
    - resident bytes
    - load latency p50/p95
    - eviction count
    - reactivation count
  - Acceptance criteria:
    - Counters are queryable at runtime and captured in benchmark output.
    - Eviction thrash detection triggers warning when an asset is evicted and reactivated more than 3 times in 60 seconds.

## Initiative 11: Platform Topology and ROM/Module Boundaries
- [ ] Feature 11.1: Host/guest service contract (input, render extraction, audio events, storage, lifecycle)
- [ ] Feature 11.2: Runtime loader path with compatibility and integrity checks
- [ ] Feature 11.3: Hash-addressed content cache and activation rollback safety
- [ ] Feature 11.4: Namespace isolation for persisted profile data
- [ ] Feature 11.5: Externalized ROM content manifests with schema validation and runtime loading
  - Outcome: ROM content definitions (objects, geometry selectors, colors, textures, and profile-specific startup configuration) are loaded from external manifest data instead of hardcoded Rust literals.
  - Blender export profile checklist (first pass):
    - Export format is GLB (`.glb`) from Blender.
    - Transform is applied before export (location/rotation/scale baked).
    - Unit scale is consistent across all exported objects in the profile.
    - Triangulation is applied before export.
    - Mesh normals are exported and validated as present.
    - UV0 is present for textured meshes.
    - Object names are stable and unique per manifest package.
    - Material slot names are stable and deterministic.
    - Texture paths are relative to ROM content root, not absolute filesystem paths.
    - A startup scene manifest references only assets that exist in `anzu-engine/src/roms/files/objects` and `anzu-engine/src/roms/files/textures`.
  - Scope:
    - In scope: manifest schema for ROM content, parser/loader path, validation errors with actionable diagnostics, and runtime wiring so ROM startup content comes from manifest payloads.
    - Non-goals: remote/network manifest loading, hot-reload tooling, and cross-profile live editing UI.
  - Constraints and assumptions:
    - Engine runtime remains content-agnostic and only consumes validated manifest payloads.
    - ROM remains owner of content policy; engine owns generic validation, transport, and execution plumbing.
    - ROM manifest identifiers remain distinct from engine-owned resource handles and are translated at the profile-load boundary.
    - Manifest format must be deterministic and versioned for forward migration compatibility.
    - Failure mode is fail-fast at ROM/profile load boundary with explicit error taxonomy.
  - Manifest schema contract (v1, implementation-ready):
    - File: `anzu-engine/src/roms/files/manifest/texture_demo.scene.json`
    - Top-level required fields:
      - `manifest_version: "1"`
      - `profile_id: string`
      - `objects: ObjectRecord[]`
      - `textures: TextureRecord[]`
      - `startup_scene: StartupScene`
    - `ObjectRecord` required fields:
      - `object_asset_id: string` (unique across objects)
      - `source: { kind: "glb", path: string }` (path relative to `anzu-engine/src/roms/files/objects`)
      - `default_material_id: string`
      - `generate_body: bool`
    - `TextureRecord` required fields:
      - `texture_id: string` (unique across textures)
      - `path: string` (relative to `anzu-engine/src/roms/files/textures`)
      - `color_space: "srgb" | "linear"`
    - `StartupScene` required fields:
      - `instances: StartupInstance[]`
    - `StartupInstance` required fields:
      - `instance_id: string` (unique in scene)
      - `object_asset_id: string` (must resolve to `objects[].object_asset_id`)
      - `transform: { translation: [f32; 3], rotation_euler_deg: [f32; 3], scale: [f32; 3] }`
      - `texture_id: string` (must resolve to `textures[].texture_id`)
      - `animation: { mode: "none" | "orbit" | "spin", phase_offset: f32 }`
    - Determinism rules:
      - Object, texture, and instance arrays are consumed in declared order.
      - Duplicate IDs are hard errors.
      - Unknown fields are rejected in strict mode for v1.
    - Compact example (`texture_demo.scene.json`):
      ```json
      {
        "manifest_version": "1",
        "profile_id": "texture_demo",
        "objects": [
          {
            "object_asset_id": "tetra_a",
            "source": { "kind": "glb", "path": "platonic/tetrahedron.glb" },
            "default_material_id": "mat_default",
            "generate_body": true
          }
        ],
        "textures": [
          {
            "texture_id": "checker_01",
            "path": "checker/checker_01.png",
            "color_space": "srgb"
          }
        ],
        "startup_scene": {
          "instances": [
            {
              "instance_id": "tetra_a_0",
              "object_asset_id": "tetra_a",
              "transform": {
                "translation": [0.0, 0.0, 0.0],
                "rotation_euler_deg": [0.0, 0.0, 0.0],
                "scale": [1.0, 1.0, 1.0]
              },
              "texture_id": "checker_01",
              "animation": { "mode": "spin", "phase_offset": 0.0 }
            }
          ]
        }
      }
      ```
  - Validation failure taxonomy (typed error targets):
    - `ManifestVersionUnsupported { found }`
    - `ManifestMissingField { field_path }`
    - `ManifestUnknownField { field_path }`
    - `DuplicateObjectAssetId { id }`
    - `DuplicateTextureId { id }`
    - `DuplicateInstanceId { id }`
    - `ObjectPathOutsideRoot { path }`
    - `TexturePathOutsideRoot { path }`
    - `ObjectFileNotFound { object_asset_id, path }`
    - `TextureFileNotFound { texture_id, path }`
    - `ObjectSourceKindUnsupported { object_asset_id, kind }`
    - `ObjectReferenceNotFound { instance_id, object_asset_id }`
    - `TextureReferenceNotFound { instance_id, texture_id }`
    - `TransformScaleInvalid { instance_id, scale }`
    - `GlbMissingNormals { object_asset_id }`
    - `GlbMissingUv0 { object_asset_id }`
    - `GlbParseFailed { object_asset_id, reason }`
    - `StartupSceneEmpty`
  - File-by-file implementation contract (GLB-first):
    - [anzu-engine/src/roms/files/objects](anzu-engine/src/roms/files/objects)
      - Action: reference/add content
      - Why: source location for Blender-exported GLB object files owned by ROM content.
      - Contract: each manifest `object_asset_id` resolves to one GLB file in this tree.
    - [anzu-engine/src/roms/files/textures](anzu-engine/src/roms/files/textures)
      - Action: reference/add content
      - Why: source location for texture payloads referenced by manifest object/material records.
      - Contract: each manifest texture reference resolves to one file under this tree.
    - [anzu-engine/src/roms/texture_demo.rs](anzu-engine/src/roms/texture_demo.rs)
      - Action: modify
      - Why: replace hardcoded startup content with manifest-backed startup payload loading.
      - Contract: ROM profile requests manifest load, receives validated records, and emits startup body/object specifications from manifest data.
    - [anzu-engine/src/assets.rs](anzu-engine/src/assets.rs)
      - Action: modify
      - Why: host typed manifest-facing asset records used by loader/provider boundaries.
      - Contract: define deterministic asset record shapes for object/mesh/material/texture references.
    - [anzu-engine/src/execution/shared.rs](anzu-engine/src/execution/shared.rs)
      - Action: modify
      - Why: runtime consumes validated startup content without owning parser details.
      - Contract: runtime accepts startup asset references through typed inputs and rejects unresolved IDs before frame loop starts.
    - [anzu-engine/src/lib.rs](anzu-engine/src/lib.rs)
      - Action: modify
      - Why: shell owns profile bootstrap sequencing and fail-fast error boundary.
      - Contract: app bootstrap performs manifest load and validation before runtime enters ready state.
    - [documents/developer-guide.md](documents/developer-guide.md)
      - Action: modify
      - Why: document Blender export checklist and manifest authoring path for contributors.
      - Contract: include deterministic authoring steps and common validation failures.
  - Milestones (implementation ready):
    - M1: Define manifest schema and typed error taxonomy for GLB object and texture references.
    - M2: Implement parser + validator path with deterministic ID/path resolution and startup fail-fast behavior.
    - M3: Wire texture-demo startup to manifest payloads and remove equivalent hardcoded object definitions.
    - M4: Add regression tests for valid/invalid manifest cases and verify native + wasm checks.
  - Acceptance criteria:
    - At least one active ROM profile (starting with texture demo) boots entirely from external manifest data.
    - Invalid manifest cases (missing required fields, out-of-range color channels, malformed indices, unknown primitive selectors) fail validation with deterministic errors.
    - Existing runtime checks pass with manifest-backed content path enabled.
    - Valid Blender-exported GLB object files referenced by manifest are visible at runtime with deterministic startup behavior.
    - Invalid GLB reference paths, missing files, and duplicate object IDs fail at load boundary with actionable diagnostics.
  - Completion evidence:
    - `cd /workspaces/Anzu/anzu-engine && cargo test`
    - `cd /workspaces/Anzu/anzu-engine && cargo check --target wasm32-unknown-unknown`
    - Manual smoke: update manifest color/content values and verify runtime startup reflects manifest changes without source edits.
- [ ] Feature 11.6: ROM-to-engine asset provider registration (mesh, texture, material)
  - Outcome: ROM-provided asset payloads are ingested once at profile load and registered into engine-owned provider registries, producing stable asset handles used by runtime systems.
  - Scope:
    - In scope: loader path from ROM payload to provider registries, provider APIs for mesh/texture/material records, typed handle allocation, and deterministic lookup behavior.
    - Non-goals: per-frame dynamic asset mutation, network streaming, and editor-driven hot reload.
  - Constraints and assumptions:
    - ROM remains owner of content policy and source payloads.
    - Engine remains owner of provider storage, validation, residency bookkeeping, and runtime handles.
    - Provider registries must be content-agnostic and reusable across profiles.
    - After registration, simulation and render paths reference engine resource handles only and do not retain ROM manifest payload dependencies.
  - Acceptance criteria:
    - A texture-demo profile can register mesh, texture, and material records through engine providers before first render.
    - Runtime simulation/render paths reference provider handles only (no direct ROM payload dependency in frame loop).
    - Missing or invalid asset references fail deterministically at load boundary with actionable diagnostics.
- [ ] Feature 11.7: Startup-only asset staging and pack-once GPU upload contract
  - Outcome: CPU-side vertex packing and texture decode/upload for static startup assets occur once during profile load, not during steady-state frame execution.
  - Scope:
    - In scope: explicit startup staging phase, immutable startup asset snapshot, and checks that frame loop uses prebuilt buffers/bind groups.
    - Non-goals: runtime asset streaming/eviction and live asset edits.
  - Constraints and assumptions:
    - Packing copies needed for GPU-interleaved layout are allowed during load stage only.
    - Frame rendering may only reference startup-prepared GPU resources and engine resource handles.
    - Any additional packing during frame execution is treated as regression unless explicitly approved.
  - Acceptance criteria:
    - Startup path creates required GPU resources exactly once for texture-demo baseline content.
    - Frame loop contains no asset decode, no provider ingest, and no startup-packing operations.
    - Tests/telemetry can detect accidental per-frame repacking behavior.

## Initiative 12: Networking and Session Features
- [ ] Feature 12.1: Session lifecycle (join, leave, active participants)
- [ ] Feature 12.2: Authoritative shared-state sync baseline
  - Prototype progress toward Features 12.1 and 12.2 in `anzu-runtime/src/main.rs`: the session creates one player and sends `AddActor` through an in-process channel; the server adds the actor, increments the world tick, and publishes a snapshot for the session.
  - Not complete: `cargo check --manifest-path anzu-runtime/Cargo.toml` fails because `Server::new` does not initialize its `simulation` field. The prototype has no leave flow or running simulation loop, and it has no synchronization tests.
- [ ] Feature 12.3: Client interpolation for remote state smoothing
- [ ] Feature 12.4: Transport abstraction with local loopback and websocket paths
- [ ] Feature 12.5: Transport and synchronization observability

## Initiative 13: Persistence and Serialization
- [ ] Feature 13.1: Local save/load state snapshots
- [ ] Feature 13.2: Snapshot versioning and migration compatibility
- [ ] Feature 13.3: Scene and manifest schema validation
- [ ] Feature 13.4: Deterministic snapshot restore and replay continuity

## Initiative 14: Audio Service
- [ ] Feature 14.1: Core audio service boundary and lifecycle
- [ ] Feature 14.2: Minimal synthesized cue slice (beep/boop)
- [ ] Feature 14.3: Streamed audio asset path
- [ ] Feature 14.4: Spatial audio baseline and attenuation controls
- [ ] Feature 14.5: Audio timing and underrun observability

## Initiative 15: Developer Tooling and Diagnostics
- [ ] Feature 15.1: Build and environment setup automation
- [ ] Feature 15.2: Asset manifest generation and validation tooling
- [ ] Feature 15.3: Runtime regression test harness
- [ ] Feature 15.4: Runtime entity/system inspection overlays
- [ ] Feature 15.5: Structured logging and trace hooks

## Initiative 16: Advanced Animation and Compute Extensions
- [ ] Feature 16.1: Skeletal data ingestion and canonicalization
- [ ] Feature 16.2: GPU skinning baseline render path
- [ ] Feature 16.3: Compute-driven animation inference pipeline
- [ ] Feature 16.4: Multi-character batching and scaling strategy
- [ ] Feature 16.5: Animation debug visualization and parity validation

## Initiative 17: Security and Access Control
- [ ] Feature 17.1: Deny-by-default authorization policy
- [ ] Feature 17.2: Per-request authorization enforcement
- [ ] Feature 17.3: Integrity verification for remotely loaded content
- [ ] Feature 17.4: Auth failure and policy observability

## Priority Bands
- P0: Runtime boundaries, extraction model, physics core, hybrid simulation loop, render/material/light foundations, deterministic quality gates
- P1: Streaming/residency, profile switching, tooling, persistence, audio, platform topology hardening
- P2: Extended networking scale, advanced animation compute features, optional high-complexity effects

## Notes
- This backlog intentionally retains planned feature scope from the prior roadmap where compatible with the simulation-first architecture discussed in this conversation.
- Feature completion history from legacy generated code is not used as progress evidence.
