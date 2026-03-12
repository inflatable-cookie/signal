# 2026-03-11 18:30:27 GMT - g02.002 rhythm structure summary opening tranche

## Summary

Opened `g02.002` by adding a compact rhythm-structure summary on top of the
existing meter/downbeat inference rather than adding another parallel
heuristics layer.

This tranche gives downstream timeline and bar-grid consumers one stable
structure surface for bar spans, downbeat continuity, and recovery-backed meter
support without forcing them to reconstruct that view from raw downbeat arrays,
recovery windows, and continuity internals.

## What changed

- extended `crates/signal-analysis-rhythm/src/lib.rs` with:
  - `BarSupportKind`
  - `BarSpan`
  - `RhythmStructureContinuitySummary`
  - `RhythmStructureSummary`
  - `BeatAnalysisResult::rhythm_structure_summary()`
- derived bar spans directly from the current `MeterEstimate`, preserving:
  - whole-track support
  - recovery-window support
  - extrapolated support outside the recovered region
- added focused rhythm fixtures that pin:
  - stable whole-track structure summaries for structured meter
  - recovery-backed structure summaries for segment-recovery meter cases

## Validation

- `cargo fmt`
- `cargo test -p signal-analysis-rhythm`

## Follow-on

The next `g02.002` batch should expose stronger ambiguity and fallback
structure for weak-accent, syncopated, and competing-meter material, then
decide whether bounded-streaming rhythm validation belongs in the milestone.
