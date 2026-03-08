# Rhythm Continuity Stage Severity

Date: 2026-03-08
Owner: core-product

## Summary

Extended Signal's staged meter continuity lifecycle with explicit per-stage
severity metadata so consumers can distinguish confirmed continuity, guarded
retention, fragile reacquisition, and fully cleared state without inventing
their own thresholds from action labels alone.

## Work completed

- added `MeterContinuitySeverity` to the public rhythm surface
- updated `MeterContinuityPlan` to publish the severity of the current
  continuity stage
- updated each `MeterContinuityTransition` to publish the severity of its
  refresh and decay stage
- centralized severity mapping in `meter_state_recommendation(...)` so staged
  continuity severity is derived consistently from action and evidence source:
  - `Lock` maps to `Confirmed`
  - retained continuity from `CurrentMeter` or `RecoveryWindow` maps to
    `Guarded`
  - retained continuity from `PriorMeter` and all reacquisition stages map to
    `Fragile`
  - cleared continuity maps to `Cleared`
- updated the offline rhythm demo to print stage severity alongside action,
  source, and beat horizons
- added a dedicated calibration test that pins severity behavior across:
  - stable structured meter
  - tentative weak-backbeat meter
  - heavy and extended-heavy dropout
  - pickup-extension phase reacquisition
  - sustained and long sustained recovery
  - mixed bar-length cleared state

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 120 --seconds 8`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- This batch kept the existing fixture families and used them to calibrate the
  new severity surface, so the work materially strengthens Signal's public
  semantics without broadening the scenario matrix again.
- Severity is intentionally categorical rather than numeric in this pass so
  Finch can consume a Signal-owned interpretation layer immediately without
  product-side score slicing.

## Next Task

Add per-stage refresh confidence or downgrade reason metadata on top of the new
severity surface, then calibrate how continuity should weaken across even
longer repeated unresolved bars such as chained pickup extensions, deeper
dropout spans, and multi-section recovery drift.
