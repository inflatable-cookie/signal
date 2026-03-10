# 2026-03-10 15:45 UTC — Runtime-owned prework forecast policy

## Summary

- moved the deterministic future-state forecast for prework priming into `signal-runtime`
- added a compact `RuntimePreworkForecastPolicy` so hosts provide profile-specific policy instead of hand-building every future target
- switched local and server hosts to use runtime-owned forecasted state both for per-block control updates and for prework window priming

## Implementation notes

- `signal-runtime` now exposes forecast helpers for transport, parameter batches, and full planning-window priming from a forecast policy
- local and server hosts now call `apply_forecast_state_for_block(...)` and `prime_engine_prework_window_with_forecast(...)`
- server keeps its distinct seeded synthetic input behavior through the forecast policy rather than host-local target construction

## Validation

- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_primes_prework_window_from_forecast_policy -- --nocapture`
- `cargo test -p signal-runtime runtime_planning_window_reuses_existing_future_sequences_and_allocates_missing -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-local --no-run`

## Next

- move from runtime-owned forecast helpers to runtime-owned forecast windows, so hosts stop even choosing the horizon size and instead provide only profile policy plus current execution context while runtime manages the future planning window end to end
