# Runtime Forecast Plan Reconciliation Refill

- Date: 2026-03-10
- Area: `signal-runtime`

## Summary

Signal runtime forecast-plan reconciliation now preserves compatible queued
prework, retires incompatible or out-of-window entries, and immediately
re-primes any missing future targets required by the revised planning window.

## Why

The previous scheduler boundary could trim or preserve queued future work when
forecast mode/profile/policy changed, but it still left holes in the future
window until later host-driven block progression re-primed them. That kept too
much scheduler recovery behavior outside runtime.

## What Changed

- `reconcile_prework_queue_with_current_forecast_plan(...)` now rebuilds the
  full desired target window after selective retirement instead of only
  trimming incompatible entries.
- Runtime passes the complete revised target set back through
  `prepare_engine_prework_window(...)`, so matching queued entries are
  preserved and only missing future blocks are newly admitted.
- Added focused runtime coverage for:
  - incompatible forecast-profile changes now leaving the queue refilled under
    the revised plan
  - compatible window expansion preserving existing entries while priming newly
    missing future blocks
  - shrink behavior still trimming the queue correctly

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_retires_queued_prework_when_forecast_profile_changes -- --nocapture`
- `cargo test -p signal-runtime runtime_rebuilds_missing_queued_prework_when_forecast_window_expands -- --nocapture`
- `cargo test -p signal-runtime runtime_selectively_trims_queued_prework_when_forecast_window_shrinks -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`

## Notes

- No stale Effigy lock recurrence showed up in this batch.
- Workspace Effigy is still blocked outside Signal by the existing Composer
  toolchain issue (`tsc: command not found`) rather than Signal changes.
