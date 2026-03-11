---
title: g01.007 runtime prework stabilization tranche
status: completed
owner: nucleus
updated: 2026-03-11
tags: [signal, runtime, roadmap, g01, g01.007, prework, tests]
---

## Summary

Stabilized a broad slice of `signal-runtime` after the transport-truth changes in `g01.007`.

The main outcome of this tranche was separating manual prework-cache tests from the repo's role-default forecast behavior so those tests no longer inherit forecast-window priming unless they are explicitly about forecast behavior.

## Changes

- Added a test helper in `crates/signal-runtime/src/runtime.rs` for configuring anticipative runtime tests with forecast mode disabled.
- Moved the manual prework cache / queue tests onto that helper so they validate explicit prework behavior instead of implicit role-default forecast state.
- Updated the forecast-window tests that were stale under the new transport-authority model:
  - compatible queued targets can be preserved rather than re-admitted
  - future-window sequences are planned as future-only targets
  - profile changes can leave pending work when budget is bounded
- Reworked the parameter/transport invalidation test setup so it uses explicit future prework targets instead of relying on incidental queued state.

## Validation

- Passed focused runtime tests for:
  - forecast window priming and queue trimming
  - graph-replan and manual prework queue behavior
  - block-sequence expiry
- `cargo test -p signal-runtime -- --nocapture` improved from 17 failures to 9 failures.
- `effigy validate --repo .` passed.
- `effigy health --repo .` failed in legacy CMake configure/build because of an existing VST3 sample bundle configure error under `legacy/cpp/build`; this was outside the Rust runtime changes.

## Remaining failures

The current remaining `signal-runtime` failures are clustered in smaller contracts rather than the earlier broad prework-window breakage:

- pressure-policy expectations:
  - `runtime_prework_service_lane_yields_under_critical_pressure`
  - `runtime_elevated_pressure_preserves_deferred_prework_targets`
  - `runtime_latency_focused_graph_expands_elevated_pressure_service_scope`
  - `runtime_plugin_backed_graph_constrains_elevated_pressure_service_scope`
- stale or incomplete snapshot/observation expectations:
  - `runtime_executes_applied_graph_block_and_updates_snapshot`
  - `runtime_event_recorder_builds_reusable_observation_diagnostics`
  - `transport_session_summary_tracks_concurrent_active_sessions`
- one remaining invalidation contract:
  - `runtime_invalidates_prework_cache_on_parameter_and_transport_changes`
- one still-flaky forecast count assertion:
  - `runtime_forecast_policy_limits_prework_window_depth`

## Notes

This tranche reduced churn by treating the remaining runtime failures as two different categories:

1. Manual prework tests that needed isolation from role-default forecast behavior.
2. Genuine remaining runtime contract mismatches that still need targeted follow-up.
