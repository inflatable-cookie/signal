# 2026-03-11 18:12:45 GMT - g02.001 loudness prep and rate policy tranche

## Summary

Advanced `g02.001` by moving loudness onto the same shared preparation
boundary as the other analyzers and freezing the first explicit profile-level
analysis-rate contract where the substrate actually matters: loudness now
analyzes at 48 kHz even when the source material arrives at another rate.

This removes the remaining analyzer-local mono/truncation staging in the
current analysis crates and turns non-48k loudness handling into an explicit
resample-then-analyze path instead of a silent confidence downgrade.

## What changed

- updated `crates/signal-analysis-loudness/src/lib.rs` to use:
  - `prepare_mono_analysis(...)`
  - `prepare_audio_analysis(...)`
  - a shared `AnalysisInputConfig` path for center trimming and mono staging
- added `analysis_sample_rate: SampleRate` to `LoudnessMeterConfig`
  - default / low / medium / high profiles now freeze loudness analysis at
    48 kHz
- kept loudness metrics on the prepared analysis stream so non-48k inputs are
  resampled into the supported weighting domain before LUFS/LRA/true-peak
  estimation
- expanded loudness coverage to pin:
  - non-48k input no longer drifts materially from the 48k path for the same
    simple fixture
  - stereo input goes through the shared preparation boundary consistently

## Validation

- `cargo fmt`
- `cargo test -p signal-analysis-loudness -p signal-analysis`

## Follow-on

The next `g02.001` batch should freeze explicit analysis-rate defaults for the
remaining rhythm, tonal, and descriptor-oriented profiles, then decide whether
any lingering window/rate knobs should move into the shared substrate contract.
