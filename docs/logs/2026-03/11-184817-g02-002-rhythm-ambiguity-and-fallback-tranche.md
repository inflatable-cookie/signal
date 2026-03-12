# 2026-03-11 18:48:17 GMT - g02.002 rhythm ambiguity and fallback tranche

## Summary

Extended `g02.002` beyond basic bar-span reporting by making ambiguity and
fallback state part of the public rhythm-structure contract.

This tranche gives downstream rhythm consumers an explicit assessment for weak
accent, syncopated/downbeat-phase competition, competing meter, and
recovery-window fallback instead of forcing them to infer those cases from raw
confidence deltas or private meter-state behavior.

## What changed

- extended `crates/signal-analysis-rhythm/src/lib.rs` with:
  - `RhythmStructureAmbiguityKind`
  - `RhythmStructureCandidate`
  - `RhythmStructureAmbiguitySummary`
  - `RhythmStructureFallbackSummary`
  - `RhythmStructureAssessment`
  - `BeatAnalysisResult::rhythm_structure_assessment()`
- preserved ambiguity evidence directly from meter inference, including:
  - primary and runner-up structure candidates
  - ambiguity confidence
  - trailing recovery-window confidence
- added focused fixtures that pin:
  - weak-accent ambiguity with usable structure retention
  - competing-meter ambiguity with visible candidate divergence
  - pickup-extension/downbeat-phase fallback behavior
  - recovery-window fallback when structure cannot be trusted
- updated the `g02.002` roadmap so ambiguity/fallback structure is recorded as
  complete evidence rather than residual scope

## Validation

- `cargo test -p signal-analysis-rhythm`

## Follow-on

The next `g02.002` batch should expose tempo-segment and continuity summaries
as stable public rhythm outputs, then decide whether bounded-streaming
validation belongs in the milestone.
