# Bounded Runtime Prework Runner And Pending Window Targets

- Date: 2026-03-10
- Area: `signal-runtime`

## Summary

Signal runtime now separates planned future prework targets from already
prepared future work. The runtime-owned forecast window can be larger than the
prepared prework queue, and runtime drains pending future targets through a
bounded anticipative service cycle budget.

## Why

The previous runtime-owned scheduler work had moved forecast planning and
window lifecycle into `signal-runtime`, but the actual preparation step still
collapsed “planned future work” and “prepared future work” into one queue. That
meant the scheduler did not yet have a real bounded runner surface.

## What Changed

- `RuntimePreworkForecastPolicy` now carries `prepare_budget_per_cycle`.
- `signal-runtime` now keeps an internal pending prework-target queue alongside
  the prepared prework cache queue.
- Forecast-window priming and forecast-plan reconciliation now:
  - preserve matching prepared entries
  - retain or revise the full future window target set
  - enqueue missing future targets as pending work
  - prepare only a bounded subset of that pending work per cycle
- The engine snapshot’s window target count/sequences now represent the full
  declared future span, not only the currently prepared queue.
- The local host timeout-recovery proof was adjusted away from stale exact
  prework/timeline literals and toward scheduler invariants consistent with the
  bounded runner model.

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_forecast_runner_leaves_pending_targets_when_budget_is_smaller_than_window -- --nocapture`
- `cargo test -p signal-runtime runtime_apply_graph_projection_primes_prework_window_from_stored_forecast_state -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `git diff --check`

## Notes

- No stale Effigy lock recurrence showed up in this batch.
- The only waits observed were normal Cargo/build-directory locks during
  overlapping Rust commands.
