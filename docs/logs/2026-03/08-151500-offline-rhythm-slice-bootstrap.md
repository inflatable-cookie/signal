# Offline Rhythm Slice Bootstrap

Date: 2026-03-08
Owner: core-product

## Summary

Expanded the initial Rust workspace shell into the first usable offline rhythm
analysis slice.

## Work completed

- strengthened `signal-primitives` with interleaved-buffer construction and mono
  mixdown
- added real STFT and magnitude-frame support in `signal-dsp-spectral` using
  `rustfft`
- implemented a first rhythm path in `signal-analysis-rhythm`:
  - spectral-flux onset envelope
  - autocorrelation tempo estimate
  - simple beat placement with local refinement
- added a tiny offline demo harness:
  - `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 120 --seconds 8`
- added focused tests for primitives, spectral transforms, silence handling, and
  click-track tempo detection

## Validation

- `cargo test --workspace`
- `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 120 --seconds 8`
- `git diff --check`

## Notes

The demo click track at 120 BPM was detected at approximately 119.68 BPM with
high confidence, which is a reasonable starting point for the first slice.

## Next Task

Extend the same pattern into tonal analysis by building reusable chroma
extraction on top of `signal-dsp-spectral`, then use that in a first
`signal-analysis-tonal` key-detection prototype.
