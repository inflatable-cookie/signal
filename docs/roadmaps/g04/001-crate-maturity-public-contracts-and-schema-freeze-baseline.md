# 001 - Crate Maturity, Public Contracts, And Schema-Freeze Baseline

Status: complete
Owner: core-product
Created: 2026-03-12
Depends on: g03.008
Vision tags: `RUNTIME`, `CONTRACTS`, `PACKAGING`

## Problem

Signal now has a broad reusable surface, but its crate boundaries, public API
expectations, and supervisor/export schema guarantees still mostly reflect the
order that implementation batches landed. That is good enough for one moving
consumer, but not for an independent multi-consumer Signal thread.

Without an explicit maturity and contract-freeze milestone:

- consumer repos will keep guessing which crates are stable versus internal
- schema/versioning policy will drift between exports, reports, and docs
- later multicore, portability, and backend work will keep reopening package
  and API boundaries
- release packaging will remain downstream negotiation instead of Signal-owned
  policy

## Goals

- [x] classify Signal crates by public maturity and intended consumer role
- [x] freeze the first explicit public contract boundary for runtime/export
  surfaces
- [x] define schema/versioning expectations for the supervisor/export/report
  family
- [x] create a stable starting point for later scheduling and portability work

## Non-Goals

- [ ] no large implementation reshuffle for its own sake
- [ ] no consumer-specific adapter work in this milestone
- [ ] no crates.io publication pipeline yet

## Execution Plan

### Batch 1.1 - Contract Inventory

- [x] inventory the current crates, contracts, and export/report surfaces that
  downstream consumers actually depend on
- [x] classify each as `public`, `consumer-facing but unstable`, or `internal`
- [x] identify the minimum boundary that must stabilize first

### Batch 1.2 - Maturity And Versioning Policy

- [x] document crate maturity tiers and what each tier promises
- [x] define schema/versioning rules for supervisor/export/report boundaries
- [x] record what can change freely versus what now requires migration notes

### Batch 1.3 - Consumer-Facing Proof

- [x] add a focused proof or fixture showing the frozen boundary is consumable
  without reading private crate internals
- [x] align README/contract docs with the chosen boundary
- [x] record residual unstable surfaces that stay explicitly deferred

## Progress Notes

- 2026-03-12: completed Batch 1.1 by inventorying the current workspace crates
  into public, consumer-facing-but-unstable, and internal tiers, then freezing
  the first explicit `g04` baseline contract around the `signal-runtime`
  runtime/export/report family plus the existing versioned supervisor export
  envelope and `signal-supervisor-tools` proof path.
- 2026-03-12: completed Batch 1.2 by extending that baseline contract with
  tiered promises for `public`, `consumer-facing but unstable`, and `internal`
  crates, tying typed `signal-runtime` report/receipt DTOs to the existing
  `signal.supervisor.export` schema discipline, and explicitly naming the
  migration-note triggers for future boundary changes.
- 2026-03-12: completed Batch 1.3 and closed `g04.001` by adding an external
  `signal-runtime` integration test that exercises the frozen report/receipt
  boundary through public re-exports only, then aligning the contract docs with
  that proof and explicitly keeping host convenience APIs, backend adapters,
  and CLI presentation details outside the first stability promise.

## Acceptance Criteria

- [x] Signal has one explicit public crate/contract boundary instead of implied
  maturity
- [x] export/report schema stability expectations are written down
- [x] later `g04` work can deepen execution and portability without reopening
  the same boundary debate

## Risks and Mitigations

- Risk: the milestone turns into a repo-wide cleanup spree.
- Mitigation: stabilize only the boundary required for the next `g04` work.
- Risk: “public” is declared too broadly and freezes half-finished surfaces.
- Mitigation: use a narrow maturity classification and defer aggressively.

## Evidence Requirements

- [x] log each meaningful boundary or policy tranche
- [x] run at least one focused proof or fixture against the frozen boundary
- [x] record explicit deferred surfaces instead of leaving them implied

## Next Task

COMPLETE. `g04.001` closed on 2026-03-12 after the crate maturity baseline,
schema/versioning policy, and consumer-facing runtime/export proof were all
landed. Continue with `g04.002` now that the first public Signal contract is
explicit.
