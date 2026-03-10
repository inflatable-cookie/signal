# Rhythm Tempo Consumption Fallback Policy

Date: 2026-03-09
Owner: core-product

## Summary

Tuned the compact `tempo_consumption(...)` surface so monitor-state tempo no
longer falls back the same way for every partially stable case. `CoreStableOnly`
material now prefers a prior locked tempo when one exists, while
`StableWithLocalizedEdgeDamage` continues to keep the current snapped tempo with
its guarded fallback horizon.

## Work completed

- updated `BeatAnalysisResult::tempo_consumption(...)` in
  `crates/signal-analysis-rhythm/src/lib.rs` to make fallback selection aware of
  `TempoStabilityScope`
- added a dedicated `prior_tempo_selection(...)` helper so monitor-state tempo
  can fall back to a caller-supplied prior tempo without changing the compact
  result shape
- changed `CoreStableOnly + Monitor` behavior so:
  - with a prior tempo, fallback becomes `PriorTempo`
  - without a prior tempo, fallback remains `NoTempo`
  - fallback horizons come from the existing continuity expiry instead of new
    wrapper heuristics
- kept `StableWithLocalizedEdgeDamage + Lock` on the current snapped tempo for
  both current and fallback selections, preserving the shorter guarded horizon
  introduced in the previous batch
- updated the real-path tempo consumption test expectations for:
  - neutral integer lock
  - slow core-stable monitor with prior tempo
  - slow core-stable monitor without prior tempo
  - weak-backbeat refined tempo that remains lockable
- marked the whole-track-default tempo-state wrappers as test-only where they no
  longer participate in non-test builds

## Real-file result

Test file:
`/Users/betterthanclay/Library/CloudStorage/Dropbox/Music/Projects/Garamond/1. 086/Output/1. 086 - v1.5 - Master Stream.wav`

Current result after this batch:

- `bpm=128.00000`
- `tempo_interpretation=SnapInteger/NearIntegerPulse`
- `tempo_state=Lock/StableTempoWithEdgeDamage@0.760`
- `tempo_consumption=current:SnappedCurrentTempo@Some(128.0)/fallback:SnappedCurrentTempo@Some(128.0)/after:14/scope:StableWithLocalizedEdgeDamage`

This path stays intentionally unchanged. The track is still treated as a stable
tempo lock with localized edge damage, so consumers do not need to fall back to
prior tempo or clear tempo state.

## Calibration result

The short 90 BPM example now makes the fallback distinction explicit:

- `tempo_stability_scope=CoreStableOnly`
- `tempo_state=action:Monitor,reason:CoreStableTempo`
- `tempo_consumption=current:SnappedCurrentTempo@Some(90.0)/fallback:NoTempo@None/action:Monitor/Reacquire/after:12@0.813/scope:CoreStableOnly`

In the real-path unit test, the same class of `CoreStableOnly` result now uses a
caller-supplied prior tempo when one exists:

- `fallback:PriorTempo@Some(89.75)`
- `fallback_after_beats=8`

So Signal now makes the intended distinction directly:

- partially stable tempo with no prior context should reacquire and eventually
  clear
- partially stable tempo with prior context should reacquire while temporarily
  preserving the caller's prior tempo

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo run -q -p signal-analysis-rhythm --example file_rhythm_probe -- '/Users/betterthanclay/Library/CloudStorage/Dropbox/Music/Projects/Garamond/1. 086/Output/1. 086 - v1.5 - Master Stream.wav'`
- `cargo run -q -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 90 --seconds 8`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy test --repo .`
- `git diff --check`

## Notes

- `effigy` still needs serial execution on this repo because overlapping runs
  continue to hit the known workspace lock conflict.
- This batch intentionally changes only the compact consumer fallback policy. It
  does not change Garamond's exact BPM result, integer snap behavior, or the
  broader tempo-state classification.

## Next Task

Tune the compact tempo-consumption fallback horizons themselves, especially
deciding whether `StableWithLocalizedEdgeDamage` should keep a shorter guarded
lock window than `WholeTrackStable`, and whether `CoreStableOnly` should clear
more aggressively when no prior tempo exists but boundary pressure is low enough
that reacquisition is still plausible.
