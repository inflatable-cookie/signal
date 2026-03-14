# Rhythm Tempo Arc Downgrade Trends

Date: 2026-03-09
Owner: core-product

## Summary

Extended the tempo continuity arc decision so each retained-tempo action now
 publishes whether downgrade pressure is rising, stable, or easing across the
 current action window. Signal now exposes both a categorical downgrade trend
 and a small pressure trajectory snapshot instead of only a static downgrade
 support vector.

## Work completed

- added `TempoContinuityArcDowngradeTrend` and
  `TempoContinuityArcDowngradeTrendSupport` to
  `crates/signal-analysis-rhythm/src/lib.rs`
- extended `TempoContinuityArcDecision` so arc-level tempo continuity now
  publishes:
  - action-local severity
  - fallback action
  - downgrade rationale
  - downgrade support metrics
  - downgrade trend
  - downgrade trend support
  - action provenance
  - action expiry window
  - decision confidence
- calibrated the current trend mapping so the main tempo-state families now
  distinguish immediate pressure trajectory:
  - stable integer lock -> `Stable`
  - core-window boundary carry -> `Rising`
  - guarded refined reacquisition -> `Stable`
  - deferred clear -> `Stable`
- updated `offline_rhythm_demo` to print the trend category plus the
  `current/next/terminal` pressure snapshot inline with the tempo continuity
  decision
- expanded the rhythm tests so the trend category and pressure ordering are
  pinned in both direct tempo-state tests and the aggregate arc calibration
  test

## Validation

- `cargo test -p signal-analysis-rhythm --no-run`
- `cargo check -p signal-analysis-rhythm --example offline_rhythm_demo`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- The trend surface is intentionally local to the current action window: it
  compares immediate action pressure to the next staged continuation rather
  than trying to summarize the entire continuity lifecycle in one number.
- This batch uses the existing staged continuity transitions as the trend
  source instead of inventing a separate tempo-specific timeline model.

## Next Task

Deepen the tempo continuity arc decision with downgrade trend rationale or
 inflection metadata, so Signal can explain whether a `Rising` or `Stable`
 trend is being shaped more by upcoming revalidation loss, boundary pressure,
 or terminal clear pressure instead of only publishing the trend category and
 pressure snapshot.
