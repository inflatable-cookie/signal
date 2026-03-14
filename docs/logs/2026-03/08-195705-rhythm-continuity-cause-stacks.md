# Rhythm Continuity Cause Stacks

Date: 2026-03-08
Owner: core-product

## Summary

Extended Signal's public meter continuity lifecycle again so each current stage
and future transition now carries a compact stacked-cause summary instead of a
single trigger alone. This gives downstream consumers a Signal-owned view of
what is destabilizing continuity in combination, such as evidence loss plus
tempo ambiguity, recovery-window instability plus irregular bar structure, or
pickup-phase displacement plus later evidence loss.

## Work completed

- added `MeterContinuityCause` and `MeterContinuityCauseStack` to the public
  rhythm surface in `crates/signal-analysis-rhythm`
- updated `MeterContinuityPlan` and `MeterContinuityTransition` to publish
  stacked causes alongside the existing action, source, severity, reason,
  confidence, trigger, and unresolved-span data
- centralized stacked-cause derivation in `meter_state_recommendation(...)`
  using:
  - lifecycle reason
  - trigger
  - suppression profile strength and regularity
  - tempo ambiguity
  - phase displacement
  - stage position within the lifecycle
- refined the offline rhythm demo output so continuity state is printed as
  short structured lines instead of a single oversized line, and each line now
  includes the stacked-cause summary
- added calibration coverage that pins representative stacked-cause behavior
  across:
  - ambiguous subdivision material
  - pickup-extension decay
  - extended dropout instability
  - modulation-heavy evidence loss

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 120 --seconds 8`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- The neutral 120 BPM click demo now completes normally again after the example
  output refactor and reports `meter_state=Clear` with explicit
  `EvidenceLoss+TempoAmbiguity+SparseMeterSupport` continuity causes.
- The current calibration intentionally distinguishes extended dropout from
  sparse-support ambiguity: dropout retains a recovery-window instability cause
  stack with irregularity, rather than being mislabeled as sparse support.

## Next Task

Add explicit stage-history aggregation on top of the new cause-stack surface,
such as whether each lifecycle step is reinforcing, preserving, or degrading
continuity, then calibrate those history semantics across chained pickup
extensions, deeper dropout spans, and multi-section recovery drift.
