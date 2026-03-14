# Rhythm Multistage Density And Accent Drift

Date: 2026-03-08
Owner: core-product

## Summary

Extended the rhythm bar-transition preset family with stacked-cue multistage
re-entry cases so Signal can distinguish between recovery paths that stay in
four-four under added fill-density drift and recovery paths that should fall
back to `meter: None` when harmonic drift and accent drift destabilize the
same window.

## Work completed

- widened `BarTransitionVariant` with four new multistage recovery cases:
  - `ReentryAcceleratingHarmonyDenseFill`
  - `ReentryDeceleratingHarmonyDenseFill`
  - `ReentryAcceleratingHarmonyAccentShift`
  - `ReentryDeceleratingHarmonyAccentShift`
- added reusable dense-recovery and accent-shift recovery bar-pattern fixtures
  so the multistage transition family now covers both activity drift and
  downbeat-destabilizing accent drift
- implemented the dense-fill recovery cases as meter-supportive multistage
  re-entry paths with evolving harmonic rhythm plus clearer retained downbeats
- implemented the accent-shift recovery cases as stacked harmonic-plus-accent
  disruption paths with beat-two/beat-four pressure and late section markers so
  the expected surface is explicitly `meter: None`
- extended the named preset expectation table and transition-family calibration
  checks so the new recovery family now encodes:
  - dense multistage recovery should keep four-four
  - accent-shifted multistage recovery should remain meter-unknown
  - dense recovery can still be confidence-dominant even when ambiguity does
    not increase monotonically against simpler harmonic-drift recovery
- added a dedicated comparison test for density-versus-accent multistage drift
  so future rhythm tuning has a stable contract around when recovery should
  remain stable versus collapse to unknown meter

## Validation

- `cargo test -p signal-analysis-rhythm`
- `effigy health`
- `effigy validate`
- `cargo test --workspace`
- `git diff --check`

## Notes

- The rhythm crate now has 28 targeted tests covering multistage recovery under
  harmonic drift, density drift, and accent drift.
- This batch keeps the calibration surface inside Signal’s shared rhythm crate
  instead of forcing Finch to infer when stacked recovery cues are too unstable
  for bar-level claims.
- Repo-wide Cargo validation is green again in this workspace, so there is no
  outstanding unrelated Rust failure to carry forward from this batch.

## Next Task

Add multistage transition presets that combine re-entry recovery drift with
section-boundary harmonic resets or cadence-like re-anchors, then tune whether
meter confidence should recover gradually or snap back quickly when bar
structure becomes stable again after a destabilized recovery window.
