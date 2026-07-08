# Stakeholder Brief - Engine Architecture and Delivery Risks (2026-07-08)

## Executive Summary
The current engine is functional, but several core design flaws create delivery risk for scale, maintainability, and future platform expansion. The main issue is architectural: adapter layers currently own too much runtime logic. This increases change cost and makes cross-platform behavior harder to guarantee.

We have already started measurable stress testing and confirmed bottlenecks in simulation broadphase and render extraction under dense synthetic workloads. The backlog has now been updated to prioritize remediation before further feature expansion, and includes a new milestone for in-game multi-ROM menu loading.

## What Is Working
- Deterministic simulation pipeline exists and is testable.
- Browser runtime is operational with control-plane overlays.
- Material and render pipeline foundations are in place.
- Stress harnesses now provide reproducible percentile metrics for key hot paths.

## Confirmed Design Flaws
1. Adapter boundary violation:
- Wasm runtime adapter currently owns host integration plus runtime/extraction policy in one module.
- This is the highest architectural risk.

2. CPU-heavy extraction path:
- Per-frame CPU transform/grouping work scales poorly with scene size.

3. Dynamic upload pressure risk:
- Current full-frame dynamic buffer write pattern is acceptable for small slices but risky at larger geometry scale.

4. Indirection seams need cleanup:
- Some trait-object and boxed seams are likely valid extension points, but others may be accidental complexity caused by mixed ownership.

## Why Stakeholders Should Care
- Schedule risk: feature velocity drops as coupling rises.
- Cost risk: late architecture repairs are more expensive than early boundary correction.
- Product risk: TriangleMan3D and large terrain goals will be constrained unless extraction/upload paths are hardened now.
- Platform risk: non-browser wasm/native host parity remains fragile while adapters own core behavior.

## Measured Signals From New Stress Harnesses
- Dense simulation scenarios show broadphase dominance and high candidate-pair growth.
- Dense render extraction scenarios show materially higher p50/p95/p99 than sparse scenarios.
- We now have structured CSV outputs for trend tracking and regression gating.

## Backlog Priority Changes Made
Backlog updates are in [BACKLOG.md](../BACKLOG.md):

- Added new top-priority remediation milestone:
  - Milestone 5.75: Architecture Remediation—Adapter Boundary and Hot-Path Hardening.
  - Priority: P0.
  - Focus: move policy to core, harden extraction/upload budgets, and unify benchmark reporting.

- Added new stakeholder feature milestone:
  - Milestone 5.10a: In-Game Multi-ROM Menu and Runtime ROM Switching.
  - Priority: P0.
  - Focus: ROM catalog, in-game menu UX, safe activation/rollback, and multi-ROM session acceptance.

## Proposed Delivery Sequence
1. Execute Milestone 5.75 remediation first.
2. Parallelize menu UX groundwork where it does not bypass safety/integrity gates.
3. Deliver Milestone 5.10a with strict activation safety and rollback behavior.
4. Resume broader feature expansion once architecture and performance gates are stable.

## Decision Request
Approve the current priority order:
- First: architecture flaw remediation and hot-path hardening.
- Second: in-game multi-ROM switching capability on top of corrected boundaries.
