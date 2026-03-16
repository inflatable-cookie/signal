# 2026-03-16 19:27:45 UTC - g06.020 Soak And Promotion Policy Contract Opening Tranche

## Summary

Opened `g06.020` by freezing the bounded long-session soak, promotion-gate, and
Loophole-readiness policy. This batch turns the final `g06` closeout question
into an explicit contract instead of leaving it as a loose end after the
integrated acceptance lane closed.

## Work completed

- added the new closeout policy contract:
  - `docs/contracts/031-long-session-soak-promotion-gate-and-loophole-readiness-contract.md`
- recorded the Batch 20.1 outcome in:
  - `docs/roadmaps/g06/020-long-session-soak-promotion-gate-and-loophole-readiness-closeout.md`
- updated the shared indexes and reference trail:
  - `docs/contracts/README.md`
  - `docs/roadmaps/g06/README.md`
  - `docs/roadmaps/README.md`
  - `docs/roadmaps/generation-index.md`
  - `docs/architecture/graph-runtime-feature-reference.md`

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Deferred scope

- the bounded soak lane itself is still not implemented
- the combined `g06` closeout descriptor and Effigy promotion gate still belong
  to Batch 20.2
- the final Loophole-facing readiness review remains Batch 20.3 work

## Next Task

Continue `g06.020` with Batch 20.2 by implementing the combined `g06`
closeout descriptor, bounded soak lane, and repo-owned gate task on top of the
frozen closeout policy.
