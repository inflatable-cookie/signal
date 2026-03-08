# Rhythm Section Transition Recovery Calibration

Date: 2026-03-08
Owner: core-product

## Summary

Extended the rhythm bar-transition family from single-bar disruptions into
section-level transition scenarios so Signal can now calibrate meter recovery
after dropout, temporary meter modulation, and cadence-like bar elongation.

## Work completed

- expanded `BarTransitionVariant` with section-level cases for:
  - temporary meter modulation
  - downbeat re-entry after a dropout section
  - cadence-like elongated bar structure
- implemented those scenarios inside the shared named preset builder instead of
  adding one-off fixture tests outside the preset surface
- updated the preset expectation table so section-level transitions now sit in
  the same calibration layer as pickup, late-shift, mixed-length, fill-density,
  harmony, and dropout families
- added stronger family comparisons around transition recovery, checking that:
  - downbeat re-entry can recover four-four meter
  - temporary modulation and cadential elongation stay meter-unknown
  - disruptive transitions increase ambiguity or reduce confidence relative to
    stable re-entry and pickup cases

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo test --workspace`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- The rhythm crate now carries 26 targeted tests and now distinguishes:
  - stable bar recovery after a marked re-entry
  - accent-shift stress that preserves meter
  - section-level bar disruption that should keep meter unknown
- This batch stays inside Signal’s reusable rhythm analysis crate and tightens
  the result surface Finch can eventually depend on without product-specific
  transition heuristics.

## Next Task

Add section-transition variants that combine bar-structure change with harmonic
rhythm change or fill-density change, then tune whether meter recovery should
win or stay unknown when multiple destabilizing cues happen at once.
