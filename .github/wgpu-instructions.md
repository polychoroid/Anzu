# wgpu Workspace Instructions

Purpose
- Document workspace-specific guidance for wgpu and GPU workloads.
- Capture renderer architecture, resource lifecycle, and performance practices.

Scope
- Real-time frame pacing, minimal allocations, command encoding, and buffer management.
- Project-specific wgpu idioms that should not be applied blindly to unrelated projects.

Guiding principles
- Prefer clear ownership of GPU resources, explicit lifecycle, and deterministic cleanup.
- Avoid runtime code generation; use design guidance and explicit architecture recommendations.
- Keep the renderer and simulation layers decoupled where possible.
