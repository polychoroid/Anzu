---
name: implement-backlog-item
description: "Implement one backlog item at a time from a detailed plan to optimize token usage and keep code changes manageable for a human engineer. Use when executing backlog plans with strict item-by-item control and engineer-guided error resolution."
argument-hint: "Implement backlog item <id> from <plan file>"
user-invocable: true
---

# Implement Backlog Item

## Purpose
Execute implementation from a backlog plan with strict one-item-at-a-time control, minimal change surface, and explicit engineer checkpoints.

## What This Skill Produces
- A completed implementation for exactly one backlog item.
- A concise implementation note that includes:
  - Item ID and acceptance criteria addressed
  - Files changed
  - Tests/checks run
  - Remaining risks
  - Suggested next backlog item
- Error analysis reports when failures occur, with engineer decision prompts.

## When to Use
- A detailed backlog plan already exists.
- The team wants to control token usage by narrowing scope.
- The engineer wants small, reviewable change sets.
- Error resolution should be human-directed, not auto-looped by the agent.

## Input Contract
Provide at least:
1. Backlog source file (default: `BACKLOG.md`).
2. Target item identifier (for example M2-I3).
3. Acceptance criteria for the target item.
4. Constraints (runtime/platform/performance/testing boundaries).

If any required input is missing, ask narrowing questions before implementation.

## Procedure

### 1. Select Exactly One Item
1. Confirm the target item ID.
2. Restate acceptance criteria in deterministic language.
3. Refuse multi-item execution in a single pass.

Decision points:
- If the request includes multiple items, ask the engineer to choose one.
- If the target item is ambiguous, pause and request item disambiguation.

### 2. Define the Change Boundary
1. List the minimum required files to change for this item.
2. List expected structs/functions to create or modify.
3. Declare non-target files and behaviors as out of scope.

Decision points:
- If boundary cannot be made small, split item into sub-item steps and ask for approval.

### 3. Implement in a Single Focused Pass
1. Apply only changes required by this item.
2. Keep code output compact and directly tied to acceptance criteria.
3. Do not start follow-on backlog items in the same pass.

### 4. Validate Once, Then Stop on Errors
1. Run the planned checks for this item.
2. If checks pass, proceed to completion report.
3. If checks fail, do not enter iterative auto-fix loops.

Error handling policy (mandatory):
- Perform detailed failure analysis:
  - Exact failing command/check
  - Error category (compile, type, test, runtime, integration)
  - Likely root cause(s)
  - Affected files/symbols
- Prompt the engineer for freeform guidance before further edits.
- Wait for explicit engineer direction before attempting a fix.

### 5. Completion Gate
Mark the item complete only if all checks pass:
- Only one backlog item was implemented.
- Acceptance criteria for the target item are satisfied.
- Change set is bounded to the declared file/function scope.
- Every new function has complete unit test coverage for expected behavior, edge cases, and error paths.
- Every new struct has complete unit test coverage for construction, core invariants, and state transitions/behavior.
- Validation evidence is recorded.
- No unresolved errors or warnings remain.

If any check fails, return an error analysis report and request engineer guidance.

## Output Format (Use This Shape)
1. Target item confirmation
2. Scope boundary (in-scope / out-of-scope)
3. Implementation summary
4. Validation results
5. Completion gate result
6. If failed: detailed error analysis + decision prompt for engineer

## Quality Bar
- One-item strictness is never bypassed.
- Prefer small, reviewable diffs over broad refactors.
- No autonomous error-fix looping.
- Escalate uncertainty early with explicit engineer prompts.
