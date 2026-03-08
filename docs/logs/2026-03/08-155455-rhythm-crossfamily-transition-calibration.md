# Rhythm Crossfamily Transition Calibration

Date: 2026-03-08
Owner: core-product

## Summary

Extended the rhythm bar-transition family with crossfamily combined-cue
scenarios so Signal can now compare meter recovery against unknown-meter
fallback when dropout/re-entry is mixed with harmonic-shift pressure or dense
fill activity.

## Work completed

- widened `BarTransitionVariant` with new crossfamily cases for:
  - re-entry plus harmonic shift
  - re-entry plus dense fill activity
  - modulation plus dense fill activity
- implemented those cases inside the shared named preset builder so they remain
  part of the same reusable calibration surface as the existing transition
  families
- updated the named preset expectation table to encode which combined-cue
  transitions should still recover four-four meter and which should stay
  meter-unknown
- strengthened the transition-family comparisons to check that:
  - re-entry can still recover meter under added harmonic or fill complexity
  - modulation plus dense fill stays meter-unknown
  - the fully destabilized combined case carries at least as much tempo
    ambiguity as the recovered combined-cue cases

## Validation

- `cargo test -p signal-analysis-rhythm`
- `effigy validate --repo .`
- `cargo test --workspace`:
  failed only at `signal-ipc` doctest time because `signal-ipc/src/lib.rs`
  imports `memmap2` without the crate being available in that doctest context
- `git diff --check`

## Notes

- The rhythm crate still passes all 26 targeted tests after adding the
  crossfamily transition surface.
- This batch keeps the work inside Signal’s reusable rhythm analysis surface;
  the remaining workspace failure is outside the touched rhythm code.
- The transition surface now distinguishes:
  - clean re-entry recovery
  - recovery under extra harmonic or fill complexity
  - stacked modulation-plus-fill disruption that should remain meter-unknown

## Next Task

Add multi-section transition presets that combine dropout/re-entry with
explicit harmonic-rhythm acceleration and deceleration over successive sections,
then tune how confidence and tempo ambiguity evolve when pulse density and bar
structure drift across more than one recovery stage.
