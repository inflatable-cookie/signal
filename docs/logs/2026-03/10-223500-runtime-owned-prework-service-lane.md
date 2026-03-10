# Runtime-Owned Prework Service Lane

- Date: 2026-03-10
- Area: `signal-runtime`, `signal-host-local`, `signal-host-server`

## Summary

Signal runtime now separates future-window reconciliation from future-work
preparation. Forecast application keeps the declared anticipative window in
sync, while a dedicated runtime-owned prework service lane advances pending
future targets independently of the realtime block path.

## Why

The earlier bounded prework runner proved that runtime could keep pending
future targets internally, but pending work still advanced mostly as a side
effect of forecast application and lifecycle rebuilds. That was still too
close to host-driven scheduler behavior.

## What Changed

- `RuntimeEngineBlockSnapshot` now reports:
  - `prework_pending_target_count`
  - cumulative prework service-cycle count
  - cumulative targets prepared by the service lane
  - last service processing epoch
  - last service cycle count
  - last service budget per cycle
  - last service prepared-target count
- `RuntimeProjectionApi` now includes `service_prework_lane(...)`.
- `signal-runtime` now:
  - reconciles the forecast window without automatically draining all pending
    future work during `apply_forecast_state_for_block(...)`
  - exposes `service_prework_lane(...)` as the explicit bounded scheduler step
  - keeps graph-apply and start/restart rebuild paths proactive by reconciling
    and servicing the current forecast window from stored runtime state
- Local and server hosts now call the runtime-owned prework service lane around
  each realtime cycle, so pending future targets can advance independently of
  current-block forecast application.
- Focused runtime and host proofs were updated to assert the new scheduler
  semantics rather than stale exact queue-state assumptions from the older
  forecast-driven path.

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_forecast_runner_leaves_pending_targets_when_budget_is_smaller_than_window -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`

## Notes

- No stale Effigy lock recurrence showed up in this batch.
- Normal Cargo/build-directory lock waits still occurred when targeted Rust
  commands overlapped locally.
