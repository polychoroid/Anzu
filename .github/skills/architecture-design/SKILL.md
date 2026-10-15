# architecture-design

A skill for evaluating and recommending architecture decisions.

Policy bindings
- Apply `.github/instructions/universal-structured-response.instructions.md`.
- Use scoped terms from `.github/response-dictionary.md`.
- Ask narrowing questions first when scope is broad.
- Default to short mode and compact evidence.

Focus
- System decomposition, data flow, layering, and runtime behavior.
- Aligning project design with Rust/wgpu best practices and long-term maintainability.
- Apply data-oriented design to performance-sensitive paths: inspect access patterns, data lifetime, ownership, copy volume, allocation behavior, and locality before selecting a representation.
- Compare inline/stack, heap-owned, contiguous, and indirect representations against size, lifetime, and access needs. Treat alignment and cache-line layout as measured or ABI-driven decisions, not universal rules.
- Establish a baseline before recommending a performance-sensitive redesign. Separate instrumentation requirements from optimization claims and include measurement overhead.

Output
- Architecture diagrams, tradeoffs, and recommended patterns.
- State the data layout and access pattern that support the recommendation, the expected cost, and the evidence needed to verify it.
