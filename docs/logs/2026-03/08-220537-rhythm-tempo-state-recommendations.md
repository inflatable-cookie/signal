# Rhythm Tempo State Recommendations

Date: 2026-03-08
Owner: core-product

## Summary

Added a Signal-owned tempo-state action layer above the tuned tempo
 interpretation policy so downstream callers can consume direct tempo behavior
 guidance instead of mapping `UseRefined`, `UseCoreWindow`, `SnapInteger`, and
 `Defer` into their own ad hoc state machine.

## Work completed

- added `TempoStateAction`, `TempoStateReason`, and `TempoStateRecommendation`
  to `crates/signal-analysis-rhythm/src/lib.rs`
- extended `BeatAnalysisResult` with `tempo_state`
- added `tempo_state_recommendation(...)` so Signal now maps tempo
  interpretation into caller-facing actions:
  - `Lock`
  - `Monitor`
  - `Defer`
- calibrated the intended policy:
  - stable integer snap -> `Lock`
  - stable refined tempo -> `Lock`
  - core-window fallback under edge pressure -> `Monitor`
  - unstable or deferred tempo -> `Defer`
- updated `offline_rhythm_demo` to print the top-level tempo state
- expanded the rhythm test surface with pure tempo-state policy tests so the new
  action layer is calibratable even while direct runtime execution remains
  blocked in this environment

## Validation

- `cargo test -p signal-analysis-rhythm --no-run`
- `cargo check -p signal-analysis-rhythm --example offline_rhythm_demo`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- This batch keeps tempo-state logic owned by Signal rather than forcing Finch
  to reinterpret lower-level diagnostics into product behavior.
- The environment-level startup limitation remains unchanged: direct Rust and C
  binaries still do not start cleanly through the current execution path here,
  while the repo-owned Effigy/CTest path continues to execute binaries
  normally.

## Next Task

Deepen the new tempo-state surface with retained-tempo continuity semantics,
 such as whether callers should keep a prior locked tempo during `Monitor`,
 reacquire it, or clear it after prolonged instability, then calibrate that
 continuity behavior against the current refined/core-window/snap/defer tempo
 categories without pushing wrapper policy into Finch.
