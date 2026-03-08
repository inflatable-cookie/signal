# Rhythm Continuity Triggers And Unresolved Spans

Date: 2026-03-08
Owner: core-product

## Summary

Extended Signal's staged meter continuity lifecycle with explicit downgrade
trigger metadata and unresolved-span counters so consumers can see not only how
continuity changes over time, but also what kind of instability is driving the
change and how many beats, bars, and failed revalidation intervals are behind
that stage.

## Work completed

- updated the rhythm public surface in `crates/signal-analysis-rhythm` with:
  - `MeterContinuityTrigger`
  - `MeterContinuityUnresolvedSpan`
- extended `MeterContinuityPlan` and `MeterContinuityTransition` so each stage
  now publishes:
  - `trigger`
  - `unresolved.beats`
  - `unresolved.bars`
  - `unresolved.failed_revalidations`
- added Signal-owned trigger semantics across the existing lifecycle:
  - `StableRevalidation`
  - `TentativeCarry`
  - `PhaseRecovery`
  - `PriorStateDrift`
  - `RecoveryWindowDrift`
  - `EvidenceLoss`
- updated `meter_state_recommendation(...)` so unresolved-span counters are
  computed alongside action, reason, severity, and confidence
- threaded the recovered beat grid into `meter_state_recommendation(...)` so
  phase-recovery counters can measure leading pickup length from the actual beat
  sequence instead of only inferring from the first downbeat
- updated the offline rhythm demo to print trigger and unresolved-span metadata
  for the current continuity stage and both lifecycle transitions
- added a dedicated calibration test that pins trigger and unresolved-span
  behavior across:
  - stable structured meter
  - tentative weak-backbeat carry
  - pickup and pickup-extension phase recovery
  - heavy and extended-heavy dropout
  - sustained and longer sustained recovery-window drift
  - mixed bar-length evidence loss

## Validation

- `cargo test -p signal-analysis-rhythm`
- attempted `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 120 --seconds 8`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- The repo has moved Rust crates under `crates/`; this batch was implemented
  and validated against the new layout without changing the rhythm contract's
  ownership or intent.
- The current contract distinguishes pickup extension through a longer phase
  recovery decay path rather than a larger immediate reacquisition span, which
  is now explicit in the published counters.
- The offline rhythm demo now emits a much larger continuity line. In this
  environment the example run did not return normally after this batch, so the
  passing validation signal for the work is the crate test suite plus serial
  Effigy validation rather than a completed demo run.

## Next Task

Add stage-to-stage downgrade event history or suppression-cause metadata on top
of the new trigger and unresolved-span surface, then calibrate how stacked
causes such as dropout plus harmonic drift or pickup plus ambiguity should alter
the published trigger, reason, and confidence across the lifecycle.
