# Rhythm Tempo Trend And Grid Error Decomposition

Date: 2026-03-08
Owner: core-product

## Summary

Extended the new local-tempo diagnostics surface in `signal-analysis-rhythm`
 with explicit trend and beat-grid error decomposition so Signal can distinguish
 gradual tempo drift from boundary-placement skew and local beat-grid wobble.

## Work completed

- added `TempoTrendDiagnostics` to the public tempo diagnostics surface in
  `crates/signal-analysis-rhythm/src/lib.rs`
  - publishes trend direction
  - publishes fitted start and end BPM
  - publishes total drift in BPM
  - publishes slope per beat
  - publishes fit error as BPM mean absolute deviation
- added `BeatGridErrorDiagnostics` and `BeatGridResidualPoint` to the public
  tempo diagnostics surface
  - fits an ideal beat grid to recovered beat positions
  - publishes per-beat fitted residuals in milliseconds
  - publishes anchored drift in milliseconds relative to the initial beat and
    median beat interval
  - publishes edge-versus-core residual summaries
- kept the new decomposition local to the existing tempo diagnostics path rather
  than widening unrelated rhythm or meter surfaces
- updated `offline_rhythm_demo` to print:
  - tempo trend summary
  - beat-grid error summary
  - first few beat-grid residual points
- expanded tests so the public surface now verifies:
  - stable integer click tracks expose a `Stable` trend with bounded drift and
    low beat-grid residual error
  - slower click-track material exposes stronger edge residual pressure than
    stable core pressure
  - irregular section fixtures produce at least as much fitted trend error as
    the neutral stable fixture

## Validation

- `cargo test -p signal-analysis-rhythm`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- The new tempo surface now distinguishes three different tempo-accuracy
  failure shapes:
  - local tempo spread over time
  - fitted trend drift across the beat grid
  - beat-grid residual error in milliseconds
- That gives Signal enough structure to answer whether a tempo miss is caused
  by slow mid-track drift, boundary-placement skew, or irregular local beat
  placement without pushing that interpretation into Finch.
- One runtime note remains unresolved in this environment: the
  `offline_rhythm_demo` example no longer returned normally after this batch,
  even though crate tests and serial Effigy validation passed. I did not count
  the demo as a passing validation signal for this batch.

## Next Task

Add confidence-gated tempo interpretation on top of the new trend and grid
 error surface, such as whether a caller should trust the refined BPM directly,
 favor a stable core-window estimate, or snap toward an integer tempo when the
 residual and trend diagnostics indicate a clean near-integer pulse.
