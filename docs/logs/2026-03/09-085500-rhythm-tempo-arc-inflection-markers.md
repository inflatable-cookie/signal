# Rhythm Tempo Arc Inflection Markers

Date: 2026-03-09
Owner: core-product

## Summary

Extended the tempo continuity arc trend surface with explicit inflection
 markers. Signal now publishes whether downgrade pressure is being shaped by
 the immediate next staged checkpoint, the terminal clear horizon, or a flat
 no-turn window, along with the beat offset and delta magnitudes behind that
 marker.

## Work completed

- added `TempoContinuityArcDowngradeInflectionStage` and
  `TempoContinuityArcDowngradeInflection` to
  `crates/signal-analysis-rhythm/src/lib.rs`
- extended `TempoContinuityArcDecision` so arc-level tempo continuity now
  publishes:
  - downgrade trend
  - downgrade trend rationale
  - downgrade trend support
  - downgrade inflection stage
  - inflection beat offset
  - next-stage and terminal delta magnitudes
- calibrated the current inflection mapping so the main tempo-state families
  now distinguish where the pressure turn is coming from:
  - stable integer lock -> `NextStage @ 12`
  - core-window boundary carry -> `NextStage @ 8`
  - guarded refined reacquisition -> `NextStage @ 4`
  - deferred clear -> `FlatWindow @ 0`
- updated `offline_rhythm_demo` to print the inflection stage, beat offset, and
  delta pair inline with the existing trend output
- expanded the rhythm tests so the new inflection markers are pinned in both
  direct tempo-state tests and the aggregate arc calibration test

## Validation

- `cargo test -p signal-analysis-rhythm --no-run`
- `cargo check -p signal-analysis-rhythm --example offline_rhythm_demo`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- The inflection layer is intentionally narrow: it identifies where the current
  pressure trajectory turns, not the entire long-horizon continuity arc.
- This keeps the trend interpretation Signal-owned and avoids pushing “which
  stage matters most right now?” heuristics into Finch.

## Next Task

Deepen the tempo continuity arc inflection surface with secondary-stage or
 competing-stage attribution, so Signal can say when the immediate next stage
 and the terminal clear horizon are both materially shaping the downgrade path
 instead of choosing only one primary marker.
