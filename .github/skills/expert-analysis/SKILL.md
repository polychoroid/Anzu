---
name: expert-analysis
description: Use when the user asks for expert, factual analysis or recommendations on architecture, algorithms, data flow, runtime execution models, performance, reliability, or techniques, especially for Rust, wgpu, and Vulkan topics. Enforces strict evidence mode with confidence labeling and explicit uncertainty.
---

# expert-analysis

A skill for producing high-confidence, evidence-driven technical analysis and advice.

Policy bindings
- Apply `.github/instructions/universal-structured-response.instructions.md`.
- Use scoped terms from `.github/response-dictionary.md`.

## Focus

- Architecture decomposition: boundaries, layering, ownership, interfaces, extension seams.
- Algorithm selection: complexity, determinism, memory behavior, and failure modes.
- Data flow: producer/consumer paths, state ownership, mutation boundaries, and lifecycle.
- Runtime execution models: fixed-step vs variable-step stages, scheduling order, and invariants.
- Reliability and security posture: typed errors, boundary validation, and predictable fallback behavior.
- Performance analysis: measurement-first guidance, bottleneck isolation, and profiling plans.
- Data-oriented design: data layout, alignment, ownership, copy and allocation costs, cache locality, and hot-path iteration patterns.
- Rust/WASM constraints: deterministic behavior under single-threaded browser runtime conditions.

## Evidence Policy (Strict Mode)

- Ground major claims in at least one of:
  - In-repo evidence (code/docs with concrete file references).
  - Official standards or vendor documentation.
  - Well-known engine/runtime references used as comparative context.
- Never present uncertain claims as facts.
- If evidence is incomplete or conflicting:
  - Say so directly.
  - Provide confidence level (`high`, `medium`, `low`).
  - Offer verification steps to close the gap.
- Prefer measurable recommendations over opinion-only guidance.
- Keep evidence compact by default and defer deep citation until requested.

## Data-Oriented Performance Method
- Distinguish data-oriented design (layout and access patterns) from data-driven configuration (behavior or content supplied as data).
- Establish a baseline on a representative workload before changing a hot path. Record platform/backend, workload size, warm-up, sample count, and instrumentation mode.
- Prefer low-cost counters for continuous monitoring and sampled or opt-in high-resolution timing. Report instrumentation overhead.
- Report stage timing distributions (at least p50, p95, and p99) with work counts. Separate CPU and GPU time where the platform exposes both.
- Treat stack use, heap allocation, alignment, cache locality, and copy reduction as hypotheses to test. Do not recommend large stack values, forced padding, or complex layouts without workload evidence.
- Compare before and after with the same workload. Keep raw output and the command or procedure used to produce it.

## WGPU and Vulkan Requirements

When rendering/runtime architecture is discussed, include these stage boundaries explicitly:

1. Frame acquire
2. Simulation and game-state stages (fixed and variable where applicable)
3. Render extraction from immutable or stabilized state snapshot
4. Command encoding
5. Queue submission
6. Present
7. Profiling and diagnostics (CPU/GPU timing, submission/pipeline churn where relevant)

Also check and call out:

- Synchronization assumptions and hazards.
- Command buffer lifetime/ownership boundaries.
- Resource residency/update strategy and fallback behavior.
- Impact of backend constraints on determinism and frame pacing.

## Analysis Output Format

Provide advice in this structure unless the user asks otherwise:

1. Narrowing questions (only if scope is broad)
2. Problem framing and assumptions
3. Evidence summary
4. Recommended approach
5. Alternatives and tradeoffs
6. Risks and edge cases
7. Validation plan (tests, profiling, acceptance checks)

Default mode
- Use short mode unless the user requests detail.

## Guardrails

- Do not invent citations or pretend certainty.
- Distinguish current-state observations from proposed design.
- Keep advice actionable and implementation-oriented.
- Prefer minimal, explicit contracts over broad implicit behavior.

## Output

- Factual, traceable analysis with confidence labels.
- Concrete recommendations on architecture, algorithms, data flow, and techniques.
- Practical validation steps for correctness, determinism, and performance.