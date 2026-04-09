# 018 - g09.010 Rhythm Regression Corpus Freeze

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.010
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/078-rhythm-continuity-failure-containment-and-policy-normalization-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/010-rhythm-engine-resilience-and-policy-normalization.md
Auto-start next card: no

## Objective

Take the next honest `g09.010` proof seam by freezing a focused rhythm
regression corpus around the recently normalized tempo and meter policy
surfaces, using the existing preset-driven rhythm tests instead of widening
into new demo or feature work.

## Scope

- stay inside `crates/signal-analysis-rhythm` plus strict-lane docs surfaces
- focus on existing `RhythmPreset`-driven regression surfaces for tempo and
  meter continuity behavior
- turn the most relevant post-normalization continuity outputs into one
  explicit, inspectable regression bundle
- document any intentional preserved versus changed posture inside the batch
  evidence
- do not widen into interactive demo work, new heuristics, or new preset
  families

## Steps

1. Freeze the existing post-normalization rhythm regression surfaces that best
   cover worker containment plus tempo and meter continuity behavior.
2. Extract or group one focused corpus/regression proof lane around those
   surfaces.
3. Make preserved versus intentionally shifted posture explicit in tests or
   evidence.
4. Rerun focused rhythm regression validation plus repo health.
5. Reassess whether `g09.010` then closes or still needs a demo-adjacent
   planning handoff.

## Acceptance Criteria

- the recent rhythm policy work is covered by one explicit focused regression
  bundle
- recommendation posture remains inspectable rather than implied across many
  ad hoc tests
- focused validation passes

## Evidence Required

- batch log for the next `g09.010` tranche
- validation actually run
- explicit note that demo work remains deferred to `g09.011+`

## Stop Conditions

- the batch broadens into new rhythm heuristics or new preset creation
- the proof surface tries to replace the later interactive demo program
- the intended preserved versus shifted posture cannot be stated clearly from
  existing rhythm evidence

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/019-g09-011-demo-program-shape.md`.
