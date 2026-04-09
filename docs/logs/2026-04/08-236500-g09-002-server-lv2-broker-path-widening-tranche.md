# 2026-04-08 - g09.002 server LV2 broker-path widening tranche

## Summary

Extended the broker-backed execution lane in `signal-host-server` to LV2, so
the server host now routes AU, VST3, and LV2 through the same typed broker
attach and teardown path when the broker environment is enabled.

## Work Completed

- updated `/crates/signal-host-server/src/host_support/sandbox_sessions.rs`
  - LV2 now uses the broker-backed ensure path when the broker environment is
    enabled
  - LV2 reuses the existing shared server broker attach helper used by AU and
    VST3
- updated `/crates/signal-host-server/src/host.rs`
  - broker-backed LV2 ensure now retains the returned broker session for later
    teardown instead of dropping it
- expanded `/crates/signal-host-server/tests/public_host_edge_sandbox_broker.rs`
  - added a focused LV2 broker roundtrip proof alongside the existing AU and
    VST3 proofs

## Validation

- `cargo check -p signal-host-server`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`

## Outcome

The broker-backed steady-state execution lane now covers:

- local host AU
- local host VST3
- server host AU
- server host VST3
- server host LV2

That closes the remaining server-side single-format gap inside the current
Batch 2.3 widening work. The next useful tranche should refocus away from
steady-state format widening and push the shared runtime-owned broker layer
earlier into recovery/control flow, especially replacement-session start
failure or overlap-prepare rollback, so broker ownership stops being shared
mainly at steady-state and teardown time.
