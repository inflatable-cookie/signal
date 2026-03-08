# Rhythm Staged Continuity Lifecycle

Date: 2026-03-08
Owner: core-product

## Summary

Replaced the one-step continuity decay surface with an explicit staged
continuity lifecycle so Signal can describe how bar-length and downbeat-phase
trust should refresh, degrade, and eventually clear across repeated failed
revalidation windows.

## Work completed

- replaced the flat continuity tail fields with structured lifecycle data:
  - `MeterContinuityTransition`
  - `MeterContinuityLifecycle`
- updated `MeterContinuityPlan` so each dimension now publishes:
  - the immediate continuity action and source
  - the next successful refresh transition
  - two sequential decay transitions for unresolved continuation
- recalibrated `meter_state_recommendation(...)` so lifecycle schedules differ
  by state type instead of collapsing to a single `decay_to` target:
  - stable meter now steps from `Lock` to retained continuity before clearing
  - tentative promoted meter now steps from retained bar length to
    `Reacquire`, then `Clear`
  - destabilized prior-meter hold now degrades from retained prior state to
    reacquisition before clearing
  - recovery-window watch states now step from retained recovery continuity to
    reacquisition before clearing
  - cleared states remain fully cleared at every stage
- updated the offline rhythm demo to print the full staged lifecycle for both
  bar length and downbeat phase
- refreshed the calibration tests so they assert staged refresh/decay behavior
  instead of only single-step decay labels

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo run -p signal-analysis-rhythm --example offline_rhythm_demo -- --bpm 120 --seconds 8`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- The public continuity contract is now closer to a consumer-facing state
  machine: callers can see how long a lock remains trusted, what it refreshes
  to when new evidence appears, and how it degrades if confirmation keeps
  failing.
- Existing longer fixtures were sufficient to calibrate the staged lifecycle,
  so this batch focused on API and behavior hardening rather than adding more
  one-off scenario families.

## Next Task

Deepen the staged continuity lifecycle with explicit per-stage confidence or
severity metadata, then calibrate how far bar-length and downbeat-phase trust
should decay across longer unresolved spans such as repeated pickup extension,
deeper dropout chains, and multi-section recovery drift.
