# Rhythm Meter Consumption Recommendations

Date: 2026-03-08
Owner: core-product

## Summary

Extended the public rhythm meter surface with a Signal-owned consumption
recommendation so downstream callers can decide whether to lock, monitor, or
defer meter-dependent behavior without inventing wrapper-specific thresholds.

## Work completed

- added `MeterRecommendation` to `signal-analysis-rhythm` with:
  - `Lock`
  - `Monitor`
  - `Defer`
- extended `MeterEstimate` so each promoted meter claim now carries:
  - detection provenance
  - trust level
  - consumption recommendation
  - support profile
  - confidence breakdown
  - recovery context
- added recommendation calibration logic that maps:
  - strong whole-track stable meter to `Lock`
  - segment-recovery meter to `Monitor`
  - weaker tentative whole-track meter to `Defer`
- updated `infer_meter(...)` so whole-track and segment-recovery candidates both
  publish the new recommendation field
- updated the offline rhythm demo to print `meter_recommendation`
- expanded calibration coverage so the current Signal contract is explicit:
  - structured active four-four recommends `Lock`
  - sustained late recovery recommends `Monitor`
  - weak backbeat promotion recommends `Defer`

## Validation

- `cargo test -p signal-analysis-rhythm`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- Effigy runs were kept serial again to avoid the known workspace lock conflict
  when repo-owned tasks overlap.
- The recommendation layer is intentionally conservative: only strong
  whole-track claims escalate to `Lock`, while recoveries and borderline
  promotions remain explicit monitor-or-defer outputs rather than looking fully
  settled.

## Next Task

Add richer recommendation semantics for transition-heavy material, such as
whether callers should hold a prior meter lock, watch for recovery, or clear
meter state after destabilization, then calibrate that behavior across
dropout-heavy, modulation-heavy, and re-anchor fixture families.
