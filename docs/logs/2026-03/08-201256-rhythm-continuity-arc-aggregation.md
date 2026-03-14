# Rhythm Continuity Arc Aggregation

Date: 2026-03-08
Owner: core-product

## Summary

Extended Signal's public meter continuity surface again so each continuity plan
now publishes a full lifecycle arc: `Recovering`, `Stalling`, or
`Collapsing`. This sits above the existing stage-history and cause-stack layers
and gives downstream consumers a concise Signal-owned read of whether a
continuity path is trending back toward stable meter, merely holding in place,
or actively falling apart across its staged transitions.

## Work completed

- added public `MeterContinuityArc` to
  `crates/signal-analysis-rhythm/src/lib.rs`
- updated `MeterContinuityPlan` to publish an `arc` alongside the existing
  current-stage action, source, severity, history, reason, confidence, trigger,
  unresolved-span, and cause-stack fields
- centralized lifecycle-arc aggregation in `meter_state_recommendation(...)`
  using:
  - current stage history
  - refresh-stage history
  - decay-stage history
  - current confidence
  - unresolved revalidation count
  - presence of evidence loss
  - irregular bar-structure causes
- calibrated the current Signal-owned arc semantics so:
  - stable locked whole-track continuity reports `Recovering`
  - tentative weak-backbeat carry reports `Stalling`
  - ambiguous and modulation-heavy cleared paths report `Collapsing`
  - pickup-extension phase recovery reports `Collapsing`
  - extended dropout carry reports `Stalling`
  - sustained recovery-window paths without irregular bar structure report
    `Recovering`
- updated the offline rhythm demo to print the plan-level arc next to the
  current continuity state
- added a dedicated calibration test that pins arc behavior across stable,
  tentative, ambiguous, pickup, dropout, sustained recovery, and
  modulation-heavy families

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 120 --seconds 8`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- The neutral 120 BPM click demo now reports `arc:Collapsing` for both bar
  length and downbeat phase, which makes the no-meter path easier to interpret
  at a glance than the lower-level stage fields alone.
- The current arc classifier intentionally distinguishes sustained recovery from
  dropout carry by treating irregular bar structure as a blocker for
  `Recovering` even when recovery-window confidence is otherwise high.

## Next Task

Add explicit arc-support or arc-rationale metadata above the new arc surface,
such as whether a `Recovering` or `Collapsing` classification is driven more by
refresh strength, unresolved drift, or structural irregularity, then calibrate
that rationale across chained pickup extensions, deeper dropout spans, and
multi-section recovery drift.
