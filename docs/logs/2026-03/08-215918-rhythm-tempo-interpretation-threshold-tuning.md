# Rhythm Tempo Interpretation Threshold Tuning

Date: 2026-03-08
Owner: core-product

## Summary

Tuned the new tempo interpretation policy so Signal is less eager to snap
 already-good near-integer tempos and more willing to defer when edge pressure
 overwhelms tempo stability.

## Work completed

- adjusted `interpret_tempo(...)` in `crates/signal-analysis-rhythm/src/lib.rs`
  so `SnapInteger` now requires a meaningful snap benefit instead of snapping
  whenever a stable tempo merely happens to be near an integer
- added a destabilized-edge-pressure path so heavy boundary skew plus weakened
  stability can fall back to `Defer` instead of overclaiming `UseCoreWindow`
- kept the public recommendation categories unchanged
- added pure interpretation tests that exercise the recommendation policy
  directly from synthetic diagnostics rather than relying on full tracker
  runtime execution
- expanded the calibration surface so the intended policy is now explicit:
  - tiny snap error should keep `UseRefined`
  - destabilized edge-heavy tempo should `Defer`
  - existing runtime-oriented presets still document the expected categories for
    integer snap, core-window fallback, stable refined tempo, and defer

## Validation

- `cargo test -p signal-analysis-rhythm --no-run`
- `cargo check -p signal-analysis-rhythm --example offline_rhythm_demo`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- This batch intentionally focused on recommendation policy calibration rather
  than adding more diagnostics fields.
- The direct-runtime limitation remains unchanged in this environment: direct
  Rust and C binaries still do not start cleanly through the command-execution
  path here, while the repo-owned Effigy/CTest route continues to execute
  binaries normally.

## Next Task

Add a public tempo-state recommendation layer above the tuned interpretation
 policy, such as whether callers should lock, monitor, or defer tempo-dependent
 behavior, then calibrate that action surface against the current
 `UseRefined`/`UseCoreWindow`/`SnapInteger`/`Defer` categories without pushing
 wrapper heuristics into Finch.
