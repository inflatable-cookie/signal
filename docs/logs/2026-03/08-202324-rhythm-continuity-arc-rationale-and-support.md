# Rhythm Continuity Arc Rationale And Support

Date: 2026-03-08
Owner: core-product

## Summary

Extended Signal's public meter continuity arc surface again so each continuity
plan now publishes both a dominant arc rationale and a compact support profile.
This gives downstream consumers a Signal-owned explanation for why an arc is
classified as `Recovering`, `Stalling`, or `Collapsing`, instead of forcing
them to infer that from the lower-level history, cause-stack, and unresolved
span signals alone.

## Work completed

- added public `MeterContinuityArcRationale` and `MeterContinuityArcSupport`
  to `crates/signal-analysis-rhythm/src/lib.rs`
- updated `MeterContinuityPlan` to publish:
  - `arc_rationale`
  - `arc_support.refresh_strength`
  - `arc_support.drift_pressure`
  - `arc_support.structural_pressure`
- centralized arc-support and rationale derivation in
  `meter_state_recommendation(...)`
- calibrated the current Signal-owned rationale mapping so:
  - stable and sustained recovery-window arcs are explained by
    `RefreshStrength`
  - weak-backbeat hold and pickup-extension downbeat recovery can be explained
    by `UnresolvedDrift`
  - dropout-heavy stalling is explained by `StructuralInstability`
  - ambiguous and modulation-heavy collapsing paths are explained by
    `EvidenceLoss`
- updated the offline rhythm demo to print arc rationale and support values on
  each continuity plan
- added a dedicated calibration test covering rationale and relative support
  behavior across stable, tentative, ambiguous, pickup, dropout, sustained
  recovery, and modulation-heavy preset families

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 120 --seconds 8`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- The neutral 120 BPM click demo now reports
  `arc:Collapsing/EvidenceLoss/support:0.000,0.540,0.540` on both continuity
  dimensions, which makes the no-meter path more interpretable than the arc
  label alone.
- The non-PTY demo invocation still appeared to stall after process start in
  this environment, but rerunning the same command under a TTY completed
  normally and produced the expected output.

## Next Task

Add explicit arc-level persistence or handoff guidance on top of the new arc
rationale surface, such as whether callers should retain prior state, watch for
recovery, or clear immediately when an arc is `Stalling` or `Collapsing`, then
calibrate that guidance across chained pickup extensions, deeper dropout spans,
and multi-section recovery drift.
