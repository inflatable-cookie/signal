# g04.001 Contract Inventory And Boundary Baseline Tranche

Date: 2026-03-12
Scope: `docs/contracts/`, `docs/roadmaps/g04/`

## Summary

Completed Batch 1.1 of `g04.001` by inventorying Signal crate maturity and
freezing the first explicit public boundary around the runtime/export/report
surface instead of leaving that contract implied across multiple crates.

## What changed

- added `docs/contracts/003-crate-maturity-and-public-runtime-boundary-baseline.md`
  to classify the current workspace crates as `public`,
  `consumer-facing but unstable`, or `internal`
- froze the first narrow `g04` boundary around the typed
  `signal-runtime` runtime/export/report family, the versioned supervisor
  export envelope already defined by contract `002`, and the
  `signal-supervisor-tools` consumer-facing export proof path
- recorded the explicit deferred surfaces that remain outside that first freeze,
  including host convenience APIs, backend adapters, CLI presentation details,
  and internal scheduling/orchestration policy not yet promoted into typed
  runtime-owned receipts
- updated the `g04.001` roadmap and generation/readme index docs so the queue
  now points at Batch 1.2 instead of treating the generation as only opened

## Why this tranche

`g04` cannot deepen multicore scheduling, deferred orchestration, or backend
portability cleanly while the crate/runtime boundary is still inferred from the
last implementation batch. This tranche makes the first stable seam explicit
without prematurely freezing every consumer-facing crate.

## Validation

- `git diff --check`
- `effigy health --repo .`

## Next

Continue `g04.001` with Batch 1.2 and define the maturity-tier promises plus
schema/versioning rules for the newly frozen runtime/export boundary.
