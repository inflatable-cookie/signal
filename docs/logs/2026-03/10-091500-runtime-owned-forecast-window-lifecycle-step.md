# 2026-03-10 09:15 UTC — Runtime-owned forecast window lifecycle step

## Summary

- removed the remaining host-owned horizon-priming step from local and server block loops
- added a runtime-owned forecast advance call that applies current-block forecast state and reconciles the future prework window in one step
- kept the compact forecast policy as the host-facing profile seam while moving window lifecycle management into `signal-runtime`

## Implementation notes

- added `advance_forecast_state_for_block(...)` to `signal-runtime`
- local and server hosts now call the runtime forecast advance path from block execution instead of running a separate `prime_engine_prework_horizon(...)` step after each block
- initial forecast seeding remains explicit at host startup, but ongoing future-window maintenance is now runtime-owned lifecycle behavior
- added a runtime proof that forecast advance both applies current control state and primes the future window

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_advance_forecast_state_primes_window_and_applies_current_block_state -- --nocapture`
- `cargo test -p signal-runtime runtime_forecast_policy_limits_prework_window_depth -- --nocapture`
- `cargo test -p signal-runtime runtime_primes_prework_window_from_forecast_policy -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`
- `effigy validate --repo .`
- `effigy health --repo .`

## Next

- move from runtime-owned forecast-window lifecycle steps to runtime-owned automatic window reconciliation, so hosts only apply current execution context and runtime maintains the future prework window without any dedicated forecast-advance call at the host layer
