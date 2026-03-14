# Rhythm Tempo Continuity Stage Rationale Weighting

Date: 2026-03-09
Owner: core-product

## Summary

Completed the current tempo continuity surfacing line by adding stage-specific
rationale weighting to the tempo arc inflection model. Signal now quantifies
not just which stages shape the downgrade path, but which rationale dominates
within the primary and competing stages.

## Work completed

- added stage-local rationale types to
  `crates/signal-analysis-rhythm/src/lib.rs`:
  - `TempoContinuityArcDowngradeStageRationale`
  - `TempoContinuityArcDowngradeStageRationaleWeights`
  - `TempoContinuityArcDowngradeInflectionRationaleBalance`
- extended `TempoContinuityArcDowngradeInflection` so it now publishes:
  - primary-stage rationale weights and dominant rationale
  - competing-stage rationale weights and dominant rationale when a competing
    stage is present
- calibrated the current public contract so the main tempo continuity families
  now read cleanly:
  - stable integer lock -> primary `StabilityWindow`, competing `EvidenceLoss`
  - boundary-drift core-window carry -> primary `BoundaryDrift`, competing
    `EvidenceLoss`
  - guarded refined reacquisition -> primary `AmbiguityCarry`, competing
    `EvidenceLoss`
  - cleared deferred tempo -> primary `EvidenceLoss`, no competing stage
- updated `offline_rhythm_demo` to print the primary and competing rationale
  weight tuples on separate lines
- expanded the direct tempo-state tests and aggregate arc calibration test so
  the new rationale weighting is part of the Signal-owned contract

## Validation

- `cargo test -p signal-analysis-rhythm --no-run`
- `cargo check -p signal-analysis-rhythm --example offline_rhythm_demo`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- This batch is intended to round out the current tempo continuity surfacing
  work rather than start a new chain of tiny metadata additions.
- The new rationale weighting is still heuristic and synthetic-fixture
  calibrated, but it is now expressive enough that Finch should not need to
  invent its own per-stage tempo continuity explanation layer.

## Next Task

Shift the next batch back toward runtime exercise and consumer fit: run the new
tempo continuity surface through realistic offline fixtures or Finch-facing
integration code, and tune any thresholds that feel too eager or too conservative
when the full result surface is consumed outside the synthetic calibration path.
