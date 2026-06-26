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
	- `State::update(...)` updates simulation state.
	- `State::render(...)` encodes and submits frame commands.

- `asset_manifest.rs`
	- `load_from_url(...)` fetches and validates manifest JSON.
	- `ManifestLoadError` provides typed error categories.
	- `AssetRegistry` exposes summary helpers for loaded data.

## Contracts

- Frame lifecycle contract: `update(delta_time)` occurs before `render()` on redraw.
- Recoverable startup/runtime failures surface as `Result` errors and console diagnostics.
- Manifest contract requires unique non-empty asset `id` and non-empty `source_url`.
- Simulation state and GPU state remain isolated for predictable extension into ECS and data-driven systems.
