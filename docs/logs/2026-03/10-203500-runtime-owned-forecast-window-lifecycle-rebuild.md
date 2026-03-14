# Runtime-Owned Forecast Window Lifecycle Rebuild

- Date: 2026-03-10
- Area: `signal-runtime`, `signal-host-local`, `signal-host-server`

## Summary

Signal runtime now treats forecast-window rebuilding as a lifecycle concern,
not just a block-loop concern. Applying a graph projection and starting or
restarting the runtime can proactively rebuild the current anticipative prework
window from stored forecast state.

## Why

The scheduler layer had moved strongly into `signal-runtime`, but host boot
still carried a special `apply_forecast_state_for_block(0, 0)` seed step to
populate the first anticipative window. That left an unnecessary host seam and
made recovery/start behavior depend on later block progression before the
future window was restored.

## What Changed

- `signal-runtime` now has a runtime-owned prework-window rebuild path that:
  - no-ops when runtime is not configured, forecast mode is disabled, or no
    anticipative graph state exists
  - primes the current forecast window from stored forecast policy/profile and
    current timeline anchor when graph state is available
  - is invoked from:
    - forecast mode/profile/policy changes when queue state alone is not enough
    - graph projection apply
    - runtime configure/start lifecycle paths
- Local and server hosts no longer call
  `apply_forecast_state_for_block(0, 0)` during boot just to seed anticipative
  work.
- Focused runtime tests now cover:
  - graph-apply-driven prework seeding
  - start-after-stop rebuilding the prework window
- The brittle local timeout-recovery proof was updated to assert scheduler
  invariants rather than stale host-era exact timeline literals.

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_apply_graph_projection_primes_prework_window_from_stored_forecast_state -- --nocapture`
- `cargo test -p signal-runtime runtime_start_rebuilds_prework_window_after_runtime_stop -- --nocapture`
- `cargo test -p signal-runtime runtime_retires_queued_prework_when_forecast_profile_changes -- --nocapture`
- `cargo test -p signal-runtime runtime_rebuilds_missing_queued_prework_when_forecast_window_expands -- --nocapture`
- `cargo test -p signal-runtime runtime_selectively_trims_queued_prework_when_forecast_window_shrinks -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`
- `effigy validate`
- `effigy health`

## Notes

- No stale Effigy lock recurrence showed up in this batch.
- The only lock waits observed were normal Cargo/build-directory locks during
  overlapping local Rust commands.
