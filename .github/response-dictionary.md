# Response Dictionary

Purpose
- Keep language consistent across agent chat and written output.
- Control vocabulary scope by domain.

Usage rules
- Use Approved Terms in responses.
| asset manifest | anzu | Manifest model for asset records and runtime settings | Relevant to asset tooling and validation |
| asset record | anzu | Single asset entry in a manifest-backed registry | |
| manifest loader | anzu | Loader interface for manifest retrieval and parse | |
- If a better term is needed, add it to Pending Terms.
- Move terms from Pending to Approved only through pull request review.
| input control map | anzu | Map of controls to semantic actions | |
| input action event | anzu | Action-level input event after control mapping | |
| input context stack | anzu | Stack of active input context identifiers | |
- If a term already exists but has overloaded meanings, add it to Ambiguous Terms first.
| control command | anzu | Engine command for overlay and control-plane behavior | |
| control-plane action | anzu | Action emitted by control-plane transitions | |
| control-plane state | anzu | State owner for overlay, notifications, and ROM context stack | |
| overlay visibility | anzu | Visibility state for overlay surfaces and tabs | |
| input diagnostics snapshot | anzu | Point-in-time diagnostics capture for input state | |
- Resolve Ambiguous Terms through pull request review before broad usage.

## Approved Terms

### Core runtime vocabulary
| rom package | anzu | Runtime module package that boots world and simulation | Use instead of bare ROM when specific |
| simulation model | anzu | Simulation behavior interface used by runtime orchestration | |
| input context request | anzu | Simulation request to Push, Pop, or Replace active context | |
| Term | Scope | Meaning | Notes |
|---|---|---|---|
| simulation | core | State update logic for the runtime model | Use instead of game loop when domain-neutral context is needed |
| extraction | core | Transformation of stable simulation state into render-ready data | |
| mesh asset id | anzu | Stable key for mesh asset lookup | |
| material id | anzu | Stable key for material asset lookup | |
| runtime pipeline | core | Ordered execution stages in a frame | |
| adapter | core | Platform bridge layer around core runtime behavior | Keep adapters thin |
| control plane | core | Runtime control state and orchestration inputs | |
| simulation and visualization system | core | Domain-neutral engine core | Preferred engine-core label |
| domain profile | core | Concrete application of the core runtime | Games are one profile |
| physics config | anzu | Physics bounds and default material behavior configuration | |
| collision bounds | anzu | Coarse bounds used for candidate collision checks | |
| polygon collider | anzu | Polygon-based collision component or shape definition | |
| rigid body | anzu | Dynamic body record participating in physics solve | |
| entity | core | Stable identity across runtime stages | |
| state | core | Time-varying simulation-owned data | |
| solver | core | Deterministic numerical stage that advances simulation | |
| collision manifold | anzu | Contact normal and penetration details for resolution | |
| render extraction | core | Read-only translation from simulation snapshot to draw data | |
| resource handle | core | Stable ID for an asset or GPU resource | Prefer handles over embedded blobs |
| platform shell | core | Browser or window integration layer | |
| runtime loop | core | Frame orchestration and fixed-tick control | |
| interaction event | anzu | Discrete interaction/collision event record | |
| interaction event kind | anzu | Classification enum for interaction event semantics | |
| world state | core | Source of truth for simulation-owned data | |
| scheduler | anzu | Simulation step orchestrator for staged physics flow | |
| scheduler frame report | anzu | Per-frame simulation metrics and event report | |
| role-pair policy | anzu | Collision policy mapping by interacting role pairs | |
| collision mode | anzu | Mode describing collision outcome handling | |
| collision action | anzu | Action selected from collision policy evaluation | |
| fragment request | anzu | Request to split or replace object fragments | |
| hitbox | anzu | Collision-only role without impulse resolution | Keep distinct from solid body |
| frame contract | core | Acquire -> encode -> submit -> present lifecycle | Use explicit stage naming |
| fixed timestep | core | Deterministic step duration for simulation | |
| variable timestep | core | Frame-duration driven update path | Use carefully for determinism |
| skeletal ingestion | anzu | Import and normalization of skeletal animation data | |
| canonicalization | anzu | Conversion to standard internal representation | |
| GPU skinning | anzu | Vertex skin deformation on GPU pipeline | |
| compute-driven animation | anzu | Animation inference or update via compute pipeline | |
| multi-character batching | anzu | Batch strategy for many animated actors | |
| parity validation | anzu | Cross-path validation for equivalent outputs | |
| deny-by-default | anzu | Access model where all actions are denied unless explicitly allowed | |
| per-request authorization | anzu | Authorization check at each request boundary | |
| auth failure observability | anzu | Metrics and traces for authorization failures | |

### Rust vocabulary
| Term | Scope | Meaning | Notes |
|---|---|---|---|
| ownership | rust | Value lifetime and move semantics model | |
| borrowing | rust | Reference-based access without transfer of ownership | |
| Result | rust | Recoverable error type | Use with explicit error propagation |
| enum | rust | Closed set of variants for state/data modeling | |
| trait object | rust | Runtime polymorphism via dyn Trait | Use only when runtime dispatch is required |
| lifetime | rust | Compile-time relation that constrains reference validity | |
| move | rust | Ownership transfer of a value | |
| copy | rust | Bitwise duplicate for Copy types | |
| clone | rust | Explicit duplicate operation for Clone types | Audit in hot paths |
| reference | rust | Borrowed pointer-like access with aliasing rules | |
| mutable reference | rust | Exclusive borrowed access for mutation | |
| immutable reference | rust | Shared borrowed read access | |
| slice | rust | Borrowed view into contiguous elements | |
| str slice | rust | Borrowed UTF-8 string view (`&str`) | |
| String | rust | Owned UTF-8 string buffer | |
| Vec | rust | Growable contiguous sequence | |
| HashMap | rust | Hash table map with average O(1) lookup | |
| BTreeMap | rust | Ordered map with range query support | |
| Option | rust | Optional value type with Some or None | |
| match | rust | Exhaustive pattern-matching control flow | |
| if let | rust | Single-pattern shorthand branch | |
| while let | rust | Single-pattern loop branch | |
| iterator adapter | rust | Composable iterator transformation operation | |
| iter | rust | Shared-reference iterator method | |
| iter_mut | rust | Mutable-reference iterator method | |
| into_iter | rust | Ownership-consuming iterator method | |
| generic | rust | Type-parameterized compile-time polymorphism | |
| monomorphization | rust | Per-concrete-type code generation for generics | |
| where clause | rust | Readable location for complex trait bounds | |
| trait bound | rust | Interface constraint on generic parameters | |
| associated type | rust | Trait-defined type placeholder | |
| newtype | rust | Single-field wrapper for domain meaning and safety | |
| zero-cost abstraction | rust | Abstraction compiled without runtime overhead in common cases | |
| panic | rust | Unrecoverable abort path for invariant violation | Avoid in production hot paths |
| question-mark operator | rust | Error propagation operator (`?`) for Result and Option | |
| unwrap | rust | Panic-on-error extraction helper | Avoid in production paths |
| expect | rust | Panic-on-error extraction with message | Avoid in production paths |
| unsafe block | rust | Scope where unsafe operations are permitted | Require stated safety invariants |
| safety invariant | rust | Condition that must hold for unsafe correctness | |
| Send | rust | Marker trait for thread-transfer safety | |
| Sync | rust | Marker trait for shared-reference thread safety | |
| interior mutability | rust | Mutation through shared references using runtime checks or atomics | |
| RefCell | rust | Single-threaded interior mutability with borrow checking at runtime | |
| Mutex | rust | Mutual exclusion primitive for synchronized mutation | |
| Arc | rust | Atomically reference-counted shared ownership pointer | |
| Box | rust | Heap allocation owning pointer | |
| Box dyn Trait | rust | Heap-allocated trait object for runtime dispatch | |
| static dispatch | rust | Compile-time dispatch via generics | |
| dynamic dispatch | rust | Runtime dispatch via trait objects | |
| async | rust | Syntax for defining poll-based asynchronous computations | |
| Future | rust | Poll-based asynchronous computation trait | |
| pinning | rust | Stability guarantee for memory location of self-referential futures | |
| channel | rust | Message-passing primitive for ownership transfer | |
| crate | rust | Compilation unit and package code root | |
| module | rust | Namespace and code organization unit | |
| prelude | rust | Common imported items set | |
| derive | rust | Attribute-based automatic trait implementation | |
| macro | rust | Compile-time code generation mechanism | Use when clearer than functions/types |
| const generic | rust | Compile-time constant parameter in type signatures | |
| no_std | rust | Standard-library-free Rust environment mode | |

### Rust error and reliability vocabulary
| Term | Scope | Meaning | Notes |
|---|---|---|---|
| typed error | rust | Explicit error type for recoverable failure modes | |
| error boundary | rust | API boundary where errors are converted or classified | |
| fallback behavior | rust | Defined behavior when normal path cannot proceed | |
| invariant | rust | Condition that must always hold for correctness | |
| deterministic behavior | rust | Equivalent inputs yield equivalent outputs | |
| benchmark evidence | rust | Measured data supporting optimization decisions | |

### wgpu vocabulary
| Term | Scope | Meaning | Notes |
|---|---|---|---|
| frame acquire | wgpu | Acquire the next render target for a frame | |
| command encoding | wgpu | Record GPU commands into command buffers | |
| queue submission | wgpu | Submit encoded commands to the GPU queue | |
| present | wgpu | Display rendered output to surface | |
| backend constraint | wgpu | Platform/backend-specific behavior limit | Distinguish native and browser paths |
| adapter request | wgpu | Selection of physical GPU capability endpoint | |
| device | wgpu | Logical GPU handle for resource and pipeline creation | |
| queue | wgpu | Submission endpoint for command buffers and writes | |
| surface | wgpu | Presentation target bound to platform window or canvas | |
| surface configuration | wgpu | Format, present mode, and size configuration record | |
| texture format | wgpu | Pixel format definition for textures and attachments | |
| present mode | wgpu | Swap behavior policy for frame presentation | |
| alpha mode | wgpu | Surface alpha compositing behavior | |
| view format | wgpu | Alternate format view compatibility for textures | |
| command encoder | wgpu | Recorder for render and compute command streams | |
| command buffer | wgpu | Finalized encoded GPU command sequence | |
| render pass | wgpu | Scoped set of graphics commands for attachments | |
| compute pass | wgpu | Scoped set of compute dispatch commands | |
| pipeline layout | wgpu | Bind-group layout sequence and push-constant ranges | |
| bind group layout | wgpu | Binding interface declaration for shader resources | |
| bind group | wgpu | Concrete resource binding set matching a layout | |
| bind slot | wgpu | Binding index in a bind-group layout | |
| shader module | wgpu | Compiled or validated shader program unit | |
| WGSL | wgpu | WebGPU Shading Language used for shader authoring | |
| naga translation | wgpu | Shader translation and validation path through naga | |
| render pipeline | wgpu | Graphics pipeline state object | |
| compute pipeline | wgpu | Compute pipeline state object | |
| pipeline cache | wgpu | Cached pipeline compilation artifacts where supported | |
| vertex buffer | wgpu | Buffer bound as vertex input stream | |
| index buffer | wgpu | Buffer bound for indexed draw calls | |
| uniform buffer | wgpu | Small frequently-updated constant-data buffer | |
| storage buffer | wgpu | Read-write shader data buffer | |
| staging buffer | wgpu | CPU-visible transfer buffer for uploads and readback | |
| mapped range | wgpu | CPU-accessible memory range of a mapped buffer | |
| buffer usage flags | wgpu | Declared usage capabilities of a buffer resource | |
| texture usage flags | wgpu | Declared usage capabilities of a texture resource | |
| texture view | wgpu | Subresource view into a texture for binding/attachment | |
| sampler | wgpu | Texture filtering and addressing descriptor object | |
| depth stencil | wgpu | Depth and stencil attachment state and operations | |
| color attachment | wgpu | Render target attachment for color output | |
| load op | wgpu | Attachment load behavior at pass start | |
| store op | wgpu | Attachment store behavior at pass end | |
| clear value | wgpu | Value used when clearing an attachment | |
| draw call | wgpu | Graphics command that emits primitives | |
| indexed draw | wgpu | Draw using index buffer indirection | |
| instancing | wgpu | Reuse geometry with per-instance attributes | |
| indirect draw | wgpu | Draw arguments supplied by GPU buffer | |
| dispatch | wgpu | Compute workload launch command | |
| workgroup | wgpu | Cooperative thread group in compute shader | |
| barrier | wgpu | Ordering and visibility synchronization requirement | Model with API-supported scopes |
| synchronization hazard | wgpu | Read-write ordering risk across passes or queues | |
| resource residency | wgpu | Availability of GPU resources for active usage | |
| upload bandwidth | wgpu | CPU-to-GPU transfer throughput budget | |
| readback | wgpu | GPU-to-CPU transfer path | Keep sparse where possible |
| frame pacing | wgpu | Cadence stability of frame presentation and submission | |
| surface loss | wgpu | Invalidated surface requiring reconfiguration | |
| reconfigure | wgpu | Apply new surface configuration after resize or loss | |
| multisampling | wgpu | Anti-aliasing strategy using multiple samples per pixel | |
| MRT | wgpu | Multiple render targets written in one pass | |
| push constants | wgpu | Small fast shader constants in pipeline layout ranges | Backend support varies |
| timestamp query | wgpu | GPU timing query mechanism | |
| occlusion query | wgpu | Visibility query mechanism | |
| feature flag | wgpu | Optional device capability toggle | |
| limits | wgpu | Device capability numeric bounds | |
| fallback path | wgpu | Alternative path for capability-limited backends | |
| WebGPU path | wgpu | Browser-native WebGPU backend route | |
| WebGL fallback | wgpu | Compatibility backend route through translation layers | |
| native backend | wgpu | Vulkan, Metal, DX12, or GLES native path | |
| shader portability | wgpu | Cross-backend shader compatibility behavior | |
| validation error | wgpu | API contract error reported by validation layers | |
| frame graph | wgpu | Logical dependency graph of render and compute passes | Use when explicit pass dependencies are modeled |

### Physics and simulation vocabulary
| Term | Scope | Meaning | Notes |
|---|---|---|---|
| broadphase | anzu | Coarse collision candidate generation stage | |
| narrowphase | anzu | Precise collision resolution stage | |
| collision shape | anzu | Geometry proxy used for collision queries | |
| mass property | anzu | Computed mass, center, and inertia terms | |
| inverse mass | anzu | Reciprocal mass term used in integration and solve steps | |
| inverse inertia | anzu | Reciprocal inertia term used in rotational solve steps | |
| SAT | anzu | Separating axis theorem collision test method | |
| interaction stream | anzu | Compact interaction or collision event stream | |
| authoritative output | anzu | Simulation output treated as canonical state | |
| dual backend validation | anzu | CPU reference and GPU runtime comparison path | |
| dispatch orchestration | anzu | Runtime coordination of compute dispatch sequence | |

### Asset and residency vocabulary
| Term | Scope | Meaning | Notes |
|---|---|---|---|
| physics mesh | anzu | Canonical local-space collision geometry | Shared by many bodies |
| display mesh | anzu | Visual mesh used for rendering | |
| shape id | anzu | Stable key for shared collision shape data | |
| body id | anzu | Stable key for dynamic body state record | |
| copy-on-change | anzu | Duplicate topology only when geometry changes | |
| topology mutation | anzu | Geometry connectivity change such as fracture or remesh | |
| prefetch | anzu | Early load of likely-soon-needed content | |
| hysteresis | anzu | Controlled delay to avoid rapid load/unload thrashing | |
| eviction pending | anzu | Residency transition state before asset removal | |
| pressure-aware policy | anzu | Budget-sensitive policy that scales quality and residency | |

### Legacy clanker_code vocabulary
| Term | Scope | Meaning | Notes |
|---|---|---|---|
| clanker_code | anzu-legacy | Legacy generated/runtime prototype module tree | Allowed for vocabulary mining |
| AssetManifest | anzu-legacy | Manifest model for asset records and settings | |
| ManifestSettings | anzu-legacy | Manifest-level runtime settings bundle | |
| PhysicsSettings | anzu-legacy | Physics configuration section from manifest | |
| AssetRecord | anzu-legacy | Single asset entry in registry data | |
| AssetRegistry | anzu-legacy | Runtime view of manifest-backed asset records | |
| ManifestLoadError | anzu-legacy | Manifest parsing or validation failure set | |
| AssetManifestLoader | anzu-legacy | Loader trait for manifest retrieval | |
| WebAssetManifestLoader | anzu-legacy | Browser-backed manifest loader implementation | |
| EntityId | anzu-legacy | Legacy ECS entity identifier | |
| Transform | anzu-legacy | Legacy transform component record | |
| MeshAssetId | anzu-legacy | Legacy mesh handle newtype | |
| MaterialId | anzu-legacy | Legacy material handle newtype | |
| MeshInstance | anzu-legacy | Legacy mesh instance component | |
| CollisionBounds | anzu-legacy | Legacy broad bounds record for collision checks | |
| PolygonCollider | anzu-legacy | Legacy polygon collision component | |
| RigidBody | anzu-legacy | Legacy rigid body component | |
| Lifecycle | anzu-legacy | Legacy spawn/despawn lifecycle marker | |
| SparseStorage | anzu-legacy | Legacy sparse ECS storage type | |
| InputEvent | anzu-legacy | Input event record before context mapping | |
| InputBinding | anzu-legacy | Binding between raw control and action | |
| InputControlMap | anzu-legacy | Map of controls to semantic actions | |
| InputActionEvent | anzu-legacy | Action-level input event after mapping | |
| InputContextStack | anzu-legacy | Stack of active input context identifiers | |
| UiBridgeState | anzu-legacy | Browser UI bridge interaction state | |
| PlatformerTuning | anzu-legacy | Tuning constants for platformer profile motion | |
| PlatformerLocomotionState | anzu-legacy | Runtime locomotion state for platformer controls | |
| InputContext | anzu-legacy | Legacy gameplay or overlay context mode | |
| ControlCommand | anzu-legacy | Engine command enum for overlay and controls | |
| HudTelemetryMode | anzu-legacy | HUD telemetry verbosity mode | |
| OverlayVisibility | anzu-legacy | Overlay visibility state record | |
| ControlPlaneAction | anzu-legacy | Action emitted by control-plane transitions | |
| ControlPlaneState | anzu-legacy | State owner for overlay, notifications, and ROM context stack | |
| BatchVertex | anzu-legacy | Renderer batch vertex layout | |
| DrawBatch | anzu-legacy | Batch of geometry grouped for one render material path | |
| MaterialBlendMode | anzu-legacy | Blend mode enum for material pipeline selection | |
| MaterialDefinition | anzu-legacy | Material shading and blend configuration record | |
| MaterialRegistry | anzu-legacy | Legacy in-renderer material definition cache | |
| RomRenderData | anzu-legacy | Render package metadata from ROM | |
| RenderRomPackage | anzu-legacy | ROM trait extension exposing render data | |
| RenderError | anzu-legacy | Renderer initialization/runtime error set | |
| RenderCore | anzu-legacy | Core render pipeline owner in legacy runtime | |
| GamepadPollStats | anzu-legacy | Diagnostics counters for gamepad polling | |
| InputDiagnosticsSnapshot | anzu-legacy | Point-in-time diagnostics capture for input state | |
| RomPackage | anzu-legacy | ROM trait for bootstrap and simulation creation | |
| InputContextRequest | anzu-legacy | Simulation request to Push/Pop/Replace active context | |
| SimulationTransform2D | anzu-legacy | Legacy 2D transform projection for simulation output | |
| SimulationModel | anzu-legacy | Simulation behavior interface trait | |
| PhysicsConfig | anzu-legacy | Physics bounds and material defaults config | |
| InteractionEventKind | anzu-legacy | Typed interaction/collision event classification | |
| InteractionEvent | anzu-legacy | One interaction event with source and kind | |
| SchedulerFrameReport | anzu-legacy | Per-frame simulation metrics and event report | |
| CollisionManifold | anzu-legacy | Contact normal and penetration details | |
| Scheduler | anzu-legacy | Legacy simulation step orchestrator | |
| TriangleMan2Rom | anzu-legacy | ROM profile package for TriangleMan2 slice | |
| TriangleMan2Simulation | anzu-legacy | Simulation model for TriangleMan2 profile | |
| TriangleMan2ControlFrame | anzu-legacy | Input frame aggregate for TriangleMan2 | |
| TriangleMan2InputManager | anzu-legacy | Input mapping and state manager for TriangleMan2 | |
| TriangleManSpec | anzu-legacy | Legacy spec/config record for TriangleMan | |
| TriangleManRom | anzu-legacy | ROM profile package for TriangleMan slice | |
| TriangleManSimulation | anzu-legacy | Simulation model for TriangleMan profile | |
| TriangleManInputFrame | anzu-legacy | Input frame aggregate for TriangleMan | |
| TriangleManInputManager | anzu-legacy | Input mapping and state manager for TriangleMan | |
| RolePairPolicy | anzu-legacy | Collision policy mapping by interacting role pairs | |
| CollisionMode | anzu-legacy | Mode describing collision outcome handling | |
| CollisionAction | anzu-legacy | Result action selected from collision policy | |
| FragmentRequest | anzu-legacy | Request to split or replace object fragments | |
| Hitbox | anzu-legacy | Collision-only role without impulse resolution | Keep distinct from solid body |

## Ambiguous Terms (needs scope clarification)
| Term | Conflicting Meanings | Preferred Scoped Replacement | Status | PR |
|---|---|---|---|---|
| ROM | Runtime module package, game/content profile, or manifest rom_id | Use `rom package` for trait/package and `domain profile` for product-level concept | open | |
| world | ECS container, gameplay universe, or simulation state snapshot | Use `ECS world` for container and `world state` for mutable simulation data | open | |
| state | Generic variable state, simulation-owned state, or UI/control state | Use `simulation state` or `control-plane state` when specific | open | |
| model | Data model, simulation behavior trait, or render model asset | Use `simulation model` or `mesh/model asset` when specific | open | |
| context | Input context, execution context, or graphics API context | Use `input context` or `runtime context` as scoped terms | open | |
| pipeline | Runtime stage pipeline, render pipeline object, or compute pipeline object | Use `runtime pipeline`, `render pipeline`, or `compute pipeline` | open | |
| material | Render material definition, gameplay material property, or physics friction defaults | Use `material definition` for rendering and `physics material parameters` for simulation | open | |
| profile | Domain profile, performance profile, or user preference profile | Use `domain profile` or `performance profile` | open | |
| snapshot | Diagnostics snapshot, save-state snapshot, or extraction snapshot | Use `diagnostics snapshot`, `state snapshot`, or `render snapshot` | open | |
| adapter | Platform adapter abstraction or wgpu adapter GPU selector | Use `platform adapter` or `GPU adapter` | open | |

## Pending Terms (PR approval required)
| Proposed Term | Scope | Proposed Meaning | Proposed By | PR |
|---|---|---|---|---|

## Review checklist for new terms
1. Is the term necessary for precision?
2. Is the meaning distinct from existing approved terms?
3. Is the scope correctly labeled (`core`, `rust`, `wgpu`, or project-specific)?
4. Is the wording short and unambiguous?
5. Has at least one reviewer approved the PR?