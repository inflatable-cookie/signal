# 2026-03-11 19:45:23 GMT - g02.003 local tonal tracking and change tranche

## Summary

Extended `g02.003` from tuning-aware whole-track key detection into a section-
local tonal tracking surface with explicit harmonic-change evidence.

This tranche gives downstream consumers overlapping local-key windows and
change events on top of the same tuning/scoring substrate, so modulation and
harmonic movement no longer have to be inferred from one global key label.

## What changed

- extended `crates/signal-analysis-tonal/src/lib.rs` with:
  - `TonalSegmentSummary`
  - `HarmonicChangeKind`
  - `HarmonicChangeSummary`
  - `LocalTonalTrackingSummary`
- updated `KeyDetectorConfig` so low/medium/high profiles now freeze explicit
  section-window defaults
- updated `TonalAnalysisResult` so tonal analysis now reports:
  - section-local tonal segments
  - harmonic-change timing and from/to key evidence
  - chroma-distance support for each detected change
- added focused fixtures that pin:
  - stable local key tracking for steady C-major material
  - explicit harmonic-change detection for a C-major to G-major modulation

## Validation

- `cargo fmt`
- `cargo test -p signal-analysis-tonal`

## Follow-on

The next `g02.003` batch should make modulation and mixed-tonality ambiguity
explicit in the local tonal API, then pin that behavior with weak-tonal-centre
and ambiguous-tracking fixtures.
