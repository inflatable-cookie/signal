# Rhythm Reanchor Recovery Calibration

Date: 2026-03-08
Owner: core-product

## Summary

Extended the rhythm transition preset family with section-boundary harmonic
reset and cadential re-anchor cases, then tuned the meter scorer to pay more
attention to late-bar recovery evidence. The resulting calibrated surface keeps
these stacked-cue re-anchor cases explicitly meter-unknown today, but now
distinguishes plain accent drift from stronger partial recovery at the end of a
destabilized window.

## Work completed

- widened `BarTransitionVariant` with four new multistage re-entry cases:
  - `ReentryAcceleratingHarmonyReset`
  - `ReentryDeceleratingHarmonyReset`
  - `ReentryAcceleratingHarmonyCadentialReanchor`
  - `ReentryDeceleratingHarmonyCadentialReanchor`
- added reusable harmonic-reset and cadential re-anchor section patterns so the
  re-entry family now covers:
  - accent-shift destabilization
  - partial harmonic reset
  - stronger cadence-like end-of-window re-anchor
- updated `infer_meter(...)` to weight later bars more heavily and add a
  `recent_strength` component to both hypothesis scoring and confidence, so
  late stable sections contribute more than earlier destabilized bars
- extended the named preset expectation table and transition-family calibration
  checks with the new re-anchor cases
- added a dedicated re-anchor recovery comparison test that now codifies the
  current Signal-owned contract:
  - accent-shift, reset, and cadential re-anchor variants all remain
    `meter: None` under the present gate
  - reset and cadential tails still raise overall recovery confidence relative
    to simpler destabilized windows

## Validation

- `cargo test -p signal-analysis-rhythm`
- `effigy health`
- `effigy validate`
- `cargo test --workspace`
- `git diff --check`

## Notes

- The rhythm crate now has 29 targeted tests covering harmonic drift, density
  drift, accent drift, and section-boundary re-anchor behavior.
- The meter scorer change is intentionally small and Signal-owned: later stable
  bars are weighted more strongly, but the current bar-claim gate still
  suppresses meter on these re-anchor fixtures.
- This batch is useful even without a meter claim because it establishes a
  calibrated recovery surface for the next scorer pass instead of leaving Finch
  to guess how late re-anchors should be interpreted.

## Next Task

Deepen `infer_meter(...)` beyond the current whole-track gate by adding
windowed or segment-aware meter scoring so late stable re-anchor sections can
optionally promote `meter` back from `None` when recovery is locally strong,
without regressing mixed-length and dropout-heavy unknown-meter behavior.
