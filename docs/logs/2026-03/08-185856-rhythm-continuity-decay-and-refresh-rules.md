# Rhythm Continuity Decay And Refresh Rules

Date: 2026-03-08
Owner: core-product

## Summary

Extended the public rhythm continuity plan so each retained or locked
bar-length/downbeat-phase claim now includes explicit refresh, decay, and
expiry behavior instead of only a source and revalidation window.

## Work completed

- extended `MeterContinuityPlan` so each continuity dimension now publishes:
  - `refresh_to`
  - `decay_to`
  - `expire_after_beats`
- updated the continuity planner so the Signal-owned contract now distinguishes:
  - stable locks that refresh to `Lock` but decay to `Retain`
  - tentative bar-length retention that can refresh to `Lock` but decays to
    `Clear`
  - prior-meter dropout holds that retain briefly, then decay to `Clear`
  - recovery-window retention that can refresh to `Lock` and otherwise step
    down by staying retained until expiry
  - reacquire-only phase plans that refresh to `Lock` and decay to either
    `Reacquire` or `Clear` depending on context
- updated the offline rhythm demo to print continuity refresh, decay, and
  expiry fields
- added longer calibration fixtures to drive the new lifecycle rules:
  - `DropoutVariant::ExtendedHeavy`
  - `BarTransitionVariant::PickupExtended`
  - `BarTransitionVariant::ReentryAcceleratingHarmonyLongSustainedReset`
- strengthened the tests so Signal now explicitly checks:
  - continuity refresh/decay behavior across stable, tentative, recovering,
    dropout, recovery, and cleared states
  - provenance plus expiry windows on longer dropout and longer sustained
    recovery material
  - longer sustained recovery producing a larger retained bar-length horizon and
    expiry window than the shorter sustained-reset family

## Validation

- `cargo test -p signal-analysis-rhythm`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- Effigy runs were kept serial again to avoid the known workspace lock conflict
  when repo-owned tasks overlap.
- The extended-heavy dropout case intentionally crosses from `PriorMeter`
  continuity into `RecoveryWindow` continuity, which is now a meaningful public
  distinction rather than a test failure.

## Next Task

Deepen the beat-grid lifecycle further by making decay stateful across repeated
failed revalidation intervals, so retained bar-length or downbeat-phase trust
can step through multiple downgrade stages instead of jumping directly from one
static plan, then calibrate that behavior across even longer multi-section
dropout, pickup-extension, and re-entry fixtures.
