# Rhythm Tempo Arc Downgrade Support

Date: 2026-03-09
Owner: core-product

## Summary

Extended the tempo continuity arc decision so each retained-tempo action now
 publishes support metrics for why it is degrading. Signal can now quantify
 whether a fallback is driven primarily by stability-window expiry, boundary
 drift, ambiguity carry, failed revalidation pressure, or evidence loss.

## Work completed

- added `TempoContinuityArcDowngradeSupport` to
  `crates/signal-analysis-rhythm/src/lib.rs`
- extended `TempoContinuityArcDecision` so arc-level tempo continuity now
  publishes:
  - action-local severity
  - fallback action
  - downgrade rationale
  - downgrade support metrics
  - action provenance
  - action expiry window
  - decision confidence
- calibrated the current support mapping so the main tempo-state families have
  distinct dominant downgrade drivers:
  - `LockCurrentTempo` is dominated by
    `stability_window_pressure`
  - `PreferCoreWindowTempo` is dominated by
    `boundary_drift_pressure`
  - `ReacquireCurrentTempo` is dominated by
    `ambiguity_pressure`
  - `ClearTempo` is dominated by
    `evidence_loss_pressure`
- updated `offline_rhythm_demo` to print the downgrade support vector inline
  with the tempo continuity decision
- expanded the rhythm tests so the dominant downgrade driver is pinned in both
  direct tempo-state tests and the aggregate arc calibration test

## Validation

- `cargo test -p signal-analysis-rhythm --no-run`
- `cargo check -p signal-analysis-rhythm --example offline_rhythm_demo`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- This keeps the downgrade explanation at the same abstraction level as the
  action itself. Callers can now consume one arc-level struct instead of
  re-deriving pressure from lower-level trigger and cause fields.
- The first compile pass in this batch failed because the new support mapping
  referenced the continuity cause stack without threading it into
  `continuity_arc_decision(...)`; that was fixed before the final validation
  round.

## Next Task

Deepen the tempo continuity arc decision with downgrade trend or decay
 semantics, so Signal can show whether downgrade pressure is rising, stable,
 or easing across the action window rather than only publishing a static
 support snapshot.
