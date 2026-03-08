# Offline Tonal Slice Bootstrap

Date: 2026-03-08
Owner: core-product

## Summary

Expanded the Rust workspace with the first usable tonal-analysis slice.

## Work completed

- added chroma extraction support in `signal-dsp-spectral`
- implemented a first key-detection path in `signal-analysis-tonal`
- added Krumhansl and Temperley profile support
- added correlation-based key selection and relative-margin confidence
- added a tiny offline tonal demo harness:
  - `cargo run -p signal-analysis-tonal --example offline_tonal_demo -- c-major`
  - `cargo run -p signal-analysis-tonal --example offline_tonal_demo -- a-minor`
- added focused tests for:
  - chroma extraction around a known pitch class
  - C major triad detection
  - A minor triad detection

## Validation

- `cargo test -p signal-analysis-tonal`
- `cargo test --workspace`
- `git diff --check`

## Notes

This is a deliberate first-pass tonal baseline:

- 12-bin chroma rather than full HPCP
- basic profile correlation rather than Essentia-level multi-profile tuning
- confidence reflects margin over the runner-up, which stays small on relative
  major/minor-ambiguous material such as bare triads

## Next Task

Choose the next Signal implementation slice:

1. extend tonal analysis toward HPCP, tuning correction, and stronger profile
   handling, or
2. build `signal-analysis-loudness` into a real LUFS/true-peak baseline.
