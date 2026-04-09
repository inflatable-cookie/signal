# 017 - g09.010 Meter Stage Plan Context Unification

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.010
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/078-rhythm-continuity-failure-containment-and-policy-normalization-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/010-rhythm-engine-resilience-and-policy-normalization.md
Auto-start next card: no

## Objective

Take the next honest `g09.010` policy-normalization seam by reducing the
duplicated stage-versus-plan continuity assembly now concentrated in
`meter_state_continuity_context.rs`.

## Scope

- stay inside `crates/signal-analysis-rhythm`
- focus on shared trigger, unresolved-span, cause-stack, and confidence
  assembly for meter continuity stage and plan creation
- extract one explicit shell or builder for the repeated context assembly
- keep the real differences between transition-stage construction and final
  plan construction explicit
- add focused tests proving the shared shell preserves current continuity plan
  posture
- do not widen into new meter heuristics, corpus work, or demo work yet

## Steps

1. Freeze the duplicated stage and plan assembly shell in
   `meter_state_continuity_context.rs`.
2. Extract one explicit helper or builder for the shared assembly path.
3. Keep stage-only and plan-only differences explicit at the call sites.
4. Add focused tests around the normalized stage/plan shell.
5. Rerun focused rhythm validation plus repo health.

## Acceptance Criteria

- duplicated meter continuity stage/plan assembly is materially reduced
- stage and final-plan distinctions remain explicit and testable
- focused validation passes

## Evidence Required

- batch log for the next `g09.010` tranche
- validation actually run
- explicit note that corpus-proof and demo work remain deferred

## Stop Conditions

- the batch broadens into new meter heuristics, tempo policy work, or public
  rhythm behavior
- the shared shell obscures the intentional difference between intermediate
  transitions and final continuity plans

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether
`g09.010` closes here or promotes a new bounded corpus-proof or demo-adjacent
batch before creating another ready card.
