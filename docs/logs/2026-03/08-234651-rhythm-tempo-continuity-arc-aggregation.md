# Rhythm Tempo Continuity Arc Aggregation

Date: 2026-03-08
Owner: core-product

## Summary

Added a plan-level tempo continuity arc above the existing severity, cause, and
 unresolved-span surface so Signal can publish whether a full tempo continuity
 path is recovering, stalling, or collapsing instead of leaving that
 interpretation to downstream wrappers.

## Work completed

- added `TempoContinuityArc`, `TempoContinuityArcRationale`, and
  `TempoContinuityArcSupport` to `crates/signal-analysis-rhythm/src/lib.rs`
- extended `TempoContinuityPlan` so tempo continuity now publishes:
  - top-level arc
  - dominant arc rationale
  - support scores for refresh strength, drift pressure, and instability
    pressure
- added arc aggregation above the existing transition-level tempo continuity
  lifecycle, using current history plus refresh/decay stages to classify:
  - stable integer tempo -> recovering
  - core-window fallback -> stalling
  - guarded refined reacquisition -> recovering
  - deferred tempo -> collapsing
- updated `offline_rhythm_demo` to print the plan-level tempo arc summary
- expanded the rhythm tests so the new arc surface is pinned directly in
  tempo-state policy tests and in a dedicated tempo continuity arc calibration
  test

## Validation

- `cargo test -p signal-analysis-rhythm --no-run`
- `cargo check -p signal-analysis-rhythm --example offline_rhythm_demo`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- This keeps tempo continuity trajectory semantics owned by Signal rather than
  forcing Finch to infer whether a monitored tempo path is recovering or
  collapsing.
- Direct Rust binary runtime remains constrained through the current execution
  path here, so this batch was validated with compile-level Rust checks plus
  serial Effigy/CTest execution.

## Next Task

Deepen the tempo continuity arc surface with arc-level recommendation or
 action semantics, so Signal can tell callers whether to keep tempo locked,
 monitor for recovery, or clear tempo state immediately when a path is
 recovering, stalling, or collapsing.
