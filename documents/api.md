# API Reference

## Overview
This document describes the current public runtime surfaces and module responsibilities for the browser WASM engine crate.

## Core Modules

- `lib.rs`
	- Exports wasm start function.
	- Owns browser event loop lifecycle and redraw scheduling.

- `renderer/mod.rs`, `renderer/core.rs`, `renderer/wasm.rs`
	- `State::new(...)` initializes GPU resources and surface configuration.
	- `State::resize(...)` handles surface reconfiguration.
	- `State::update(...)` advances the fixed-tick runtime and simulation pipeline.
	- `State::render(...)` encodes and submits frame commands.
	- Draw extraction groups geometry into material-aware batches carrying `(material_id, vertex_offset, vertex_count)`.
	- Render core resolves `MaterialDefinition` and selects blend-aware pipeline per draw batch.
	- Per-material uniforms are updated through dynamic uniform buffer offsets.
	- Browser-side gamepad polling and semantic control normalization live in `src/renderer/wasm.rs`.
	- Runtime input routing synchronizes active ROM context with `SimulationModel` and applies simulation context transition requests into `ControlPlaneState`.
	- Overlay-visible state forces engine overlay input context and blocks gameplay event forwarding.
	- Simulation hooks synchronize ROM world state before and after physics.
	- Implementation lives under `src/renderer/mod.rs`, `src/renderer/core.rs`, and `src/renderer/wasm.rs`.

- `simulation.rs`
	- `SimulationModel` exposes ROM-owned input handling, active input context synchronization (`active_input_context_id`, `set_active_input_context`), optional context transition request emission (`pop_input_context_request`), anchor synchronization, per-tick update, and reconciliation hooks.
	- `InputContextRequest` supports `Push`, `Pop`, and `Replace` context transitions for runtime application.
	- `Scheduler` performs physics, including broadphase candidate selection and narrowphase collision response.
	- `SchedulerFrameReport` returns interaction and despawn results for ROM reconciliation.
	- Sensor/hitbox rigid bodies emit collision events but skip impulse resolution.

- `asset_manifest.rs`
	- `load_from_url(...)` fetches and validates manifest JSON.
	- `ManifestLoadError` provides typed error categories.
	- `AssetRegistry` exposes summary helpers for loaded data.

- `triangle_man.rs`, `triangle_man_2.rs`, `platformer_common.rs`
	- `TriangleManRom` implements the asteroids-style ROM package.
	- `TriangleMan2Rom` implements the platformer ROM package.
	- `platformer_common.rs` hosts shared platformer mesh/material definitions and locomotion state math.

## Contracts

- Frame lifecycle contract: fixed-tick simulation stages run before `render()` on redraw.
- Input contract: renderer-side browser adapters may emit both raw controls and semantic aliases, but ROMs should bind to semantic controls by default.
- Input context contract: runtime owns authoritative context activation through `ControlPlaneState`; ROMs request transitions, runtime applies them, and overlay context preempts ROM contexts while visible.
- Simulation contract: ROMs own intent composition and scenario rules; the scheduler owns shared physics execution and collision response.
- Recoverable startup/runtime failures surface as `Result` errors and console diagnostics.
- Manifest contract requires unique non-empty asset `id` and non-empty `source_url`.
- World state and GPU state remain isolated for predictable extension into ECS and data-driven systems.
- Material contract: visual differences should flow through `MaterialDefinition` parameters and `material_id` assignment rather than geometry-format churn.
- Collision outcome contract: role-pair policy selection determines hitbox vs solid behavior and despawn/fragment replacement decisions during reconciliation.

- Asset ingestion contract:
	- `data-loader` decodes source formats (`.glb` or `.obj` + `.mtl` + textures).
	- `asset-provider` canonicalizes decoded content into runtime records and exposes stable handles.
	- Runtime modules request model data from `asset-provider` by handle, not by direct file access.
	- Physics consumes canonical physics mesh records from `asset-provider` to build `BodyState`.

- Multi-format parity contract:
	- GLB and OBJ/MTL import paths must produce equivalent canonical mesh records for equivalent geometry.
	- Canonical records must include deterministic local-space physics mesh geometry and construction-time mass property inputs.
