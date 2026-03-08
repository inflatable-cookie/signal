# Rhythm Tempo Continuity Severity And Refresh Strength

Date: 2026-03-08
Owner: core-product

## Summary

Extended the Signal-owned tempo continuity surface with stage-level severity,
 lifecycle history, and refresh-strength metadata so downstream callers can
 distinguish strong retained tempo from fragile carry and see when a refresh is
 reinforcing versus merely delaying a clear.

## Work completed

- added `TempoContinuitySeverity` and `TempoContinuityHistory` to
  `crates/signal-analysis-rhythm/src/lib.rs`
- extended `TempoContinuityPlan` and `TempoContinuityTransition` so tempo
  continuity now publishes:
  - severity
  - lifecycle history
  - calibrated `refresh_strength`
- mapped the existing tempo continuity branches into explicit lifecycle
  semantics:
  - stable integer snap -> confirmed, reinforcing continuity
  - stable refined tempo -> confirmed, reinforcing continuity
  - core-window carry -> guarded, preserving continuity
  - guarded refined tempo -> fragile, preserving continuity with a reinforcing
    refresh stage
  - deferred tempo -> cleared, degrading continuity
- updated `offline_rhythm_demo` to print severity, history, and refresh
  strength for the current plan and each tempo continuity transition
- expanded rhythm tests so the new severity/history/refresh-strength surface is
  pinned across stable integer, core-window fallback, guarded refined, and
  deferred tempo families

## Validation

- `cargo test -p signal-analysis-rhythm --no-run`
- `cargo check -p signal-analysis-rhythm --example offline_rhythm_demo`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- This keeps tempo continuity interpretation owned by Signal instead of forcing
  Finch to infer whether a monitored tempo is still trustworthy.
- Direct Rust binary runtime remains constrained through the current execution
  path here, so this batch was validated with compile-level Rust checks plus
  serial Effigy/CTest execution.

## Next Task

Deepen the tempo continuity surface with explicit cause and unresolved-span
 metadata, so Signal can explain whether tempo continuity is weakening because
 of boundary drift, tempo ambiguity, or prior-state carry and publish how many
 beats or failed revalidations remain unresolved before a clear becomes likely.
