# wgpu Workspace Instructions

Purpose
- This project uses `wgpu` as a serious learning target, not only as a dependency.
- Agents working on the engine must explain `wgpu` usage factually, concretely, and in a way that helps the user learn the real architecture.
- When proposing code, design changes, or explanations, agents should act as if the user wants to understand both the local engine code and the upstream `wgpu` model.

Primary expectation
- Treat `wgpu` knowledge as source-grounded engineering knowledge.
- Do not hand-wave with generic graphics advice when the upstream `wgpu` repository, examples, or public docs give a more precise answer.
- Prefer explicit explanations of why a `wgpu` pattern exists, what stage it belongs to, and what tradeoffs it creates.

Authoritative sources to prefer
- `wgpu` repository: https://github.com/gfx-rs/wgpu
- `wgpu` docs: https://docs.rs/wgpu/
- trunk docs: https://wgpu.rs/doc/wgpu/
- official examples: https://github.com/gfx-rs/wgpu/tree/trunk/examples#readme
- upstream README learning links:
	- Learn Wgpu: https://sotrh.github.io/learn-wgpu/
	- WebGPU Fundamentals: https://webgpufundamentals.org/
- WebGPU spec: https://www.w3.org/TR/webgpu/
- WGSL spec: https://gpuweb.github.io/gpuweb/wgsl/

Repo-level upstream facts agents should know
- `wgpu` is a cross-platform, safe, pure-Rust graphics API.
- It runs on Vulkan, Metal, DX12, OpenGL, WebGL2, and WebGPU depending on platform and features.
- The API is based on WebGPU, but the Rust implementation has native and browser-specific behavior differences.
- The workspace is not a single crate; it includes distinct layers and support crates, including:
	- `wgpu`
	- `wgpu-core`
	- `wgpu-hal`
	- `wgpu-types`
	- `wgpu-macros`
	- `wgpu-naga-bridge`
	- `naga`
	- examples, tests, benches, and tooling crates

Upstream architecture model agents should internalize
- `wgpu` crate:
	- Safe public API used by engine/application code.
	- Primary layer the Anzu engine should target.
- `wgpu-core`:
	- Validation, tracking, and core implementation behavior.
	- Useful when explaining why some public API constraints exist.
- `wgpu-hal`:
	- Lower-level backend abstraction over Vulkan/Metal/DX12/GLES and related targets.
	- Relevant when discussing backend limitations, portability, or surface/device behavior.
- `naga`:
	- Shader translation and WGSL-related behavior for native and WebGL translation paths.
	- Important when explaining shader portability and WGSL feature support.
- `wgpu-types`:
	- Shared public types and configuration surface.

Agents must not blur these layers.
- Do not describe `wgpu-hal` implementation details as if they are stable public `wgpu` API guarantees.
- Do not assume browser WebGPU behavior and native behavior are identical.
- Do not assume WGSL support is identical across native, browser WebGPU, and browser WebGL fallback paths.

How to explain `wgpu` in this project
- Explain work in frame-stage order whenever possible:
	1. Surface/target acquisition
	2. Resource/state preparation
	3. Command encoding
	4. Queue submission
	5. Present
- When discussing simulation/render architecture, explicitly separate:
	- simulation/state mutation
	- render extraction
	- GPU encoding/submission
- When discussing performance, distinguish:
	- CPU extraction cost
	- upload bandwidth cost
	- draw submission cost
	- GPU shading/raster cost
	- backend/surface reconfiguration cost

How to use examples
- Use official `wgpu` examples as the primary explanation surface when the user asks how something should be done.
- Name the example and explain why it is relevant.
- Do not just say “similar to official examples”; specify the example and the concept:
	- `hello_triangle`: minimal render workflow
	- `uniform_values`: uniforms, app state, and simple interaction
	- `cube`: vertex/index buffers, textures, transforms
	- `bunnymark`: many draw calls and bind-group variation
	- `skybox`: 3D scene patterns, depth textures, model loading, interaction
	- `shadow`: more complex scene/light/shadow architecture
	- `render_to_texture`: off-screen rendering
	- `multiple-render-targets`: MRT usage
	- `hello_compute`, `repeated_compute`, `hello_workgroups`, `hello_synchronization`, `storage_texture`: compute patterns
	- `boids`: combined compute + render workflow
- For learning-oriented answers, compare the local engine pattern against the nearest official example and describe the gap.

How to answer example questions
- If the user asks for an explanation of a `wgpu` example or a local design inspired by one:
	- identify the upstream example
	- summarize what it demonstrates
	- describe the important resources involved
	- explain the command flow
	- call out what Anzu currently does similarly or differently
- Keep the explanation factual, not mystical or overly abstract.

Required factual habits for agents
- Prefer “the upstream repo/examples/docs show X” over unsupported opinion.
- When uncertain, say what is known versus what is inferred.
- Distinguish public API guidance from backend implementation detail.
- Distinguish native-only example patterns from all-platform-safe patterns.
- Distinguish WebGPU path from WebGL fallback path.
- Do not invent support guarantees for features such as ray queries, mesh shaders, storage textures, or advanced pipeline features.

Platform understanding required for this repo
- Browser WebGPU and WebGL fallback are both relevant here.
- Browser behavior can differ from native:
	- WGSL support may differ depending on browser implementation.
	- WebGL fallback relies on translation paths and different capability ceilings.
	- Surface/present behavior and shader support must not be described as identical across backends.
- Non-browser wasm runtimes may matter in this project.
- Native host expansion is part of the roadmap, so explanations should avoid browser-only assumptions unless clearly labeled.

Performance guidance agents should apply
- Prefer GPU-resident static data for large/static geometry.
- Avoid assuming full-frame CPU rebuild + upload is acceptable once geometry scales.
- Keep upload budgets explicit and measurable.
- Treat depth/stencil, culling, LOD, and visibility management as first-order concerns for 3D growth.
- Explain why an approach is appropriate for a tiny 2D slice versus why it fails for large 3D terrain.
- Use measured evidence from this repo when available before speculating.

Engine-specific guidance for Anzu
- Keep simulation logic and render logic separate.
- Adapter layers must stay thin.
- Do not let wasm/browser glue absorb core runtime or extraction policy.
- When proposing renderer changes, call out whether the change belongs in:
	- browser adapter
	- wasm-runtime adapter
	- runtime core
	- render extraction core
	- render backend core
- Preserve deterministic simulation behavior while improving render-side structure.

Learning-oriented answer style
- The user is learning. Optimize for understanding.
- Prefer clear factual explanations of resource ownership, command flow, shader path, and performance consequences.
- If recommending a pattern, explain:
	- what problem it solves
	- where it fits in the `wgpu` model
	- what tradeoffs it introduces
	- which upstream example or documentation supports it
- Avoid shallow “guru” tone. Deliver real substance instead.

What not to do
- Do not present cargo-cult `wgpu` patterns without context.
- Do not summarize all GPU concepts at once when the local question is narrower.
- Do not describe all backends as equivalent.
- Do not treat example code as production guidance without noting what it simplifies.
- Do not make claims about `wgpu` internals that are not grounded in the public repo/docs.

Project-specific expectation
- When modifying engine code that touches `wgpu`, agents should leave behind explanations, documentation, or benchmark hooks that improve the user’s understanding of the system.
- The goal is not only a working engine. The goal is a working engine that teaches its architecture clearly.
