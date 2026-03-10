# Rhythm Tempo Arc Action Rationale

Date: 2026-03-09
Owner: core-product

## Summary

Extended the tempo continuity arc decision so each retained-tempo action now
 publishes its own action-local severity and downgrade rationale. Signal can
 now explain whether an arc action is confirmed, guarded, fragile, or cleared,
 and whether it will degrade because of boundary drift, ambiguity carry,
 evidence loss, or the end of a stable revalidation window.

## Work completed

- added `TempoContinuityArcDowngradeRationale` to
  `crates/signal-analysis-rhythm/src/lib.rs`
- extended `TempoContinuityArcDecision` so arc-level tempo continuity now
  publishes:
  - action-local severity
  - fallback action
  - downgrade rationale
  - action provenance
  - action expiry window
  - decision confidence
- calibrated the current rationale mapping:
  - `LockCurrentTempo` -> `Confirmed` +
    `StabilityWindowEnd`
  - `PreferCoreWindowTempo` -> `Guarded` +
    `BoundaryDrift`
  - `ReacquireCurrentTempo` -> `Fragile` +
    `AmbiguityCarry`
  - `ClearTempo` -> `Cleared` +
    `EvidenceLoss`
- updated `offline_rhythm_demo` to print action severity and downgrade
  rationale inline with the tempo continuity decision
- expanded the rhythm tests so the new decision-level severity and rationale
  fields are pinned in direct tempo-state tests and the aggregate arc
  calibration test

## Validation

- `cargo test -p signal-analysis-rhythm --no-run`
- `cargo check -p signal-analysis-rhythm --example offline_rhythm_demo`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- This keeps the explanation for tempo downgrade at the same abstraction level
  as the action itself instead of forcing callers to infer it from lower-level
  trigger and cause fields.
- `RepeatedFailedRevalidation` is now part of the public downgrade vocabulary
  even though the current calibrated tempo-state fixtures still land on
  stability-window, boundary-drift, ambiguity-carry, and evidence-loss paths.

## Next Task

Deepen the tempo continuity arc decision with downgrade support metrics, so
 each fallback says not just why it is degrading but how strongly boundary
 drift, ambiguity carry, or repeated failed revalidation are contributing to
 that downgrade.
