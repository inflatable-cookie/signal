# Rhythm Tempo Continuity Arc Action Expiry

Date: 2026-03-09
Owner: core-product

## Summary

Extended the tempo continuity arc decision so retained-tempo actions are now
 self-contained. Signal now publishes not only the arc recommendation and
 action, but also the action provenance, the fallback action, and the beat
 window for when that action should downgrade or clear.

## Work completed

- added `TempoContinuityArcActionExpiry` to
  `crates/signal-analysis-rhythm/src/lib.rs`
- extended `TempoContinuityArcDecision` so arc-level tempo continuity now
  publishes:
  - retained-tempo action
  - fallback action
  - action provenance
  - action expiry window
  - decision confidence
- calibrated the current action-expiry mapping:
  - `LockCurrentTempo` keeps current tempo provenance and falls back to
    `ReacquireCurrentTempo`
  - `PreferCoreWindowTempo` carries core-window provenance and falls back to
    `PreservePriorTempo`
  - `ReacquireCurrentTempo` carries guarded refined provenance and falls back
    to `ClearTempo`
  - `ClearTempo` stays self-clearing with zero expiry
- updated `offline_rhythm_demo` to print fallback action plus action-level
  provenance and expiry inline with the tempo continuity decision
- expanded the rhythm tests so the new arc-level provenance, fallback, and
  expiry fields are pinned in direct tempo-state tests and the aggregate arc
  calibration test

## Validation

- `cargo test -p signal-analysis-rhythm --no-run`
- `cargo check -p signal-analysis-rhythm --example offline_rhythm_demo`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- This makes the arc layer independently consumable: callers no longer need to
  cross-reference the broader tempo continuity plan just to know how long an
  arc action should remain trusted or what it should degrade into next.
- The current calibrated tempo-state fixtures still exercise
  `LockCurrentTempo`, `PreferCoreWindowTempo`, `ReacquireCurrentTempo`, and
  `ClearTempo`; `PreservePriorTempo` remains part of the owned public action
  vocabulary for future stalling-carry scenarios.

## Next Task

Deepen the tempo continuity action surface with action-local severity and
 downgrade rationale, so arc decisions can explain whether a fallback is being
 driven by boundary drift, ambiguity carry, or repeated failed revalidation
 rather than only publishing the fallback action and expiry window.
