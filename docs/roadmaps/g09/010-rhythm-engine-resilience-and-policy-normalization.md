# 010 - Rhythm Engine Resilience And Policy Normalization

Status: draft
Owner: core-product
Created: 2026-04-08
Depends on: g09.001
Vision tags: `RHYTHM`, `ANALYSIS`, `RESILIENCE`
Contract refs: `047`, `078`

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

- [ ] remove `join().unwrap()` style worker joins from production rhythm paths
- [ ] define typed degraded and failed analysis receipts for worker-loss cases
- [ ] add targeted failure-injection tests around worker panics and partial
      feature availability

### Batch 10.2 - Policy Normalization

- [ ] inventory duplicated tempo and meter recommendation arms
- [ ] replace near-copy branch families with policy tables, scorecards, or
      staged evaluators
- [ ] keep recommendation provenance explicit so tuning stays inspectable

### Batch 10.3 - Corpus Proof And Demo

- [ ] rerun rhythm corpus comparisons across old and new policy surfaces
- [ ] document any intentional breaking recommendation shifts
- [ ] add an interactive rhythm continuity demo scenario under the demo
      substrate

## Acceptance Criteria

- [ ] worker failure no longer crashes the rhythm path
- [ ] tempo and meter continuity policy is materially less duplicated
- [ ] recommendation changes are corpus-backed and inspectable

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

Switch into executable proof with `g09.011`, so the new capability claims are
demonstrable through repo-owned demos rather than roadmap prose alone.
