# g04.001 Maturity Tier And Schema Versioning Policy Tranche

Date: 2026-03-12
Scope: `docs/contracts/`, `docs/roadmaps/g04/`

## Summary

Completed Batch 1.2 of `g04.001` by turning the crate/runtime boundary
inventory into an explicit policy surface: each maturity tier now has a stated
promise level, and the runtime/export/report family now has clear change-control
rules for additive growth, breaking changes, and migration-note triggers.

## What changed

- extended `docs/contracts/003-crate-maturity-and-public-runtime-boundary-baseline.md`
  with explicit promises for `public`, `consumer-facing but unstable`, and
  `internal` crates
- tied the typed `signal-runtime` report/receipt surface to the existing
  `signal.supervisor.export` schema discipline rather than letting Rust DTOs
  and JSON export evolve under separate undocumented rules
- recorded the concrete changes that now require migration notes, including
  crate reclassification, breaking runtime/export/report DTO changes, runtime
  ownership drift into host-local bookkeeping, and export default/schema
  behavior changes
- updated the `g04.001` roadmap and generation/readme queue so the next step is
  the consumer-facing proof tranche rather than more policy-only work

## Why this tranche

Batch 1.1 named the first frozen boundary, but it still left consumers guessing
which changes would be considered routine iteration versus contract breakage.
This tranche closes that gap before `g04` moves on to deeper scheduling or
portability work.

## Validation

- `git diff --check`
- `effigy health`

## Next

Continue `g04.001` with Batch 1.3 and add a focused proof or fixture showing
the frozen boundary is consumable without private crate internals.
