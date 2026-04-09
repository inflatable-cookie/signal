# 016 - g09.010 Meter Continuity Trigger Cause Normalization

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.010
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/078-rhythm-continuity-failure-containment-and-policy-normalization-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/010-rhythm-engine-resilience-and-policy-normalization.md
Auto-start next card: no

## Objective

Take the next honest `g09.010` policy-normalization seam by reducing the
spread-out derivation logic behind meter continuity trigger, reason, and cause
selection now split across `meter_state_continuity_helpers.rs` and
`meter_state_continuity_cause_stack.rs`.

## Scope

- stay inside `crates/signal-analysis-rhythm`
- focus on meter continuity trigger/reason/cause derivation only
- extract repeated or parallel derivation logic into one explicit rule surface
- keep the intentional distinctions between stable, tentative, recovery-window,
  phase-displacement, and clear behavior explicit
- add focused tests proving the normalized derivation still preserves the
  current trigger/cause posture
- do not widen into new meter heuristics, corpus work, or demo work yet

## Steps

1. Freeze the overlapping trigger/reason/cause derivation logic.
2. Extract one explicit rule surface or configuration-backed evaluator for the
   shared derivation shell.
3. Keep policy-specific differences explicit in configuration or rule entries.
4. Add focused tests around the normalized derivation surface.
5. Rerun focused rhythm validation plus repo health.

## Acceptance Criteria

- duplicated or parallel meter continuity derivation logic is materially reduced
- trigger, reason, and cause posture remain explicit and testable
- focused validation passes

## Evidence Required

- batch log for the next `g09.010` tranche
- validation actually run
- explicit note that corpus-proof and demo work remain deferred

## Stop Conditions

- the batch broadens into new meter heuristics or public rhythm behavior
- the normalized derivation surface obscures the intended stable, tentative,
  recovery, phase, or clear distinctions

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/017-g09-010-meter-stage-plan-context-unification.md`.
