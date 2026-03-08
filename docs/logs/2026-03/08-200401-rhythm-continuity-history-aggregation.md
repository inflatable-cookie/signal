# Rhythm Continuity History Aggregation

Date: 2026-03-08
Owner: core-product

## Summary

Extended Signal's public meter continuity lifecycle again so each current
continuity plan and future transition now publishes an explicit stage-history
classification: `Reinforcing`, `Preserving`, or `Degrading`. This gives
downstream consumers a Signal-owned view of whether a lifecycle step is
strengthening current meter state, merely carrying it forward, or actively
losing continuity under ambiguity or structural instability.

## Work completed

- added public `MeterContinuityHistory` to
  `crates/signal-analysis-rhythm/src/lib.rs`
- updated both `MeterContinuityPlan` and `MeterContinuityTransition` to publish
  stage-history alongside the existing severity, reason, confidence, trigger,
  unresolved-span, and cause-stack surfaces
- centralized history mapping in `meter_state_recommendation(...)` using:
  - action and source
  - lifecycle reason and trigger
  - stage confidence
  - unresolved revalidation count
  - stacked continuity causes
- calibrated the mapping so:
  - stable current-meter locks reinforce continuity
  - prior-meter and recovery-window retains preserve continuity until
    degradation conditions accumulate
  - phase-reacquisition and evidence-loss stages degrade continuity
- updated the offline rhythm demo to print history on both the current
  continuity plan and each lifecycle transition
- added a dedicated calibration test covering:
  - stable structured meter
  - tentative weak-backbeat meter
  - pickup-extension phase recovery
  - extended dropout carry and decay
  - sustained late recovery refresh
  - modulation-heavy cleared state

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 120 --seconds 8`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- The neutral 120 BPM click demo now completes with explicit degrading
  continuity stages for both bar length and downbeat phase, which makes the
  no-meter path easier to consume than the previous action-only output.
- History is still heuristic and fixture-calibrated. It explains directional
  continuity change, but it does not yet summarize whether multiple consecutive
  stages collectively form a recovery arc versus a collapse.

## Next Task

Add explicit lifecycle-arc aggregation above the new history surface, such as
whether the full continuity path is recovering, stalling, or collapsing across
multiple stages, then calibrate that arc signal across chained pickup
extensions, deeper dropout spans, and multi-section recovery drift.
