# 014 - g09.010 Tempo State Arm Unification

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.010
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/078-rhythm-continuity-failure-containment-and-policy-normalization-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/010-rhythm-engine-resilience-and-policy-normalization.md
Auto-start next card: no

## Objective

Take the first honest policy-normalization seam in `signal-analysis-rhythm` by
collapsing the near-copy tempo-state recommendation arms for snapped integer
tempo and refined-stable tempo into one explicit staged policy surface.

## Scope

- stay inside `crates/signal-analysis-rhythm`
- focus on `tempo_state_snap_integer_arm.rs` and
  `tempo_state_use_refined_stable_arm.rs`
- extract the shared continuity-stage shell into an explicit helper or policy
  table
- preserve the intentional coefficient and reason differences between the two
  arms
- add focused tests proving both recommendation families still diverge where
  expected while sharing one policy skeleton
- do not widen into meter policy normalization yet

## Steps

1. Freeze the true shared structure between the two tempo-state arms.
2. Extract the common staged continuity shell into one explicit helper or
   configuration-driven evaluator.
3. Keep integer-specific and refined-specific thresholds, reasons, and
   confidence scales explicit.
4. Add focused tests around shared structure and intentional divergence.
5. Rerun focused rhythm validation plus repo health.

## Acceptance Criteria

- duplicated tempo-state arm structure is materially reduced
- integer and refined tempo recommendations remain explicit and testable
- focused validation passes

## Evidence Required

- batch log for the next `g09.010` tranche
- validation actually run
- explicit note that meter-policy normalization remains deferred

## Stop Conditions

- the batch broadens into meter-policy or corpus-demo work
- the extracted helper hides the intentionally different confidence and
  continuity posture between integer and refined tempo recommendations

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/015-g09-010-meter-continuity-plan-shell-unification.md`.
