# 2026-04-10 - g09.014 Release Gate Baseline

Status: complete
Owner: core-product
Roadmap: `docs/roadmaps/g09/014-production-readiness-grade-and-generation-release-gate.md`
Card: `docs/specs/batch-cards/036-g09-014-release-gate-baseline.md`

## Summary

Defined the first repo-owned production-readiness gate baseline for reopened
`g09` and recorded which evidence is currently required, advisory, and deferred.

## Outcome

- froze the reopened `g09` gate baseline in contract `080`
- defined the current required evidence as:
  - `effigy health`
  - `effigy qa:docs`
  - `effigy qa:northstar`
  - `effigy demo:coverage-matrix`
  - focused runnable proof families for any crate or family promoted to
    `production-ready for role`
- classified broader `effigy acceptance:*`, live demo launch tasks, and machine-
  readable boundary descriptors as advisory depth
- classified `effigy validate` and `cargo test --workspace --no-run` as
  explicitly deferred because the current workspace validate wall is broken by
  stale split test-module wiring and host-test import drift in
  `signal-host-local` and `signal-host-server`
- promoted the next batch to repair that workspace validate surface directly

## Validation Run

- `effigy tasks`
- `effigy test --plan`
- `effigy health`
- `effigy validate` (fails at the current broken workspace validate wall)
- `effigy qa:docs`
- `effigy qa:northstar`

## Next Task

Continue the reopened strict `g09` lane from
`docs/specs/batch-cards/037-g09-014-workspace-validate-surface-repair.md`.
