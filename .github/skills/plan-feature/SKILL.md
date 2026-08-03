---
name: plan-feature
description: "Plan new feature work from backlog epics using iterative Q&A refinement and produce an implementation-ready contract with files, structs, enums, and function signatures."
argument-hint: "Plan new feature <x> from backlog epic <y>"
user-invocable: true
---

# Plan Feature

## Purpose
Create an implementation-ready backlog plan that removes design ambiguity before coding starts. The plan must be concrete enough that an engineer can implement it without asking follow-up questions about ownership, file boundaries, signatures, or test strategy.

## What This Skill Produces
- A planning document with:
  - Outcome summary
  - Scope and non-goals
  - Constraints and assumptions
  - Milestone-ordered backlog with dependencies and risks
  - File-level change map with action and rationale
  - File-oriented implementation contract where each file is a first-class section
  - Struct, enum, and trait inventory with responsibilities and fields
  - Function/method inventory with signatures, responsibilities, inputs/outputs, side effects, and error modes
  - Execution flow and ownership boundaries across files
  - Test and validation plan
  - Risk register and open questions that are driven to zero or explicitly converted into assumptions
- Default write target: BACKLOG.md unless the user requests a different file.

## When to Use
- A feature request is clear at outcome level but unclear at implementation level.
- The team wants a "plan first, build second" workflow.
- The implementation must proceed without further requirement clarification.

## Input Contract
Before drafting, gather or confirm at least:
1. Target outcome or user story.
2. Constraints (platform, runtime, performance, compatibility, deadlines).
3. Existing subsystem(s) likely affected.
4. Acceptance evidence expected by the requester.

If any critical item is missing, start with a focused refinement loop: ask 3-7 high-value questions that change structure, interfaces, or sequencing. If the user cannot answer, state assumptions explicitly and continue.

## Procedure

### 1. Frame the Planning Surface
1. Restate the request in one sentence.
2. Confirm boundaries:
   - In scope
   - Out of scope
   - Explicit non-goals
3. Capture hard constraints and assumptions.
4. Identify the likely subsystem entry points and current seam points.

Decision points:
- If scope is broad, split into milestones before decomposition.
- If constraints conflict, surface the tradeoff and request a decision or record the chosen assumption.

### 2. Build the First-Pass Plan
1. Produce milestone-ordered backlog items (M1, M2, ...).
2. For each backlog item, define:
   - Objective
   - Dependencies
   - Risks
   - Acceptance criteria
3. Map impacted code areas at file granularity.
4. Identify the likely new or changed public symbols.

### 3. Run Iterative Q&A Refinement Loops
Do not finalize the plan until the critical ambiguity is reduced.

Loop protocol:
1. Ask 3-7 high-value questions if needed.
2. Update the plan immediately after answers.
3. Recompute affected files, structs, enums, traits, and functions.
4. Repeat until no blocking ambiguity remains.

Question design rules:
- Prefer binary or bounded-choice questions where possible.
- Ask only questions that change structure, interfaces, sequencing, or validation.
- Avoid cosmetic or non-blocking questions.

Decision points:
- If an answer changes architecture, rewrite milestones and dependencies.
- If uncertainty remains high, add a time-boxed spike task with explicit exit criteria.
- Do not stop due to round count; continue refinement until blockers are zero or you have explicit assumptions.

### 4. Lock the Implementation Contract
For each planned file, define:
1. Action: create, modify, split, or deprecate.
2. Why the file changes.
3. Which structs/enums/traits are added or updated.
4. Which functions/methods are added or updated, including:
   - Signature-level details (name, params, return type)
   - Responsibility
   - Inputs/outputs
   - Side effects
   - Error modes

For cross-file behavior, define:
- Ownership boundaries
- Execution flow
- Data flow and state handoff
- Error propagation path

### 5. Add Verification Strategy
1. Map each backlog item to validation:
   - Unit tests
   - Integration tests
   - Runtime/manual checks
2. Define completion evidence per milestone.
3. Include rollback/fallback notes for high-risk changes.
4. Call out the exact command or behavior that proves completion when possible.

### 6. Perform Completion Gate
A plan is complete only when all checks pass:
- Open questions list is empty or reduced to explicit assumptions.
- Every backlog item has acceptance criteria.
- Every impacted file has explicit planned changes.
- Every new or changed struct, enum, trait, and function has defined responsibility.
- Dependency order is coherent and implementable.
- Validation coverage exists for all critical paths.

If any check fails, return to Step 3.

## Output Format (Use This Shape)
1. Outcome summary
2. Scope and non-goals
3. Constraints and assumptions
4. Milestone backlog
5. File change map
6. File-by-file implementation contract
   - For each planned file, include:
     - Action: create, modify, split, or deprecate
     - Why the file changes
     - New or updated structs, enums, and traits
     - New or updated functions and methods with signatures
     - Ownership and dependency notes
7. Execution flow and ownership boundaries
8. Validation plan
9. Risk register and open questions
10. Final completion gate report

## Required Detail Level
- Include concrete file paths and planned actions.
- Organize the plan by file, not as a flat list of unrelated changes.
- For each file, show the relevant structs, enums, traits, and functions beneath that file heading so the plan reads like an implementation map.
- Include concrete symbol names and, where practical, signatures such as `fn handle_redraw(&mut self, delta_seconds: f32) -> anyhow::Result<()>`.
- Include concrete enum variants when the feature introduces a new state machine or error taxonomy.
- Do not leave placeholders like "TBD" or "some new module" unless the plan explicitly marks them as a spike item.
- Prefer deterministic language over aspirational language.

## Quality Bar
- Specific over generic.
- Implementation-oriented over descriptive.
- Executable by an engineer without follow-up clarification.
- Traceable to the current codebase and architecture.
