# 013 - g09.010 Rhythm Worker Failure Containment

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.010
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/078-rhythm-continuity-failure-containment-and-policy-normalization-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/010-rhythm-engine-resilience-and-policy-normalization.md
Auto-start next card: no

## Objective

Start `g09.010` with the narrowest resilience seam in
`signal-analysis-rhythm`: remove production `join().unwrap()` worker failure
crashes from onset feature extraction and replace them with typed degraded
feature availability that preserves deterministic output shape.

## Scope

- stay inside `crates/signal-analysis-rhythm`
- focus on `onset_features.rs` worker failure containment
- replace crash-on-worker-loss with typed degraded-path behavior
- add targeted failure-injection tests around partial worker loss
- do not widen into tempo/meter policy normalization yet

## Steps

1. Extract the onset worker-join behavior into an explicit failure-containment
   path.
2. Replace `join().unwrap()` with typed degraded or failed feature results.
3. Keep output determinism explicit when one or more feature workers fail.
4. Add focused tests for worker panic or injected failure.
5. Rerun the focused rhythm validation surface plus repo health.

## Acceptance Criteria

- rhythm worker failure no longer crashes production onset feature extraction
- degraded feature availability is explicit and testable
- focused validation passes

## Evidence Required

- batch log for the next `g09.010` tranche
- validation actually run
- explicit note that tempo/meter policy normalization remains deferred to later
  `g09.010` work

## Stop Conditions

- the batch starts rewriting tempo or meter policy families instead of
  containing worker failure
- the change broadens into demo or corpus-policy work

## Next Task

Continue the active strict `g09` lane from
`docs/roadmaps/g09/batch-cards/014-g09-010-tempo-state-arm-unification.md`.
