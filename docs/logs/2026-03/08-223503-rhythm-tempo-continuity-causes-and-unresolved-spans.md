# Rhythm Tempo Continuity Causes And Unresolved Spans

Date: 2026-03-08
Owner: core-product

## Summary

Extended the Signal-owned tempo continuity surface with explicit trigger,
 cause-stack, and unresolved-span metadata so downstream callers can see why
 tempo continuity is weakening and how much unresolved carry remains before a
 clear becomes likely.

## Work completed

- added `TempoContinuityTrigger`, `TempoContinuityUnresolvedSpan`,
  `TempoContinuityCause`, and `TempoContinuityCauseStack` to
  `crates/signal-analysis-rhythm/src/lib.rs`
- extended `TempoContinuityPlan` and `TempoContinuityTransition` so tempo
  continuity now publishes:
  - lifecycle trigger
  - unresolved beats
  - failed revalidations
  - dominant cause plus secondary causes
- calibrated the existing tempo continuity branches into explicit explanations:
  - stable integer and stable refined tempo -> `StableRevalidation` with
    `StableTempoEvidence`
  - core-window fallback -> `BoundaryDrift` with `CoreWindowCarry`
  - guarded refined tempo -> `AmbiguityCarry`
  - deferred tempo -> `EvidenceLoss` with tempo-ambiguity carry still visible
- fed the new trigger/cause/unresolved surface into continuity history and
  refresh-strength scoring so preserving vs degrading behavior now reflects the
  underlying explanation instead of only the action enum
- updated `offline_rhythm_demo` to print plan-level tempo continuity causes
  plus unresolved-span metadata
- expanded rhythm tests to pin cause and unresolved-span behavior across stable
  integer, core-window fallback, guarded refined, and deferred tempo families

## Validation

- `cargo test -p signal-analysis-rhythm --no-run`
- `cargo check -p signal-analysis-rhythm --example offline_rhythm_demo`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- This keeps tempo continuity diagnosis owned by Signal instead of forcing
  Finch to infer whether tempo drift is mostly boundary-driven, ambiguity-
  driven, or prior-state carry.
- Direct Rust binary runtime remains constrained through the current execution
  path here, so this batch was validated with compile-level Rust checks plus
  serial Effigy/CTest execution.

## Next Task

Deepen the tempo continuity surface with arc-level aggregation above the new
 cause and unresolved metadata, so Signal can publish whether a full tempo
 continuity path is recovering, stalling, or collapsing across multiple
 stages instead of leaving that interpretation to downstream wrappers.
