# g04.002 Schedule Refresh And Gate Collapse Tranche

Date: 2026-03-12
Scope: `crates/signal-runtime/`, `docs/architecture/`, `docs/contracts/`,
`docs/roadmaps/g04/`

## Summary

Completed the third implementation tranche of Batch 2.2 by making
schedule-projection refresh and running forecast-plan churn reuse the same
runtime-owned widened prework service policy, then proving widened requests
still collapse cleanly under plugin and transport gates.

## What changed

- `signal-runtime` now treats `apply_schedule_projection()` as an active runtime
  refresh seam by refreshing scheduler/prework state and rebuilding the current
  forecast window instead of leaving schedule width as passive topology metadata
- running forecast-plan reconciliation now routes through the same widened
  `service_prework_lane_with_policy()` path used by steady-state realtime
  servicing, while paused/non-running reconciliation keeps the existing bounded
  single-cycle priming behavior
- added focused runtime proofs that:
  - applying a compatible schedule projection while running refreshes the
    current prework window using widened requested/effective scope
  - forecast-plan churn while running reuses that widened scope and still
    records invalidation/retirement churn
  - widened requests still yield without servicing when plugin or transport
    gates are active under elevated pressure

## Why this tranche

The earlier Batch 2.2 work widened steady-state service behavior, but refresh
and invalidation paths could still lag behind that policy. This tranche closes
that contract gap inside `signal-runtime` rather than letting hosts infer when
schedule-width changes should matter.

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime schedule_projection_advances_epoch`
- `cargo test -p signal-runtime schedule_projection_refreshes_running_prework_window_with_widened_scope`
- `cargo test -p signal-runtime runtime_forecast_plan_change_rebuild_uses_schedule_widened_service_scope`
- `cargo test -p signal-runtime runtime_schedule_widened_plugin_gate_yields_without_servicing`
- `cargo test -p signal-runtime runtime_schedule_widened_transport_gate_yields_without_servicing`
- `cargo test -p signal-runtime runtime_retires_queued_prework_when_forecast_profile_changes`
- `cargo test -p signal-runtime runtime_rebuilds_missing_queued_prework_when_forecast_window_expands`
- `cargo test -p signal-runtime runtime_scheduler`
- `git diff --check`
- `effigy health --repo .`

## Next

Continue `g04.002` with the next Batch 2.2 runtime-depth tranche and push the
same schedule-width policy through restart/reconfigure and mixed
execution-class transition paths, then close Batch 2.2 with a compact
acceptance proof before moving to Batch 2.3 stress fixtures.
