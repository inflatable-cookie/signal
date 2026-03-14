# Rhythm Meter Support Calibration Surface

Date: 2026-03-08
Owner: core-product

## Summary

Extended the public meter surface so callers can distinguish whole-track meter
support from segment-recovery support without reverse-engineering one scalar
confidence. Signal meter results now expose a support profile with separate
whole-track, segment-recovery, and recovery-duration strengths.

## Work completed

- added `MeterSupportProfile` to `signal-analysis-rhythm`
- extended `MeterEstimate` so each meter claim now carries:
  - `whole_track_strength`
  - `segment_recovery_strength`
  - `recovery_duration_strength`
- added recovery-duration calibration based on recovered beat span and the
  number of supporting local windows
- updated `infer_meter(...)` so both whole-track and segment candidates inform
  the published support profile, even when only one candidate is selected
- kept the provenance-aware chooser from the prior batch and now expose enough
  support detail for downstream consumers to interpret:
  - strong stable whole-track claims
  - weaker but acceptable segment recoveries
  - no-meter outcomes with no promoted support surface
- updated the offline rhythm demo to print the new support profile
- strengthened the tests so the current Signal contract is explicit:
  - structured stable fixtures report stronger whole-track support than segment
    support
  - sustained recovery fixtures report stronger segment support than whole-track
    support, with non-trivial recovery-duration strength

## Validation

- `cargo test -p signal-analysis-rhythm`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- Effigy validation was kept serial for this batch to avoid the known workspace
  lock conflict when repo-owned tasks overlap.
- One old relative meter-confidence ordering assertion was intentionally left
  removed; the support-profile assertions are a better contract than brittle
  comparisons between unrelated fixture families.

## Next Task

Add an explicit meter-trust or recommendation layer on top of the support
profile, then calibrate categories like stable, recovering, and tentative so
Finch can consume Signal-owned meter semantics directly instead of inventing its
own threshold heuristics.
