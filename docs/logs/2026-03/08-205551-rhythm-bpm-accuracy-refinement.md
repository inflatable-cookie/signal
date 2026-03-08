# Rhythm BPM Accuracy Refinement

Date: 2026-03-08
Owner: core-product

## Summary

Investigated the consistent sub-BPM tempo error in `signal-analysis-rhythm`
 and tightened the public BPM path. The main source of the drift was not random
 variance: tempo was being converted directly from an integer onset-envelope lag
 at the STFT hop rate, which quantized integer BPM material onto coarse tempo
 bins such as `46.875 -> 47 -> 119.68 BPM` at 48 kHz with a 512-sample hop.

## Work completed

- traced the root cause to the current `estimate_tempo(...)` path in
  `crates/signal-analysis-rhythm/src/lib.rs`
  - onset rate is `sample_rate / hop_size`
  - tempo candidates were scored only on integer lags
  - BPM was published as `60 * onset_rate / lag`
- added sub-frame lag refinement around the autocorrelation peak via local
  parabolic interpolation
- kept integer lag frames for beat tracking, but stopped publishing BPM from the
  raw integer lag alone
- added sub-frame beat-peak refinement on the recovered beat grid
- added a consistency-weighted beat-grid BPM refinement pass so recovered beats
  can tighten the final BPM without blindly overruling the autocorrelation lag
  on noisier grids
- updated the public primary tempo candidate to match the final refined BPM so
  the result surface is internally consistent
- added an explicit integer-tempo regression test that requires sub-tenth-BPM
  accuracy on simple 120 BPM and 90 BPM click tracks

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 120 --seconds 8`
- `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 90 --seconds 8`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- After this batch, the offline demo moved from roughly `119.68 BPM` on a 120
  BPM synthetic click track to roughly `119.90 BPM`, and from a visibly
  half-BPM-ish bias on 90 BPM material to roughly `90.07 BPM`.
- The remaining error is much smaller, but not fully eliminated. The current
  path is still bounded by the onset-envelope resolution and by the fact that
  meter/downbeat logic still consumes integer beat frames.
- One environment note remains: the non-PTY demo invocation can still appear to
  stall before producing output here, while the same command under a TTY
  completes normally.

## Next Task

Add an explicit local-tempo or beat-interval diagnostics surface on top of the
new refined BPM path, so Signal can show where residual drift is coming from
over time, then calibrate that against constant-tempo integer tracks, swung
material, and section-transition fixtures before tightening tempo accuracy
further.
