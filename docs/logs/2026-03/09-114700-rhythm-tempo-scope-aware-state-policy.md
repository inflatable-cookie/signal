# Rhythm Tempo Scope-Aware State Policy

Date: 2026-03-09
Owner: core-product

## Summary

Connected the new tempo stability scope to Signal's actual tempo-state and
tempo-consumption policy. Stable tempo with localized edge damage can now stay
locked with an explicit guarded reason, while core-only stability no longer gets
the same hard-lock treatment as whole-track or edge-damaged stable material.

## Work completed

- added two tempo-state reasons in
  `crates/signal-analysis-rhythm/src/lib.rs`:
  - `StableTempoWithEdgeDamage`
  - `CoreStableTempo`
- split the tempo-state mapper into a scope-aware path used by real analysis and
  retained a test-only whole-track-default wrapper for older synthetic callers
- updated the state policy so:
  - `StableWithLocalizedEdgeDamage` plus strong integer/refined tempo can still
    `Lock`, but with shorter trust and revalidation windows
  - `CoreStableOnly` with plausible current tempo now `Monitor`s and
    `Reacquire`s instead of hard-locking
- propagated the new policy into `tempo_consumption(...)` without changing the
  compact current/fallback tempo surface shape
- updated tests to pin:
  - edge-damaged integer lock behavior
  - core-stable integer monitor behavior
  - the short 90 BPM click-track downgrade
  - preserved lock behavior for richer weak-backbeat refined-tempo material

## Real-file result

Test file:
`/Users/betterthanclay/Library/CloudStorage/Dropbox/Music/Projects/Garamond/1. 086/Output/1. 086 - v1.5 - Master Stream.wav`

Current result after this batch:

- `bpm=128.00000`
- `tempo_interpretation=SnapInteger/NearIntegerPulse`
- `tempo_state=Lock/StableTempoWithEdgeDamage@0.760`
- `tempo_consumption=current:SnappedCurrentTempo@Some(128.0)/fallback:SnappedCurrentTempo@Some(128.0)/after:14/scope:StableWithLocalizedEdgeDamage`

This is the intended contract. Garamond still gets a hard lock because the
tempo pulse is strong and the instability is localized, but Signal now labels it
as an edge-damaged stable lock instead of treating it as indistinguishable from
whole-track-stable material.

## Calibration result

The short 90 BPM click-track example now behaves differently by design:

- `tempo_stability_scope=CoreStableOnly`
- `tempo_state=action:Monitor,reason:CoreStableTempo`
- `tempo_consumption=.../action:Monitor/Reacquire/.../scope:CoreStableOnly`

That downgrade is intentional because the short analysis only recovers a stable
core region after an early beat miss, so Signal should no longer overclaim a
full hard lock for that case.

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo run -q -p signal-analysis-rhythm --example file_rhythm_probe -- '/Users/betterthanclay/Library/CloudStorage/Dropbox/Music/Projects/Garamond/1. 086/Output/1. 086 - v1.5 - Master Stream.wav'`
- `cargo run -q -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 90 --seconds 8`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy test --repo .`
- `git diff --check`

## Notes

- `effigy` still needs serial execution on this repo; overlapping runs continue
  to hit the known workspace lock conflict.
- The current policy change is intentionally conservative: it only downgrades
  cases classified as `CoreStableOnly`, while `StableWithLocalizedEdgeDamage`
  stays lockable if the tempo interpretation itself is strong enough.

## Next Task

Tune the compact tempo-consumption fallback policy for `CoreStableOnly` and
`StableWithLocalizedEdgeDamage`, especially deciding when a monitor state should
fall back to prior tempo versus no tempo, and whether edge-damaged locks should
carry a distinct fallback horizon from whole-track-stable locks in the consumer
surface.
