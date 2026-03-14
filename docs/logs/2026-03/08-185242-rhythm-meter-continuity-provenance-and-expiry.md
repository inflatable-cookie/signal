# Rhythm Meter Continuity Provenance And Expiry

Date: 2026-03-08
Owner: core-product

## Summary

Extended the public rhythm continuity surface so callers now get not only
bar-length and downbeat-phase continuity actions, but also the provenance and
beat-based expiry window for each retained or locked continuity claim.

## Work completed

- replaced the continuity action pair with explicit per-dimension continuity
  plans that now publish:
  - `action`
  - `source`
  - `trusted_beats`
  - `revalidate_after_beats`
- added `MeterContinuitySource` so Signal can distinguish continuity that comes
  from:
  - current promoted meter
  - retained prior meter state
  - a recovery window
  - explicit cleared state
- updated `meter_state_recommendation(...)` so continuity windows now vary by
  state type instead of using bare action labels:
  - stable whole-track meter locks both dimensions from `CurrentMeter`
  - tentative promoted meter retains bar length from `CurrentMeter` but forces
    downbeat-phase reacquisition
  - recovery-backed states retain bar length from `RecoveryWindow` with larger
    beat horizons
  - dropout-heavy hold states retain prior continuity from `PriorMeter`
  - cleared states expire both dimensions immediately
- added longer calibration fixtures:
  - `DropoutVariant::ExtendedHeavy`
  - `BarTransitionVariant::PickupExtended`
  - `BarTransitionVariant::ReentryAcceleratingHarmonyLongSustainedReset`
- updated the offline rhythm demo to print continuity source and beat-based
  revalidation windows
- expanded the test surface so the Signal contract is explicit for:
  - continuity provenance across stable, tentative, recovery, and cleared cases
  - longer-dropout expiry behavior
  - pickup-extension phase reacquisition
  - longer sustained recovery windows carrying longer retained bar-length trust

## Validation

- `cargo test -p signal-analysis-rhythm`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- Effigy runs were kept serial again to avoid the known workspace lock conflict
  when repo-owned tasks overlap.
- The longer-dropout fixture now intentionally surfaces `RecoveryWindow`
  continuity instead of plain `PriorMeter`, which means the public contract can
  distinguish a destabilized hold from a longer-form revalidation attempt.

## Next Task

Deepen the continuity surface with explicit lock expiry and refresh semantics at
the beat-grid level, such as whether retained bar length or phase should decay,
refresh, or be downgraded after each additional unresolved bar, then calibrate
that behavior across even longer multi-section dropout and re-entry families.
