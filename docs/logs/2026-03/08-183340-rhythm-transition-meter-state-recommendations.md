# Rhythm Transition Meter State Recommendations

Date: 2026-03-08
Owner: core-product

## Summary

Extended the public rhythm result surface with a top-level transition-aware
meter state recommendation so callers can decide whether to lock, hold, watch,
or clear meter-dependent behavior even when `meter` is `None`.

## Work completed

- added `MeterStateAction`, `MeterStateReason`, and
  `MeterStateRecommendation` to `signal-analysis-rhythm`
- extended `BeatAnalysisResult` so every analysis now publishes a top-level
  `meter_state` alongside the optional promoted `meter`
- refactored meter inference to return both:
  - the promoted `MeterEstimate`, when available
  - suppression/recovery evidence for meterless outcomes
- added trailing-window meter evidence so transition-heavy sections can surface
  emerging recovery without forcing a promoted bar estimate
- calibrated the new public meter-state contract so:
  - strong whole-track meter maps to `Lock`
  - sustained segment recovery maps to `Watch`
  - tentative meter claims map to `Hold`
  - dropout-heavy meterless transitions map to `Hold`
  - reset/re-anchor meterless transitions map to `Watch`
  - modulation-heavy meterless transitions map to `Clear`
- updated the offline rhythm demo to print the new meter-state action, reason,
  and confidence
- expanded the test surface with explicit transition-aware meter-state
  assertions for meterless dropout, recovery, and modulation families

## Validation

- `cargo test -p signal-analysis-rhythm`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- Effigy runs were kept serial again to avoid the known workspace lock conflict
  when repo-owned tasks overlap.
- The no-meter state split currently leans on ambiguity plus trailing recovery
  evidence, which is good enough for the current synthetic transition families
  but should be revisited once richer offline fixtures are available.

## Next Task

Deepen the transition-aware state surface with explicit continuity semantics,
such as whether callers should retain a prior bar length or downbeat phase
while holding meter state, then calibrate that behavior across pickup, mixed
bar-length, and cadence/re-entry families.
