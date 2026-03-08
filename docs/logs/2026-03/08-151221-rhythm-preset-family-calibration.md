# Rhythm Preset Family Calibration

Date: 2026-03-08
Owner: core-product

## Summary

Promoted the rhythm fixture work into named reusable preset families so the
crate can validate whole scenario classes instead of ad hoc synthetic cases,
then added preset-surface and preset-family calibration tests on top of that
layer.

## Work completed

- introduced named rhythm presets for:
  - neutral click baseline
  - structured four-four groove
  - subdivided ambiguous pulse
  - weak-backbeat groove
  - section-transition groove
  - fill-transition groove
  - dropout-heavy unknown-meter case
- added preset rendering and analysis helpers so tests can assert against
  scenario families without rebuilding fixtures inline
- added shared assertion helpers for BPM and meter expectations to keep the
  preset tests diff-friendly and consistent
- refactored transition-oriented tests to use the named preset layer where it
  provides the same coverage with less duplication
- added preset-family tests covering:
  - expected surface shape for each named preset
  - ambiguity calibration between stable and subdivided presets
  - meter-presence contrast between structured/transition presets and
    neutral/dropout presets

## Validation

- `cargo test -p signal-analysis-rhythm`
- `effigy validate --repo .`
- `cargo test --workspace`
- `git diff --check`

## Notes

- `effigy validate --repo .` completed successfully in this batch, but it still
  drives the heavyweight CMake/C++ path rather than the Rust workspace only.
- The rhythm crate now carries 22 targeted tests and has a cleaner reusable
  calibration surface for future confidence and ambiguity tuning.
- The preset-family comparisons intentionally avoid brittle total orderings on
  confidence values and instead check the scenario relationships the current
  analysis surface actually guarantees.

## Next Task

Extend the named preset layer with groove-dropout, fill-density, and
harmonic-rhythm variants inside each family, then add comparative calibration
tests that tune confidence and ambiguity monotonicity across those variants
before Finch depends on fixed thresholds.
