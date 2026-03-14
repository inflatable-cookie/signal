---
title: Requested vs Effective Forecast Mode Reconciliation
status: completed
updated: 2026-03-10
owner: codex
---

## Summary

- split runtime forecast state into requested mode and effective mode so explicit/raw override intent can survive reconfigure and recovery while effective mode still drops to `Disabled` when anticipative planning is off
- made `configure(...)` reconcile forecast state from stored request instead of resetting explicit overrides back to role default on every reconfigure
- allowed explicit profile and raw policy override requests to be stored while anticipative planning is disabled, with effective mode remaining `Disabled` until anticipative planning is re-enabled
- preserved raw override state across restart without reconfigure and exposed requested/effective mode separately through the shared engine snapshot/export surface

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime forecast -- --nocapture`
- `cargo test -p signal-runtime runtime_restart_preserves_raw_forecast_override_request -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`
- `effigy validate`
- `effigy health`

## Notes

- no stale Effigy lock recurrence showed up in this batch
- current workspace Effigy failures, if present, are still expected to come from the Composer `tsc` toolchain gap rather than this Signal runtime slice
