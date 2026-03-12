# 2026-03-11 19:26:29 GMT - g02.003 tonal tuning and scoring opening tranche

## Summary

Opened `g02.003` by extending `signal-analysis-tonal` from a bare whole-track
key detector into an explicit tuning-reference and scoring surface.

This tranche gives downstream tonal consumers a stable report for estimated or
fixed tuning reference, key-profile ranking, and frozen analysis-tier behavior
before section-local harmonic tracking is introduced.

## What changed

- extended `crates/signal-analysis-tonal/src/lib.rs` with:
  - `TuningReferenceMode`
  - `TuningReferenceSource`
  - `TuningCandidate`
  - `TuningEstimate`
  - `TonalProfileCandidate`
  - `TonalScoringSummary`
- updated `KeyDetectorConfig` so low/medium/high profiles now freeze explicit
  tuning search defaults
- updated `TonalAnalysisResult` so tonal analysis now reports:
  - tuning reference and confidence
  - runner-up tuning candidate
  - key-profile scoring summary
- added focused fixtures that pin:
  - stable major/minor key detection under estimated tuning
  - detuned material with recovered tuning reference
  - fixed-reference reporting
  - non-native-rate stability under the frozen substrate

## Validation

- `cargo fmt`
- `cargo test -p signal-analysis-tonal`

## Follow-on

The next `g02.003` batch should add section-local key tracking and
harmonic-change evidence on top of the new tuning/scoring surface, then pin
that behavior with modulating fixtures.
