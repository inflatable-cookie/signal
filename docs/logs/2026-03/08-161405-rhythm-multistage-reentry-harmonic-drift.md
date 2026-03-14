# Rhythm Multistage Reentry Harmonic Drift

Date: 2026-03-08
Owner: core-product

## Summary

Extended the rhythm bar-transition preset family with multistage re-entry
recovery cases so Signal can calibrate meter recovery when harmonic rhythm
accelerates or decelerates across successive post-dropout sections.

## Work completed

- widened `BarTransitionVariant` with:
  - `ReentryAcceleratingHarmony`
  - `ReentryDeceleratingHarmony`
- refactored the repeated re-entry fixture setup into
  `build_reentry_transition_fixture(...)` so the common intro and dropout
  bridge stay consistent across the whole re-entry family
- rewired the existing re-entry, harmonic-shift, and dense-fill transition
  presets through that shared helper to keep the preset family diff-friendly
- implemented the new multistage variants as two-stage recovery paths:
  - accelerating harmonic rhythm after re-entry
  - decelerating harmonic rhythm after re-entry
- extended the named preset expectation table and transition calibration tests
  so the new variants must:
  - retain four-four meter
  - preserve credible confidence
  - keep tempo ambiguity in line with the base re-entry family
- added a dedicated recovery-stage drift test to compare the accelerating and
  decelerating post-re-entry paths directly

## Validation

- `cargo test -p signal-analysis-rhythm`
- `effigy health`
- `effigy validate`
- `cargo test --workspace`:
  failed outside this batch in `signal-plugin/src/lib.rs` because tests there
  reference `BlockProcessResult` and `CompletionSlot` without importing them
- `git diff --check`

## Notes

- The rhythm crate now has 27 targeted tests covering multistage recovery drift
  in addition to the earlier preset-family calibration surface.
- `effigy validate` still runs the heavyweight CMake/C++ path here,
  but it completed successfully once the workspace lock from the concurrent
  `effigy health` invocation cleared.
- The new transition surface keeps this work inside Signal’s reusable rhythm
  analysis boundary and does not push any recovery heuristics into Finch.

## Next Task

Add multistage transition presets that combine re-entry recovery drift with
fill-density change or explicit sub-band accent change, then tune whether meter
recovery should stay stable or fall back to `meter: None` when harmonic drift
and accent drift destabilize the same recovery window.
