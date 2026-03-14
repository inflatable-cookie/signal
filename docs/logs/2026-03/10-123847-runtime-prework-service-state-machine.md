# Runtime Prework Service State Machine

- Date: 2026-03-10
- Area: `signal-runtime`, `signal-host-local`, `signal-host-server`

## Summary

Signal runtime now treats the anticipative prework lane as a stateful scheduler
surface. The background lane no longer appears only as a bounded service call;
it now reports whether prework is disabled, idle, pending, actively servicing,
paused, or starved.

## Why

The previous service-lane batch separated future-window reconciliation from
future-work preparation, but the lane was still too opaque. It could advance
future targets, yet it did not say whether the scheduler was paused behind a
stopped runtime, starved by zero budget, or simply idle because no future work
remained.

## What Changed

- `RuntimePreworkServiceState` is now part of the shared engine snapshot, with
  `Disabled`, `Idle`, `Pending`, `Servicing`, `Paused`, and `Starved` states.
- `signal-runtime` now:
  - reconciles prework-lane state when lifecycle, forecast, or queue state
    changes
  - counts pause, resume, and starvation transitions
  - marks the service lane `Starved` when pending work exists but the runtime
    is asked to service with zero effective budget
  - restores the service lane to `Pending` or `Idle` when runtime resumes and
    future work can move again
- Shared report/render surfaces now surface that scheduler state directly
  instead of leaving it visible only through raw JSON fields.

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_prework_service_lane_enters_starved_state_when_budget_is_zero -- --nocapture`
- `cargo test -p signal-runtime runtime_prework_service_lane_resumes_after_start -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`
- `effigy validate`
- `effigy health`

## Notes

- No stale Effigy lock recurrence showed up in this batch.
- The only waits were normal Cargo/build-directory locks during overlapping
  local Rust commands.
