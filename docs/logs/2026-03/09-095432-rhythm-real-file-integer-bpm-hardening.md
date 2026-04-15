# Rhythm Real-File Integer BPM Hardening

Date: 2026-03-09
Owner: core-product

## Summary

Hardened Signal's tempo interpretation against a real stable master where BPM was
landing near, but not on, the known integer tempo. The main issue was not the
core beat grid; it was the interpretation layer overreacting to localized tail
outliers and refusing to snap a stable near-integer result.

## Work completed

- added a real-file probe example at
  `crates/signal-analysis-rhythm/examples/file_rhythm_probe.rs` so Signal can be
  exercised directly against WAV fixtures outside synthetic presets
- added `hound` as a dev dependency for that probe in
  `crates/signal-analysis-rhythm/Cargo.toml`
- made `boundary_bias_bpm` in `analyze_local_tempo(...)` robust by summarizing
  multiple edge windows with a median instead of using a single max edge-window
  outlier
- added long-form boundary locality discounting in `interpret_tempo(...)` so
  start/end noise on long tracks does not dominate whole-track tempo
  interpretation
- kept low-snap behavior conservative for already-exact refined tempos by only
  allowing the tiny snap path for meaningful near-integer offsets
- aligned `tempo_state_recommendation(...)` with the stronger integer-anchor
  logic so stable snapped tempos can publish `Lock` instead of collapsing to
  defer on octave-style ambiguity
- added or updated regression coverage around:
  - stable near-integer master-like snapping
  - long-form boundary pressure localization
  - real-analysis tempo consumption behavior after the interpretation changes

## Real-file result

Test file:
`~/Library/CloudStorage/Dropbox/Music/Projects/Garamond/1. 086/Output/1. 086 - v1.5 - Master Stream.wav`

Before this batch:

- `bpm=127.97321`
- `tempo_interpretation=UseCoreWindow/StableCoreWindow`
- `tempo_state=Monitor/CoreWindowFallback`
- `boundary_pressure=1.000`
- `boundary_bias_bpm=11.33755`

After this batch:

- `bpm=128.00000`
- `tempo_interpretation=SnapInteger/NearIntegerPulse`
- `tempo_state=Lock/StableIntegerTempo`
- `boundary_pressure=0.229`
- `boundary_bias_bpm=0.70492`

The key diagnostic finding was that the last few beat intervals were noisy, but
the core local tempo windows stayed tightly clustered around 128 BPM. The old
edge-max boundary metric was turning that localized tail noise into a false
whole-track instability signal.

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo check -p signal-analysis-rhythm --example file_rhythm_probe`
- `cargo run -p signal-analysis-rhythm --example file_rhythm_probe -- '~/Library/CloudStorage/Dropbox/Music/Projects/Garamond/1. 086/Output/1. 086 - v1.5 - Master Stream.wav'`

## Notes

- Several older synthetic tempo calibration tests were updated to match the new
  integer-snap and continuity semantics. The real-file fix is deliberate, but
  some older preset contracts were previously too tightly coupled to the harsher
  boundary interpretation behavior.

## Next Task

Run the same real-file probe path against a small family of known integer-tempo
masters across different BPM ranges, then calibrate whether the robust boundary
handling should also trim or downweight terminal beat outliers earlier in the
analysis pipeline instead of only correcting them at interpretation time.
