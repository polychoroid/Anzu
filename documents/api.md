# API Reference

## Overview
This document describes the current public runtime surfaces and module responsibilities for the browser WASM engine crate.

## Core Modules

- `lib.rs`
	- Exports wasm start function.
	- Owns browser event loop lifecycle and redraw scheduling.

- `renderer.rs`
	- `State::new(...)` initializes GPU resources and surface configuration.
	- `State::resize(...)` handles surface reconfiguration.
	- `State::update(...)` advances the fixed-tick runtime and simulation pipeline.
	- `State::render(...)` encodes and submits frame commands.
	- Simulation hooks synchronize ROM world state before and after physics.
	- Implementation lives under `src/renderer/mod.rs`, `src/renderer/core.rs`, and `src/renderer/wasm.rs`.

- `simulation.rs`
	- `SimulationModel` exposes ROM-owned input handling, anchor synchronization, per-tick update, and reconciliation hooks.
	- `Scheduler` performs physics, including broadphase candidate selection and narrowphase collision response.
	- `SchedulerFrameReport` returns interaction and despawn results for ROM reconciliation.

- `asset_manifest.rs`
	- `load_from_url(...)` fetches and validates manifest JSON.
	- `ManifestLoadError` provides typed error categories.
	- `AssetRegistry` exposes summary helpers for loaded data.

## Contracts

- Frame lifecycle contract: fixed-tick simulation stages run before `render()` on redraw.
- Simulation contract: ROMs own intent composition and scenario rules; the scheduler owns shared physics execution and collision response.
- Recoverable startup/runtime failures surface as `Result` errors and console diagnostics.
- Manifest contract requires unique non-empty asset `id` and non-empty `source_url`.
- World state and GPU state remain isolated for predictable extension into ECS and data-driven systems.
