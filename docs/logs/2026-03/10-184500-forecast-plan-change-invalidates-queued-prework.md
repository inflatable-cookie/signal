---
title: Forecast Plan Change Invalidates Queued Prework
status: completed
updated: 2026-03-10
owner: codex
---

## Summary

- made runtime forecast profile/policy/mode transitions retire queued future prework when the forecast plan changes
- added `ForecastPlanChanged` as a runtime prework invalidation and retirement reason
- kept requested/effective forecast mode reconciliation intact while pushing that boundary into the actual scheduler path instead of only snapshot metadata
- pinned the scheduler behavior with focused runtime tests for explicit profile changes and anticipative-disabled reconfigure, plus the existing local/server timeout-recovery proofs

## Validation

- `cargo test -p signal-runtime runtime_retires_queued_prework_when_forecast_profile_changes -- --nocapture`
- `cargo test -p signal-runtime runtime_retires_queued_prework_when_effective_mode_drops_to_disabled -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `git diff --check`

## Notes

- no stale Effigy lock recurrence showed up in this batch
- this batch deliberately targeted the runtime scheduler boundary; broader workspace Effigy still depends on the Composer `tsc` toolchain being present
