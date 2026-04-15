# Rhythm Tempo Consumption Horizon Tuning

Date: 2026-03-09
Owner: core-product

## Summary

Tuned the compact `tempo_consumption(...)` horizon policy so the published
fallback timing now reflects the stability scope more directly. Stable tempo
with localized edge damage keeps a shorter guarded lock horizon than
whole-track-stable tempo, and `CoreStableOnly` monitor paths now clear earlier
when no prior tempo exists and the remaining instability is localized enough
that continued no-context monitoring is not useful.

## Work completed

- updated `BeatAnalysisResult::tempo_consumption(...)` in
  `crates/signal-analysis-rhythm/src/lib.rs` with an explicit early-clear rule
  for `CoreStableOnly` monitor results that have:
  - no prior tempo
  - low boundary pressure
  - high enough confidence and interior stability to make reacquisition
    plausible without preserving stale state
- kept the existing prior-tempo fallback path for `CoreStableOnly` monitor
  results that do have caller context
- added a localized-edge lock horizon helper in the tempo-state mapper so
  `StableWithLocalizedEdgeDamage` no longer reuses the broader whole-track lock
  windows
- tightened localized-edge lock timings to a more clearly guarded profile when
  edge pressure is meaningfully non-zero
- updated direct tests so the compact consumer surface now pins:
  - earlier no-prior clear for core-stable monitor tempo
  - shorter guarded lock windows for edge-damaged stable tempo

## Real-file result

Test file:
`~/Library/CloudStorage/Dropbox/Music/Projects/Garamond/1. 086/Output/1. 086 - v1.5 - Master Stream.wav`

Current result after this batch:

- `bpm=128.00000`
- `tempo_interpretation=SnapInteger/NearIntegerPulse`
- `tempo_state=Lock/StableTempoWithEdgeDamage@0.760`
- `tempo_consumption=current:SnappedCurrentTempo@Some(128.0)/fallback:SnappedCurrentTempo@Some(128.0)/after:12/scope:StableWithLocalizedEdgeDamage`

This is the intended consumer contract. Garamond still locks hard on the
snapped current tempo, but the published guarded fallback horizon is now
shorter than the whole-track-stable lock path.

## Calibration result

The short 90 BPM click example now clears more aggressively when no prior tempo
exists:

- `tempo_stability_scope=CoreStableOnly`
- `tempo_state=action:Monitor,reason:CoreStableTempo`
- `tempo_consumption=current:SnappedCurrentTempo@Some(90.0)/fallback:NoTempo@None/action:Monitor/Reacquire/after:8@0.813/scope:CoreStableOnly`

The corresponding real-path test still preserves prior tempo when caller context
exists:

- `fallback:PriorTempo@Some(89.75)`
- `fallback_after_beats=8`

So the compact policy is now explicit:

- `WholeTrackStable` lock keeps the longer fallback window
- `StableWithLocalizedEdgeDamage` lock keeps a shorter guarded window
- `CoreStableOnly` monitor with prior tempo preserves that prior tempo briefly
- `CoreStableOnly` monitor without prior tempo clears on the earlier horizon
  when continued carry would only preserve uncertainty

## Validation

- `cargo test -p signal-analysis-rhythm tempo_state_locks_edge_damaged_integer_scope -- --nocapture`
- `cargo test -p signal-analysis-rhythm beat_tracker_resolves_tempo_consumption_across_real_analysis_paths -- --nocapture`
- `cargo run -q -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 90 --seconds 8`
- `cargo run -q -p signal-analysis-rhythm --example file_rhythm_probe -- '~/Library/CloudStorage/Dropbox/Music/Projects/Garamond/1. 086/Output/1. 086 - v1.5 - Master Stream.wav'`
- `cargo test -p signal-analysis-rhythm`
- `effigy health`
- `effigy validate`
- `effigy test`
- `git diff --check`

## Notes

- `effigy` still needs serial execution on this repo because overlapping runs
  continue to hit the known workspace lock conflict.
- This batch changes the compact published fallback timing, not the actual BPM
  estimate or integer-snap decision for Garamond.

## Next Task

Add explicit compact tempo-consumption horizon semantics on top of the current
beat counts, such as whether a fallback window is a hard lock window, a guarded
carry window, or a reacquisition-only grace window, so consumers can distinguish
why two decisions share similar beat counts without unpacking the full tempo
continuity tree.
