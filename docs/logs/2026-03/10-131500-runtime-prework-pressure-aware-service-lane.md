# Runtime Prework Pressure-Aware Service Lane

- Date: 2026-03-10
- Area: `signal-runtime`, `signal-host-local`, `signal-host-server`

## Summary

Signal runtime now treats the anticipative prework lane as pressure-aware
scheduler state. The lane can be hinted as `Normal`, `Elevated`, or
`Critical`, and runtime now throttles or yields background prework instead of
draining every service cycle identically.

## Why

The earlier prework service-lane batch created a stateful background lane, but
it still lacked a real runtime-pressure boundary. That meant timeout and
watchdog paths could only affect prework indirectly through later recovery and
queue churn instead of directly telling runtime to back off or yield.

## What Changed

- `RuntimeEngineBlockSnapshot` now carries:
  - `prework_service_pressure`
  - `prework_service_throttle_count`
  - `prework_service_yield_count`
  - last requested/effective service cycles
  - last requested/effective service budget
- `signal-runtime` now:
  - exposes `set_prework_service_pressure(...)` on the projection surface
  - yields the prework lane entirely under `Critical` pressure
  - throttles service down to a minimal drain under `Elevated` pressure
  - records those yield/throttle decisions in the shared engine snapshot
- Local host now drives that pressure surface from its timeout/watchdog path,
  so the real recovery proof exercises at least one prework yield and one
  throttled prework service period instead of only steady-state draining.

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_prework_service_lane_yields_under_critical_pressure -- --nocapture`
- `cargo test -p signal-runtime runtime_prework_service_lane_throttles_under_elevated_pressure -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`

## Notes

- No stale Effigy lock recurrence showed up in this batch.
- The only waits observed were normal Cargo/build-directory locks during
  overlapping local Rust commands.
