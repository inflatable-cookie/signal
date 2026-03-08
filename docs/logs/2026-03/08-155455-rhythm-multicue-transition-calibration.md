# Rhythm Multicue Transition Calibration

Date: 2026-03-08
Owner: core-product

## Summary

Extended the bar-transition preset family with stacked destabilizers so Signal
can now compare meter recovery versus unknown-meter fallback when bar-structure
change happens together with harmonic shift or dense fill activity.

## Work completed

- widened `BarTransitionVariant` with combined-cue cases for:
  - re-entry plus harmonic shift
  - re-entry plus dense fill activity
  - modulation plus dense fill activity
- kept those scenarios inside the shared named preset builder rather than
  creating ad hoc one-off tests outside the preset surface
- updated the named preset expectation table so combined-cue transitions now
  participate in the same calibration layer as simpler transition cases
- strengthened transition-family comparisons to check that:
  - re-entry can still recover four-four under added harmonic or fill pressure
  - modulation plus dense fill remains meter-unknown
  - the unknown-meter combined case carries at least as much ambiguity as the
    recovered-meter combined cases

## Validation

- `cargo test -p signal-analysis-rhythm`
- `effigy validate --repo .`
- `cargo test --workspace`:
  failed only at `signal-ipc` doctest time because `signal-ipc/src/lib.rs`
  imports `memmap2` without the crate being available in that doctest context
- `git diff --check`

## Notes

- The rhythm crate still passes all 26 targeted tests after adding the
  combined-cue transition surface.
- This batch keeps the work inside Signal’s reusable DSP/analysis layer; the
  remaining workspace failure is outside the touched rhythm code.
- The transition surface now distinguishes:
  - stable recovery after a marked re-entry
  - recovery under extra harmonic or fill complexity
  - stacked destabilization that should remain meter-unknown

## Next Task

Add cross-family transition presets that mix dropout/re-entry with explicit
harmonic-rhythm acceleration or deceleration over multiple sections, then tune
how tempo ambiguity and meter confidence evolve when both pulse density and bar
structure change together.
