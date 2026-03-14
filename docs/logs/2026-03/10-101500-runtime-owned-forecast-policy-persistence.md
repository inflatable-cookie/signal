# 2026-03-10 10:15 UTC — Runtime-owned forecast policy persistence

## Summary

- moved prework forecast policy ownership into `signal-runtime` as stored runtime state
- switched local and server hosts to set forecast policy once during boot instead of passing it on every block
- extended the shared engine snapshot so supervisor/report surfaces can see whether a runtime forecast policy is configured and what target window size it carries

## Implementation notes

- added `set_prework_forecast_policy(...)` to the runtime projection surface
- `apply_forecast_state_for_block(...)` now uses stored runtime policy instead of taking a policy parameter on each call
- runtime clears stored forecast policy on `configure(...)`, so reconfiguration requires an explicit policy reset instead of silently carrying stale profile assumptions
- local and server hosts now set forecast policy once during boot and then apply forecast state per block without resending the profile policy

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_apply_forecast_state_primes_window_and_applies_current_block_state -- --nocapture`
- `cargo test -p signal-runtime runtime_clears_persisted_forecast_policy_on_reconfigure -- --nocapture`
- `cargo test -p signal-runtime runtime_forecast_policy_limits_prework_window_depth -- --nocapture`
- `cargo test -p signal-runtime runtime_primes_prework_window_from_forecast_policy -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`
- `effigy validate`
- `effigy health`

## Next

- move from runtime-owned forecast-policy persistence to runtime-owned forecast-policy selection and profile switching, so hosts stop constructing profile-specific forecast policies inline and instead choose a runtime-known policy/profile identifier with only minimal overrides where necessary
