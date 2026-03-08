# Rhythm Local Tempo Diagnostics

Date: 2026-03-08
Owner: core-product

## Summary

Added a public local-tempo diagnostics surface to `signal-analysis-rhythm` so
Signal can show where residual BPM variance is coming from over time instead of
only publishing a single refined BPM number.

## Work completed

- added `TempoDiagnostics` to `BeatAnalysisResult` in
  `crates/signal-analysis-rhythm/src/lib.rs`
- added per-interval and four-beat local tempo points so callers can inspect
  tempo over time rather than only the final aggregate BPM
- added summary metrics for raw interval tempo and four-beat window tempo:
  median, drift span, and mean absolute deviation
- added core-versus-boundary diagnostics for windowed tempo:
  - `core_windowed_*` summaries for the trimmed middle of the track
  - `boundary_bias_bpm` to separate startup and teardown skew from ongoing
    beat-grid drift
- updated `offline_rhythm_demo` to print the new diagnostics surface along with
  the first few interval and windowed tempo points
- added test coverage that:
  - verifies stable integer click tracks expose low local-tempo deviation while
    keeping refined median tempo near the requested BPM
  - compares local-tempo variability across stable and irregular preset
    families

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 120 --seconds 8`
- `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 90 --seconds 8`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- On the neutral 120 BPM click demo, Signal now reports:
  - final refined BPM `119.90`
  - interval median `119.88`, interval MAD `0.139`, interval span `0.863`
  - windowed median `119.90`, windowed MAD `0.117`, windowed span `0.371`
  - core windowed median `119.90`, core windowed MAD `0.133`, boundary bias
    `0.088`
- That means the remaining 120 BPM error is not mostly edge bias; the current
  beat grid still drifts gradually through the middle of the track.
- On the neutral 90 BPM click demo, Signal now reports:
  - final refined BPM `90.07`
  - interval median `90.29`, interval MAD `0.724`, interval span `5.616`
  - windowed median `90.00`, windowed MAD `0.150`, windowed span `1.199`
  - core windowed median `90.00`, core windowed MAD `0.000`, boundary bias
    `1.199`
- That means the remaining 90 BPM variation is dominated by leading and
  trailing beat-placement skew rather than drift through the stable middle.

## Next Task

Add a tempo-trend or beat-grid error decomposition surface on top of the new
local-tempo diagnostics, so Signal can distinguish gradual beat-grid drift from
boundary-placement skew and then decide whether the next BPM-accuracy step
should be denser onset timing, stronger beat-grid refinement, or confidence-
gated integer-tempo snapping.
