# 2026-03-10 08:35 UTC — Runtime-owned forecast window management

## Summary

- removed the last host-owned prework horizon cap from local and server host priming
- changed `signal-runtime` so forecast-window priming uses only runtime policy size instead of `remaining_blocks`
- kept the forecast policy as the host-facing seam while moving window depth management fully into runtime

## Implementation notes

- `prime_engine_prework_window_with_forecast(...)` no longer takes `remaining_blocks`
- `RuntimePreworkForecastPolicy.target_window_blocks` is now the only window-size input for forecast priming
- local and server hosts now prime forecasted prework using only current execution context plus runtime profile policy
- host loops no longer gate priming on a remaining-block calculation before asking runtime to maintain the future window

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_forecast_policy_limits_prework_window_depth -- --nocapture`
- `cargo test -p signal-runtime runtime_primes_prework_window_from_forecast_policy -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`
- `effigy validate --repo .`
- `effigy health --repo .`

## Next

- move from runtime-owned forecast-window sizing to runtime-owned forecast-window lifecycle management, so hosts stop explicitly calling a horizon-priming step and runtime maintains and revises the future prework window from current execution context plus profile policy
