# Rhythm Continuity Reason And Confidence Surface

Date: 2026-03-08
Owner: core-product

## Summary

Extended Signal's staged meter continuity lifecycle again so each continuity
stage now carries both a categorical rationale and a stage-local confidence.
This lets downstream consumers distinguish stable evidence, tentative carry,
prior-state hold, recovery-window support, pickup-like phase displacement, and
revalidation decay without reverse-engineering those semantics from action and
severity alone.

## Work completed

- added `MeterContinuityReason` to the public rhythm surface
- updated `MeterContinuityPlan` to publish:
  - `reason`
  - `confidence`
- updated each `MeterContinuityTransition` to publish:
  - `reason`
  - `confidence`
- centralized continuity rationale and confidence mapping in
  `meter_state_recommendation(...)`
- established a Signal-owned reason vocabulary across the existing lifecycle:
  - `StableEvidence`
  - `TentativeEvidence`
  - `PriorStateCarry`
  - `RecoveryWindowSupport`
  - `PhaseDisplacement`
  - `RevalidationDecay`
  - `InsufficientEvidence`
- updated the offline rhythm demo to print reason and confidence for the
  current continuity stage plus both lifecycle transitions
- added a dedicated calibration test that checks:
  - reason assignment across stable, tentative, prior-carry, recovery-window,
    pickup-phase, and cleared cases
  - refresh transitions outranking degraded stages where recovery is expected
  - longer sustained recovery carrying at least as much bar-length continuity
    confidence as the shorter sustained recovery case

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 120 --seconds 8`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- This batch keeps the current fixture families and deepens the public semantic
  layer instead of multiplying scenario variants again.
- Stage confidence is intentionally local to the continuity lifecycle and does
  not replace the higher-level meter confidence or trust surfaces already
  exposed by Signal.

## Next Task

Add explicit downgrade-trigger metadata or unresolved-span counters to the
continuity lifecycle, then calibrate how repeated failed revalidation across
chained pickup extensions, deeper dropout spans, and multi-section recovery
drift should change the published reason and confidence from one stage to the
next.
