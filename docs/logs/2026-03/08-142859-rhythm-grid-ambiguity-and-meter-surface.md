# Rhythm Grid Ambiguity And Meter Surface

Date: 2026-03-08
Owner: core-product

## Summary

Extended the rhythm-analysis result surface so Signal now returns alternative
tempo hypotheses, explicit tempo ambiguity metadata, and a first bar-phase /
downbeat estimate alongside the primary beat grid.

## Work completed

- widened `signal-analysis-rhythm` result types with:
  - `tempo_candidates`
  - `tempo_ambiguity`
  - `meter` with `beats_per_bar`, confidence, and downbeat positions
- reworked tempo selection to keep the top ranked hypotheses instead of
  collapsing immediately to one BPM result
- added ambiguity scoring that stays elevated when runner-up tempo candidates
  remain competitive, especially for simple subdivision relations
- split beat placement into frame-space first, then derived beat seconds and
  downbeat seconds from the refined grid
- added first-pass meter inference over the recovered beat grid by scoring
  downbeat contrast and bar-phase support across 3-beat and 4-beat groupings
- calibrated meter confidence so unaccented pulse trains can still return a bar
  guess but stay visibly low-confidence
- updated the offline rhythm demo to print tempo candidates, ambiguity, and
  meter/downbeat metadata
- expanded rhythm tests to verify:
  - alternative tempo candidates on double-time ambiguity
  - silence clears ambiguity and meter output
  - 4/4 downbeat inference from accented bar patterns
  - 3/4 downbeat inference from waltz-style accents

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo test --workspace`
- `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 120 --seconds 8`
- `git diff --check`

## Notes

- The offline 120 BPM click demo remained stable at approximately 119.68 BPM
  with 0.934 overall confidence.
- The same unaccented click demo now reports tempo ambiguity and a low-confidence
  bar guess (`meter_confidence` approximately 0.124), which is preferable to
  overstating meter certainty where no real accent pattern exists.
- This is still a heuristic offline pass. The crate can now surface meter and
  ambiguity explicitly, but it does not yet infer downbeats from richer musical
  cues such as harmonic change, low-frequency accents, or section-level context.

## Next Task

Deepen beat-grid quality by adding stronger downbeat evidence beyond simple beat
accent contrast, such as sub-band accent profiles or harmonic-change cues, and
then expose a clearer “unknown / low-certainty meter” path so weakly accented
material does not need to masquerade as 3/4 or 4/4 at all.
