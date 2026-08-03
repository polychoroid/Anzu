# Rust Workspace Instructions

Purpose
- This project treats Rust itself as a primary learning target, not just an implementation language.
- Agents must use canonical Rust forms from the Rust Book and explain them clearly.
- Recommendations must balance performance with ease of reading and long-term maintainability.
- Terminology baseline: treat Anzu as a simulation and visualization system; treat games as one domain profile built on top of the core.
- For response structure and cost controls, apply `.github/instructions/universal-structured-response.instructions.md`.
- For term consistency, use Rust-scoped entries in `.github/response-dictionary.md`.

Primary expectation
- Treat Rust guidance as source-grounded engineering guidance, not style folklore.
- Prefer canonical forms shown in the Rust Book when they fit the problem.
- If proposing a non-canonical optimization, explain why it is needed and what tradeoff it introduces.

Authoritative sources to prefer
- Rust Book (primary): https://doc.rust-lang.org/stable/book/
- The Rust Reference (language details): https://doc.rust-lang.org/stable/reference/
- Rust std docs (APIs and semantics): https://doc.rust-lang.org/stable/std/
- Rust API Guidelines (public API design): https://rust-lang.github.io/api-guidelines/
- Rustonomicon (advanced unsafe-only context): https://doc.rust-lang.org/nomicon/

Source hierarchy for answers
- For canonical style and learning guidance, prefer the Rust Book.
- For precise language rules, confirm with the Rust Reference.
- For type/function behavior and complexity implications, confirm with std docs.
- For public crate API shape, use API Guidelines.
- Use the Rustonomicon only when unsafe details are required.

Canonical Rust patterns agents should default to
- Ownership and borrowing first:
  - Prefer borrowing over cloning.
  - Prefer references and slices in APIs when ownership transfer is not required.
- Type clarity:
  - Prefer concrete types in local implementation code.
  - Use generics for compile-time polymorphism where practical.
  - Use trait objects (`dyn Trait`) only when runtime polymorphism is truly required.
- Error handling:
  - Use `Result<T, E>` and `?` for recoverable errors.
  - Reserve `panic!` for unrecoverable invariant violations, tests, and impossible states.
  - Avoid `unwrap`/`expect` in production paths unless rationale is explicit.
- Data modeling:
  - Prefer `enum` for closed sets and state machines.
  - Prefer newtypes for domain meaning and type safety.
- Control flow:
  - Prefer `match` for exhaustive branching.
  - Prefer `if let` and `while let` for single-pattern ergonomic cases.
- Iteration:
  - Prefer iterator adapters and clear loops over index-heavy mutation unless indexing is the clearest form.
  - Use `iter`, `iter_mut`, and `into_iter` intentionally; explain ownership impact.

Performance-first defaults (without harming readability)
- Avoid unnecessary allocation:
  - Prefer `&str` for read-only text parameters and `String` for owned storage.
  - Pre-allocate with `Vec::with_capacity` when approximate size is known.
  - Reuse buffers in hot paths.
- Avoid unnecessary dynamic dispatch and heap indirection:
  - Prefer enums or generics over `Box<dyn Trait>` in hot paths when feasible.
  - Use `Box` for recursive types, large move-avoidance, or ownership/lifetime boundaries where justified.
- Avoid unnecessary cloning and copying:
  - Audit `clone()` in loops and per-frame code.
  - Pass references where possible.
- Use data locality consciously:
  - Prefer contiguous structures (`Vec<T>`, slices) for hot iteration.
  - Keep hot-path structs compact and avoid pointer-chasing designs unless needed.
- Keep critical paths measurable:
  - Require benchmark or timing evidence for non-obvious optimization claims.
  - Distinguish algorithmic improvements from micro-optimizations.

Readability rules agents must apply
- Prefer simple, explicit code over clever abstractions.
- Keep functions focused:
  - One primary responsibility per function.
  - Extract helper functions when branching depth or local state becomes hard to scan.
- Prefer descriptive names over abbreviations.
- Keep lifetime annotations minimal and local:
  - Add explicit lifetimes when required by signatures.
  - Do not over-annotate where elision already communicates intent.
- Keep trait bounds readable:
  - Use `where` clauses when bounds are long.
- Keep modules coherent:
  - Group by domain responsibility, not by language feature.

Guidance on `dyn`, `Box`, and abstraction boundaries
- `dyn Trait` is appropriate when:
  - Runtime-selected behavior is required.
  - Plugin-style or boundary-oriented extensibility is required.
- Generics/enums are preferred when:
  - Variants are known and finite.
  - Dispatch can be resolved at compile time.
  - Code is in hot loops where dispatch overhead and locality matter.
- `Box<T>` is appropriate when:
  - Recursive size indirection is required.
  - Ownership transfer across boundaries needs stable indirection.
  - Large values should avoid repeated stack moves.
- `Box` is a smell when:
  - Used only to silence borrow-checker confusion without design rationale.
  - Stacked with trait objects in hot paths without profiling evidence.

Collections and API surface guidance
- Prefer slices (`&[T]`, `&mut [T]`) in function parameters over `&Vec<T>`.
- Prefer iterator-accepting APIs (`impl IntoIterator`) where it improves call-site ergonomics without obscuring ownership.
- Use maps/sets intentionally:
  - `HashMap` for average O(1) lookup.
  - `BTreeMap` for ordering and range queries.
- Document mutation and ownership expectations in function names and docs.

Concurrency and async guidance
- Use message-passing and ownership transfer patterns where they simplify reasoning.
- Minimize shared mutable state; prefer scoped mutation and clear synchronization boundaries.
- For async code:
  - Avoid unnecessary boxing of futures unless object safety or trait erasure is required.
  - Keep async boundaries coarse enough to remain understandable.

Unsafe and low-level rules
- Default to safe Rust.
- Only introduce `unsafe` when a measurable need exists or API boundary requires it.
- Every unsafe block must include a concise safety invariant explanation.
- Keep unsafe blocks small and isolated.

Documentation and explanation requirements
- When proposing Rust changes, explain:
  - Why the pattern is canonical (book-grounded).
  - Why it is performant enough (or more performant) for this path.
  - Why it improves readability and maintenance.
- If choosing a non-canonical pattern, explain:
  - Why canonical form is insufficient here.
  - What evidence supports the deviation.

Required factual habits
- Distinguish known facts from inference.
- Do not invent language guarantees or complexity claims.
- If unsure about semantics, check the Rust Reference or std docs before asserting.
- Prefer small, verifiable claims tied to concrete code paths.

What not to do
- Do not use advanced features only for style points.
- Do not optimize speculative bottlenecks without measurement.
- Do not hide ownership semantics behind opaque helper layers.
- Do not overuse macros where normal functions/types are clearer.
- Do not trade readability for micro-optimizations in non-hot paths.

Project-specific expectation
- Leave code easier to read than you found it.
- Keep Rust changes teachable: future readers should understand ownership, lifetimes, and data flow directly from the code.
- Favor architecture that scales from current 2D needs toward larger 3D workloads without abandoning canonical Rust design.
- Keep type and module naming aligned with domain-neutral glossary terms in `documents/architecture.md` (`Core Glossary`), especially for core runtime and simulation boundaries.
