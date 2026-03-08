# Rhythm Meter Unknown Gating

Date: 2026-03-08
Owner: core-product

## Summary

Strengthened rhythm meter inference with an extra low-band accent cue and added
an explicit unknown-meter path so weakly accented material no longer has to
pretend to be 3/4 or 4/4.

## Work completed

- added a meter-specific low-band flux cue in `signal-analysis-rhythm` to give
  downbeat inference stronger evidence than broadband beat accent alone
- fed the low-band cue into bar-phase scoring so 3-beat and 4-beat hypotheses
  can use both general beat salience and low-frequency accent contrast
- tightened meter support scoring and added an explicit suppression gate:
  - weak bar evidence now returns `meter: None`
  - meter confidence only survives when both bar salience and winner margin are
    strong enough
- updated the plain click-track test contract so stable tempo detection does not
  imply a forced meter estimate

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo test --workspace`
- `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 120 --seconds 8`
- `git diff --check`

## Notes

- The 120 BPM offline click demo remained stable at approximately 119.68 BPM
  with 0.934 overall confidence and 0.354 tempo ambiguity.
- After the gating change, that same unaccented click demo now reports:
  - `beats_per_bar=unknown`
  - `meter_confidence=0.000`
  - `downbeats=[]`
- This is a better contract for Finch and future consumers because low-certainty
  meter now stays explicit instead of surfacing a misleading bar structure.

## Next Task

Deepen downbeat inference with richer musical cues that can survive beyond
simple kick/accent heuristics, such as harmonic-change or sub-band profile
features, then validate the combined meter path against more adversarial
patterns including weak backbeats, pickup bars, and bar-length changes.
