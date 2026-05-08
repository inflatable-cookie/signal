# 015 - g09.010 Meter Continuity Plan Shell Unification

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.010
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/078-rhythm-continuity-failure-containment-and-policy-normalization-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/010-rhythm-engine-resilience-and-policy-normalization.md
Auto-start next card: no

## Objective

Take the next honest `g09.010` policy-normalization seam by collapsing the
repeated meter continuity stage-plan shell now spread across
`meter_state_continuity_hold_arms.rs`, `meter_state_continuity_lock_arms.rs`,
and `meter_state_continuity_watch_clear_arms.rs`.

## Scope

- stay inside `crates/signal-analysis-rhythm`
- focus on meter continuity plan construction only
- extract the repeated staged-plan shell into one explicit helper or
  configuration-driven builder
- keep the intentional hold, lock, watch, and clear behavior differences
  explicit
- add focused tests proving shared shell reuse without hiding meter-specific
  policy posture
- do not widen into new meter heuristics or corpus-demo work yet

## Steps

1. Freeze the repeated stage-plan shell across the meter continuity arm files.
2. Extract one explicit helper or staged builder for the repeated plan pattern.
3. Keep hold, lock, watch, and clear policy differences explicit in their
   configuration and reasons.
4. Add focused tests around shared shell reuse and intentional divergence.
5. Rerun focused rhythm validation plus repo health.

## Acceptance Criteria

- duplicated meter continuity plan structure is materially reduced
- meter continuity behaviors remain explicit and testable
- focused validation passes

## Evidence Required

- batch log for the next `g09.010` tranche
- validation actually run
- explicit note that corpus-proof and demo work remain deferred

## Stop Conditions

- the batch broadens into meter heuristics or user-facing rhythm behavior
- the extracted helper obscures the intended distinctions between hold, lock,
  watch, and clear continuity posture

## Next Task

Continue the active strict `g09` lane from
`docs/roadmaps/g09/batch-cards/016-g09-010-meter-continuity-trigger-cause-normalization.md`.
