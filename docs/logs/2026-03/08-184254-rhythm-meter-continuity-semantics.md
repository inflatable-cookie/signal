# Rhythm Meter Continuity Semantics

Date: 2026-03-08
Owner: core-product

## Summary

Extended the public rhythm meter-state surface with explicit continuity
semantics so callers can decide independently whether to retain, reacquire, or
clear prior bar-length and downbeat-phase state during transition-heavy
material.

## Work completed

- added `MeterContinuityAction` and `MeterContinuityRecommendation` to
  `signal-analysis-rhythm`
- extended `MeterStateRecommendation` so every result now publishes separate
  continuity guidance for:
  - prior bar length
  - prior downbeat phase
- calibrated continuity behavior across promoted-meter and meterless paths:
  - stable whole-track meter locks both bar length and downbeat phase
  - tentative promoted meter retains bar length but reacquires downbeat phase
  - recovering segment meter retains bar length and reacquires downbeat phase
  - dropout-heavy hold states retain both dimensions
  - modulation-heavy clear states clear both dimensions
- added pickup-aware continuity handling so early displaced bar entry can keep
  bar length locked while still recommending downbeat-phase reacquisition
- updated the offline rhythm demo to print meter-state continuity semantics
- expanded the rhythm calibration tests so continuity behavior is explicit
  across:
  - structured stable material
  - tentative weak-backbeat material
  - sustained re-entry recovery
  - dropout-heavy meterless holds
  - modulation-heavy clears
  - pickup, mixed bar-length, and cadence/re-entry transition families

## Validation

- `cargo test -p signal-analysis-rhythm`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- Effigy runs were kept serial again to avoid the known workspace lock conflict
  when repo-owned tasks overlap.
- Continuity semantics are intentionally conservative: ambiguous or recovering
  material can preserve bar-length context without overstating downbeat-phase
  certainty.

## Next Task

Deepen the continuity surface with explicit retained-state provenance, such as
how long a prior bar-length or phase lock should remain trusted during hold or
watch states, then calibrate those expiry/revalidation rules across longer
dropout, pickup-extension, and multi-section re-entry fixtures.
