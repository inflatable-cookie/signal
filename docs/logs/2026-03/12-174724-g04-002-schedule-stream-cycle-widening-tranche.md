# g04.002 Schedule Stream Cycle Widening Tranche

Date: 2026-03-12
Scope: `crates/signal-runtime/`, `docs/architecture/`, `docs/contracts/`,
`docs/roadmaps/g04/`

## Summary

Completed the second implementation tranche of Batch 2.2 by making compatible
schedule-stream capacity widen requested anticipative service cadence at the
runtime-owned realtime service call sites.

## What changed

- `signal-runtime` now scales requested anticipative service cycles from
  compatible `ScheduleProjection.stream_count` before calling
  `service_prework_lane_with_policy()` from both
  `prime_engine_prework_window_with_forecast()` and
  `enforce_scheduler_after_engine_block()`
- added focused runtime proofs that:
  - a compatible wider schedule widens both requested cycles and effective
    normal-pressure service scope
  - a missing schedule projection still keeps service at the base scope
  - elevated pressure clamps widened cycle requests back to the bounded safe
    scope while preserving runtime-owned throttle receipts
  - the existing normal-pressure realtime scheduler path remains stable
- updated the multicore scheduling contract and architecture reference so the
  frozen scheduler interpretation now includes schedule-width-driven cadence
  widening in addition to budget widening

## Why this tranche

The prior Batch 2.2 tranche made schedule width affect only per-cycle budget,
which left cadence and dispatch behavior unchanged. This tranche moves one step
deeper into runtime-owned scheduling behavior without inventing a thread model
or pushing policy decisions into host-local code.

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_compatible_schedule_projection_widens_normal_prework_service_scope`
- `cargo test -p signal-runtime runtime_missing_schedule_projection_does_not_widen_prework_service_budget`
- `cargo test -p signal-runtime runtime_elevated_pressure_clamps_schedule_widened_prework_cycles`
- `cargo test -p signal-runtime runtime_realtime_block_services_prework_window_under_normal_pressure`
- `cargo test -p signal-runtime runtime_scheduler`
- `git diff --check`
- `effigy health --repo .`

## Next

Continue `g04.002` with the next Batch 2.2 runtime-depth tranche and extend
schedule-width effects into transition-heavy invalidation and refresh paths,
then prove widened service scope still collapses cleanly under plugin and
transport gating without introducing host-local scheduler policy.
