---
title: Selective Prework Preservation Across Forecast Plan Changes
status: completed
updated: 2026-03-10
owner: codex
---

## Summary

- changed forecast-plan reconciliation so `signal-runtime` no longer flushes the entire queued prework window on every forecast profile or mode change
- preserved queued future entries that still match the revised forecast plan and retired only the entries that fall out of scope or no longer match the revised future transport/parameter/input forecast
- kept `ForecastPlanChanged` as the runtime-visible invalidation boundary while making it a selective scheduler reconciliation instead of unconditional full queued-work retirement

## Validation

- `cargo test -p signal-runtime runtime_preserves_compatible_queued_prework_when_forecast_mode_changes_but_plan_matches -- --nocapture`
- `cargo test -p signal-runtime runtime_selectively_trims_queued_prework_when_forecast_window_shrinks -- --nocapture`
- `cargo test -p signal-runtime runtime_retires_queued_prework_when_forecast_profile_changes -- --nocapture`
- `git diff --check`

## Notes

- no stale Effigy lock recurrence showed up in this batch
- this batch intentionally stayed on the runtime scheduler slice rather than rerunning broad workspace validation after every small adjustment
