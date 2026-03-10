# Rhythm Tempo Arc Trend Rationale

Date: 2026-03-09
Owner: core-product

## Summary

Extended the tempo continuity arc decision so each downgrade trend now explains
 what is shaping that trend. Signal now publishes whether the trend is driven
 by boundary escalation, terminal clear pressure, flat collapse, ambiguity
 carry, revalidation decay, or stability-window carry instead of exposing only
 the trend category and trend-support snapshot.

## Work completed

- added `TempoContinuityArcDowngradeTrendRationale` to
  `crates/signal-analysis-rhythm/src/lib.rs`
- extended `TempoContinuityArcDecision` so arc-level tempo continuity now
  publishes:
  - downgrade trend
  - downgrade trend rationale
  - downgrade trend support
- calibrated the current rationale mapping so the main tempo-state families now
  distinguish why the trend looks the way it does:
  - stable integer lock -> `Stable` +
    `TerminalClearPressure`
  - core-window boundary carry -> `Rising` +
    `BoundaryEscalation`
  - guarded refined reacquisition -> `Stable` +
    `TerminalClearPressure`
  - deferred clear -> `Stable` +
    `FlatCollapse`
- updated `offline_rhythm_demo` to print the trend rationale inline with the
  existing trend category and pressure snapshot
- expanded the rhythm tests so the new trend-rationale surface is pinned in
  both direct tempo-state tests and the aggregate arc calibration test

## Validation

- `cargo test -p signal-analysis-rhythm --no-run`
- `cargo check -p signal-analysis-rhythm --example offline_rhythm_demo`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- The trend rationale stays at the same abstraction level as the trend itself:
  it explains why the pressure trajectory looks the way it does without
  forcing callers to cross-reference the lower-level action rationale or raw
  support vector.
- The current calibrated tempo-state families still do not exercise an
  explicit `Easing` path, but the public rationale vocabulary now supports it
  for future recovery-focused families.

## Next Task

Deepen the tempo continuity arc trend surface with explicit inflection timing
 or stage markers, so Signal can say whether a rising or flat trend is being
 shaped by the immediate next stage or only by the longer terminal clear
 horizon.
