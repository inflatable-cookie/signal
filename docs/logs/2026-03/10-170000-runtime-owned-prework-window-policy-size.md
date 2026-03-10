# 2026-03-10 17:00 UTC — Runtime-owned prework window policy size

## Summary

- moved prework window depth from host-local constants into `RuntimePreworkForecastPolicy`
- switched local and server hosts so they pass remaining execution context and profile policy instead of clamping a horizon size themselves
- added a runtime proof that policy-owned window size bounds queued prework depth

## Implementation notes

- `RuntimePreworkForecastPolicy` now includes `target_window_blocks`
- `signal-runtime` now clamps the planning window internally using policy size plus remaining-block context
- local and server hosts no longer carry `PREWORK_PRIMING_HORIZON_BLOCKS`

## Validation

- `cargo test -p signal-runtime runtime_forecast_policy_limits_prework_window_depth -- --nocapture`
- `cargo test -p signal-runtime runtime_primes_prework_window_from_forecast_policy -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-local --no-run`

## Next

- move from runtime-owned forecast-window size to runtime-owned forecast windows end to end, so hosts stop even passing `remaining_blocks` and runtime derives and revises the future planning window from its own execution/timeline state plus profile policy
