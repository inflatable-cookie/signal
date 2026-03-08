# Rhythm Tempo Continuity Provenance And Expiry

Date: 2026-03-08
Owner: core-product

## Summary

Extended the Signal-owned tempo continuity surface with structured provenance
 and expiry metadata so callers can tell where retained tempo came from and how
 long it should remain trustworthy before degrading or clearing.

## Work completed

- added `TempoContinuityProvenance` and `TempoContinuityExpiry` to
  `crates/signal-analysis-rhythm/src/lib.rs`
- extended `TempoContinuityPlan` and `TempoContinuityTransition` so tempo
  continuity now publishes provenance alongside action/source/reason
- derived an expiry policy for each tempo continuity plan:
  - guaranteed trust window
  - downgrade point
  - clear point
  - failed revalidation budget
- calibrated provenance/expiry behavior for the current tempo-state families:
  - stable integer snap -> `IntegerSnap` provenance with longer trust window
  - stable refined tempo -> `StableRefinedEstimate` provenance with the same
    long hold/decay shape
  - guarded refined tempo -> `GuardedRefinedEstimate` provenance with short
    reacquire-first expiry
  - core-window fallback -> `CoreWindowEstimate` provenance with explicit
    downgrade to prior-tempo carry
  - deferred tempo -> `NoTempo` provenance with immediate clear
- updated `offline_rhythm_demo` to print plan provenance, expiry windows, and
  per-stage transition provenance
- expanded the rhythm tests so stable integer, stable refined, guarded refined,
  core-window fallback, and deferred tempo all pin the new provenance/expiry
  contract

## Validation

- `cargo test -p signal-analysis-rhythm --no-run`
- `cargo check -p signal-analysis-rhythm --example offline_rhythm_demo`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- This keeps tempo carry and expiry semantics owned by Signal instead of
  pushing retained-tempo policy into Finch.
- Direct Rust binary runtime remains constrained through the current execution
  path here, so this batch was validated with compile-level Rust checks plus
  serial Effigy/CTest execution.

## Next Task

Deepen the tempo continuity surface with stage-level severity and refresh
 confidence metadata, so Signal can distinguish strong retained tempo from
 fragile carry during `Monitor` and expose when reacquisition is reinforcing
 versus merely delaying a clear.
