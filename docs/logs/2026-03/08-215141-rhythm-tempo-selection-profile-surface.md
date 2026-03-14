# Rhythm Tempo Selection Profile Surface

Date: 2026-03-08
Owner: core-product

## Summary

Deepened the new Signal-owned tempo interpretation layer so the recommendation
 surface now publishes the actual selection profile behind each decision instead
 of only the final action label.

## Work completed

- added `TempoInterpretationProfile` to the public rhythm tempo surface in
  `crates/signal-analysis-rhythm/src/lib.rs`
- the profile now publishes:
  - refined BPM
  - core-window BPM
  - nearest integer BPM
  - snap error in BPM
  - aggregate stability score
  - boundary edge-gap in milliseconds
- threaded the profile through the existing tempo interpretation path so every
  recommendation now carries both the selected BPM and the full decision basis
- updated `offline_rhythm_demo` to print the new profile values alongside the
  existing interpretation and support summaries
- expanded the interpretation contract in the rhythm tests so the intended
  public behavior is now explicit at the surface level:
  - clean stable integer click material should expose small snap error and high
    stability before recommending `SnapInteger`
  - edge-skewed slower click material should expose non-zero boundary edge-gap
    before recommending `UseCoreWindow`
  - stable non-integer groove material should prefer `UseRefined`
  - ambiguous subdivision material should keep low enough stability to justify
    `Defer`

## Validation

- `cargo test -p signal-analysis-rhythm --no-run`
- `cargo check -p signal-analysis-rhythm --example offline_rhythm_demo`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- This batch does not change the recommendation categories themselves; it makes
  them more inspectable and easier to calibrate once direct Rust runtime
  execution is available again.
- The environment-level startup issue remains: direct Rust and C binaries still
  do not start cleanly through the current execution path here, while the repo's
  Effigy/CTest validation path continues to execute binaries normally.

## Next Task

Use the new selection profile to tune the recommendation thresholds themselves:
 decide when clean near-integer material should keep `UseRefined` instead of
 `SnapInteger`, and when edge-skewed material should fall back from
 `UseCoreWindow` to `Defer` as stability and boundary gap worsen.
