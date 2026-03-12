# 2026-03-11 18:15:30 GMT - g02.001 shared analysis substrate opening tranche

## Summary

Opened `g02.001` with the first real shared analysis substrate rather than
adding more analyzer-local helpers. This tranche adds deterministic offline and
chunked mono resampling, a shared analysis input-preparation contract for mono
reduction plus center trimming plus optional target-rate conversion, and a
streaming STFT path that matches the existing offline spectral framing.

The result is that rhythm, tonal, and character analyzers now consume the same
preparation seam, while `signal-dsp-spectral` exposes a reusable chunked STFT
surface that deeper `g02` analyzers can build on without re-implementing frame
math.

## What changed

- added `crates/signal-dsp-resample` with:
  - `ResampleConfig`
  - `ResampleQuality`
  - `StreamingResampler`
  - `resample_mono(...)`
- extended `crates/signal-analysis` with:
  - `AnalysisChannelPolicy`
  - `AnalysisInputConfig`
  - `PreparedAnalysisBuffer`
  - `prepare_audio_analysis(...)`
  - `prepare_mono_analysis(...)`
- extended `crates/signal-dsp-spectral` with:
  - `StreamingStft`
  - offline/streaming frame equivalence coverage
  - explicit final zero-padded flush semantics for partial trailing hops
- moved analyzer entry preparation in:
  - `crates/signal-analysis-rhythm`
  - `crates/signal-analysis-tonal`
  - `crates/signal-analysis-character`
  onto the shared preparation contract so they no longer open-code center
  trimming and mono staging

## Validation

- `cargo fmt`
- `cargo test -p signal-dsp-resample -p signal-analysis -p signal-dsp-spectral -p signal-analysis-tonal -p signal-analysis-character -p signal-analysis-rhythm`

## Follow-on

The next `g02.001` batch should migrate loudness and the remaining analyzer
surfaces onto the same preparation boundary, then decide which profiles should
freeze explicit analysis sample rates instead of inheriting the source rate.
