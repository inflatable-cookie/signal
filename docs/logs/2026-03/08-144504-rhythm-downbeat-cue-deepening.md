# Rhythm Downbeat Cue Deepening

Date: 2026-03-08
Owner: core-product

## Summary

Deepened the rhythm meter path with a second downbeat cue based on spectral
profile change, then hardened the bar-phase scorer against pickup bars and
mixed-meter material.

## Work completed

- added a broad-band spectral-profile change cue in `signal-analysis-rhythm`
  so downbeat inference can react to timbral or harmonic change instead of
  relying only on low-band or click-accent evidence
- combined the low-band cue and spectral-profile cue into a single meter cue
  surface before bar-phase scoring
- reweighted meter scoring so profile-change evidence can materially support
  weakly accented but bar-structured material
- added bar-strength regularity into meter confidence and suppression, which
  improves rejection of inconsistent bar-length patterns
- expanded the synthetic rhythm test harness with:
  - tone-burst events for bar-level spectral change
  - arbitrary beat-sequence construction for pickup and mixed-meter cases
- added adversarial tests covering:
  - 4/4 inference after a two-beat pickup
  - weakly accented 4/4 supported by bar-level spectral change
  - suppression on mixed 4-beat / 3-beat bar-length sequences

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo test --workspace`
- `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 120 --seconds 8`
- `git diff --check`

## Notes

- The neutral 120 BPM click demo remained stable at approximately 119.68 BPM
  with 0.934 overall confidence and still correctly reports:
  - `beats_per_bar=unknown`
  - `meter_confidence=0.000`
  - `downbeats=[]`
- The new meter cue work is aimed at structured but weakly accented material;
  it does not weaken the unknown-meter fallback for plain pulse trains.
- The meter path is still heuristic and offline. It now handles pickup bars and
  mixed-meter suppression more credibly, but it is not yet validated against
  real musical audio with evolving sections or sparse harmonic motion.

## Next Task

Move from synthetic-only meter validation toward more realistic offline fixtures:
add rendered rhythm-plus-tonal examples that include weak backbeats, sparse
harmonic changes, and section transitions, then tune the meter/downbeat scorer
against those fixtures before exposing the result surface to Finch integration.
