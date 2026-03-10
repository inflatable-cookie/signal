# 2026-03-10 09:27 UTC — Runtime automatic forecast window reconciliation

## Summary

- removed the dedicated host-side forecast-advance step from local and server block execution
- merged forecast-state application and future-window reconciliation into one runtime-owned path
- left the compact forecast policy as the host-facing profile seam while making future-window maintenance automatic inside `signal-runtime`

## Implementation notes

- `apply_forecast_state_for_block(...)` in `signal-runtime` now applies the current block forecast state and reconciles the future prework window in one call
- `advance_forecast_state_for_block(...)` was removed
- local and server hosts now use only the runtime forecast-application path during block execution
- initial startup seeding still uses the same runtime method, so the first current-block state application and future-window priming follow the same path as steady-state block execution

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_apply_forecast_state_primes_window_and_applies_current_block_state -- --nocapture`
- `cargo test -p signal-runtime runtime_forecast_policy_limits_prework_window_depth -- --nocapture`
- `cargo test -p signal-runtime runtime_primes_prework_window_from_forecast_policy -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`
- `effigy validate --repo .`
- `effigy health --repo .`

## Next

- move from automatic runtime forecast-window reconciliation to runtime-owned forecast-policy persistence, so hosts stop passing the same profile policy on every block and runtime maintains the future prework window from stored policy plus current execution context alone
