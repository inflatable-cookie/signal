# Rhythm Tempo Continuity Arc Actions

Date: 2026-03-09
Owner: core-product

## Summary

Extended the arc-level tempo continuity recommendation surface with explicit
 retained-tempo actions so Signal now says not only whether callers should keep
 lock, monitor, or clear, but also which tempo source they should keep using.

## Work completed

- added `TempoContinuityArcAction` to
  `crates/signal-analysis-rhythm/src/lib.rs`
- extended `TempoContinuityArcDecision` so tempo continuity now publishes:
  - high-level recommendation
  - retained-tempo action
  - decision confidence
- calibrated the current arc-level action mapping:
  - stable recovering tempo -> `KeepLock` + `LockCurrentTempo`
  - stalling boundary/core-window tempo -> `MonitorRecovery` +
    `PreferCoreWindowTempo`
  - recovering guarded refined tempo -> `MonitorRecovery` +
    `ReacquireCurrentTempo`
  - collapsing tempo -> `Clear` + `ClearTempo`
- updated `offline_rhythm_demo` to print the new arc action inline with the
  tempo continuity decision
- expanded the rhythm tests so the arc-level action surface is pinned in both
  direct tempo-state tests and the aggregate arc calibration test

## Validation

- `cargo test -p signal-analysis-rhythm --no-run`
- `cargo check -p signal-analysis-rhythm --example offline_rhythm_demo`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- This keeps retained-tempo behavior owned by Signal rather than pushing the
  final “which tempo should I trust right now?” policy into Finch.
- `PreservePriorTempo` is now part of the Signal-owned action vocabulary even
  though the current calibrated preset families still map primarily to
  `LockCurrentTempo`, `PreferCoreWindowTempo`, `ReacquireCurrentTempo`, and
  `ClearTempo`.
- Direct Rust binary runtime remains constrained through the current execution
  path here, so this batch was validated with compile-level Rust checks plus
  serial Effigy/CTest execution.

## Next Task

Deepen the tempo continuity action surface with action provenance and expiry at
 the arc layer, such as how long a `PreferCoreWindowTempo` or
 `ReacquireCurrentTempo` action should remain in effect before degrading to
 clear, and calibrate that behavior across recovering, stalling, and
 collapsing arc families.
