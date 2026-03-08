# Rhythm Tempo Continuity Semantics

Date: 2026-03-08
Owner: core-product

## Summary

Added a Signal-owned tempo continuity layer above the existing tempo-state
action surface so downstream callers can keep, reacquire, or clear prior tempo
state without inventing wrapper-specific policy.

## Work completed

- added `TempoContinuityAction`, `TempoContinuitySource`,
  `TempoContinuityReason`, `TempoContinuityTransition`,
  `TempoContinuityLifecycle`, and `TempoContinuityPlan` to
  `crates/signal-analysis-rhythm/src/lib.rs`
- extended `TempoStateRecommendation` with `continuity`
- calibrated continuity behavior for the existing top-level tempo states:
  - stable integer snap -> lock current tempo, then retain briefly before clear
  - stable refined tempo -> lock current tempo, then retain briefly before clear
  - core-window fallback -> retain core-window tempo, then reacquire or clear
  - unstable/deferred tempo -> clear immediately
- updated `offline_rhythm_demo` to print the current tempo continuity plan plus
  refresh and decay stages
- extended rhythm tests so tempo continuity semantics are part of the public
  contract for stable integer tempo, guarded core-window fallback, stable
  refined tempo, and deferred tempo

## Validation

- `cargo test -p signal-analysis-rhythm --no-run`
- `cargo check -p signal-analysis-rhythm --example offline_rhythm_demo`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- This keeps tempo retention and expiry logic owned by Signal instead of
  forcing Finch to build a separate tempo state machine.
- Direct Rust binary runtime remains environment-constrained through this
  execution path, so this batch was validated with compile-level Rust checks
  plus serial Effigy/CTest execution.

## Next Task

Deepen the tempo continuity surface with provenance and expiry metadata, such
 as how long retained tempo remains trustworthy during `Monitor`, when
 reacquisition should downgrade to clear, and how that behavior should vary
 across stable refined tempo, core-window carry, and deferred tempo families.
