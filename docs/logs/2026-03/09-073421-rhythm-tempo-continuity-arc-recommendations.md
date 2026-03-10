# Rhythm Tempo Continuity Arc Recommendations

Date: 2026-03-09
Owner: core-product

## Summary

Added an arc-level tempo continuity recommendation layer above the existing arc
 surface so Signal can tell downstream callers whether to keep tempo locked,
 monitor for recovery, or clear tempo state immediately.

## Work completed

- added `TempoContinuityArcRecommendation` and
  `TempoContinuityArcDecision` to
  `crates/signal-analysis-rhythm/src/lib.rs`
- extended `TempoContinuityPlan` so tempo continuity now publishes a
  recommendation plus confidence alongside the existing arc, rationale, and
  support fields
- added recommendation mapping above the current arc surface:
  - recovering + confirmed + reinforcing -> `KeepLock`
  - recovering but not yet stable enough -> `MonitorRecovery`
  - stalling -> `MonitorRecovery`
  - collapsing -> `Clear`
- updated `offline_rhythm_demo` to print the arc-level recommendation and its
  confidence inline with the tempo continuity summary
- expanded the rhythm tests so stable integer tempo now pins `KeepLock`,
  guarded/core-window paths pin `MonitorRecovery`, and deferred tempo pins
  `Clear`

## Validation

- `cargo test -p signal-analysis-rhythm --no-run`
- `cargo check -p signal-analysis-rhythm --example offline_rhythm_demo`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- This keeps tempo continuity action semantics owned by Signal instead of
  forcing Finch to convert arc categories into product behavior.
- Direct Rust binary runtime remains constrained through the current execution
  path here, so this batch was validated with compile-level Rust checks plus
  serial Effigy/CTest execution.

## Next Task

Deepen the tempo continuity recommendation surface with retained-tempo
 continuity actions at the arc layer, such as whether `MonitorRecovery` should
 preserve prior tempo, prefer core-window tempo, or reacquire from current
 refined tempo, and calibrate that behavior across recovering, stalling, and
 collapsing arc families.
