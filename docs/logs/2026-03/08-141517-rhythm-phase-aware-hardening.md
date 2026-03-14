# Rhythm Phase-Aware Hardening

Date: 2026-03-08
Owner: core-product

## Summary

Deepened the offline rhythm-analysis slice with phase-aware spectral features,
candidate tempo rescoring, and stronger beat-phase selection so the crate is
less fragile on swung, syncopated, and subdivision-heavy material.

## Work completed

- extended `signal-dsp-spectral` STFT frames to retain per-bin phase alongside
  magnitudes so downstream analysis crates can reuse a richer spectral surface
- hardened `signal-analysis-rhythm` onset extraction with:
  - bandwise spectral flux
  - complex-domain spectral difference
  - local-mean onset sharpening before normalization
- reworked tempo estimation to:
  - score and rank local tempo candidates
  - rescore hypotheses with beat-phase support instead of trusting raw
    autocorrelation alone
  - carry the selected lag/phase through beat placement
- replaced the old max-onset beat anchor with phase-selected beat tracking plus
  local refinement
- expanded rhythm-focused tests with adversarial synthetic patterns covering:
  - swung eighth feel
  - syncopated offbeat emphasis
  - double-time subdivision ambiguity
  - single loud offbeat phase distraction

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo test --workspace`
- `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 120 --seconds 8`
- `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 90 --seconds 8`
- `git diff --check`

## Notes

- The offline demo remained stable at approximately 119.68 BPM for the 120 BPM
  click track and improved to approximately 90.73 BPM with 0.860 confidence for
  the 90 BPM click track.
- `effigy health` was attempted first per repo guidance but behaved as
  a heavyweight CMake/build path in this workspace, so Cargo validation was used
  for the Rust batch instead.
- `cargo fmt --check` still reports pre-existing formatting drift in unrelated
  Rust files outside this batch; the touched files were formatted directly with
  `rustfmt`.

## Next Task

Push the next meaningful rhythm batch into beat-grid quality: add explicit
downbeat/bar-phase inference and expose secondary tempo candidates or ambiguity
metadata so Finch can distinguish stable quarter-note pulse from unresolved
meter/double-time cases without product-specific heuristics.
