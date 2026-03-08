# Multifeature Rhythm Deepening

Date: 2026-03-08
Owner: core-product

## Summary

Hardened the first offline rhythm-analysis slice with a stronger onset envelope
and more stable tempo scoring.

## Work completed

- replaced the single-feature onset envelope in `signal-analysis-rhythm` with a
  weighted multifeature onset path:
  - spectral flux
  - high-frequency content
  - energy flux
- upgraded tempo estimation from a basic autocorrelation peak search to a
  harmonic-aware scorer with runner-up comparison
- preserved the existing offline demo harness and expanded rhythm-focused tests
  to cover slower click tracks and non-empty onset extraction

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 120 --seconds 8`
- `cargo test --workspace`
- `git diff --check`

## Notes

The 120 BPM synthetic click-track demo remained stable at approximately 119.68
BPM after the onset and tempo changes. Confidence reduced relative to the first
slice because the scorer now penalizes ambiguity more aggressively.

## Next Task

Push rhythm analysis further with stronger onset discrimination and tempo
ambiguity handling: add complex-domain or bandwise flux features, improve beat
phase selection, and introduce adversarial tests for swung, syncopated, and
double-time click patterns.
