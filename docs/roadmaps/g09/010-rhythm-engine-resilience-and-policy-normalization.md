# 010 - Rhythm Engine Resilience And Policy Normalization

Status: complete
Owner: core-product
Created: 2026-04-08
Depends on: g09.001
Vision tags: `RHYTHM`, `ANALYSIS`, `RESILIENCE`
Contract refs: `047`, `078`
Strict lane refs: `docs/specs/001-g09-lane-first-strict-adoption.md`

## Problem

`signal-analysis-rhythm` still carries panic-on-worker-failure behavior and
duplicated continuity-policy arms that make the most complex analysis crate hard
to trust and hard to evolve.

## Goals

- [ ] contain worker failures as typed degraded analysis outcomes
- [ ] replace duplicated tempo and meter branch families with a more explicit
      policy model
- [ ] preserve deterministic continuity behavior while making it easier to tune

## Non-Goals

- [ ] no brand-new rhythm feature breadth
- [ ] no product-local groove or editing UX

## Execution Plan

### Batch 10.1 - Failure Containment

- [x] freeze the first worker-failure containment seam as the next ready batch
- [x] remove `join().unwrap()` style worker joins from production rhythm paths
- [x] define typed degraded and failed analysis receipts for worker-loss cases
- [x] add targeted failure-injection tests around worker panics and partial
      feature availability

### Batch 10.2 - Policy Normalization

- [x] inventory duplicated tempo and meter recommendation arms
- [x] replace the first near-copy tempo-state branch family with a staged
      policy helper
- [x] replace the first repeated meter continuity plan shell with a staged
      builder
- [x] replace the spread-out meter continuity trigger, reason, and cause
      derivation with an explicit shared rule surface
- [x] replace the duplicated meter continuity stage-versus-plan assembly shell
      with one explicit context assembler
- [ ] replace the remaining near-copy branch families with policy tables,
      scorecards, or staged evaluators
- [ ] keep recommendation provenance explicit so tuning stays inspectable

### Batch 10.3 - Corpus Proof And Demo Handoff

- [x] rerun focused rhythm regression comparisons across the normalized tempo
      and meter policy surfaces
- [x] document preserved posture and explicit deferred demo work in the strict
      evidence layer
- [x] hand the interactive rhythm continuity demo scenario forward to the demo
      substrate milestones

## Acceptance Criteria

- [x] worker failure no longer crashes the rhythm path
- [x] tempo and meter continuity policy is materially less duplicated
- [x] recommendation changes are corpus-backed and inspectable

## Risks And Mitigations

- Risk: policy normalization changes recommendation behavior unpredictably.
- Mitigation: use corpus comparisons and explicit change logs for each policy
  tranche.

- Risk: degraded analysis answers become vague or non-actionable.
- Mitigation: define typed degraded outcomes before removing panic paths.

## Evidence Requirements

- [ ] log each rhythm tranche
- [ ] run `cargo check -p signal-analysis-rhythm`
- [ ] run focused rhythm corpus or regression lanes
- [ ] run `effigy health`

## Next Task

Continue the active strict `g09` lane from
`docs/roadmaps/g09/batch-cards/019-g09-011-demo-program-shape.md`.
