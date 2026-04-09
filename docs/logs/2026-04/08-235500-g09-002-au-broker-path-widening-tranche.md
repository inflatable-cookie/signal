# 2026-04-08 - g09.002 AU broker-path widening tranche

## Summary

Widened the broker-backed execution path beyond the bounded VST3 branch by
routing AU sandbox ensure and teardown through the same broker process contract
in both hosts.

## Work Completed

- updated `/crates/signal-host-local/src/host_support/sandbox_sessions.rs`
  - AU now uses the broker-backed ensure path when the broker environment is
    enabled
  - AU and VST3 now share the same local broker attach helper
- updated `/crates/signal-host-server/src/host_support/sandbox_sessions.rs`
  - AU now uses the broker-backed ensure path when the broker environment is
    enabled
  - AU and VST3 now share the same server broker attach helper
- updated `/crates/signal-host-local/src/host_api.rs`
  - broker-backed AU ensure now retains the returned broker session for later
    teardown instead of dropping it
- updated `/crates/signal-host-server/src/host.rs`
  - broker-backed AU ensure now retains the returned broker session for later
    teardown instead of dropping it
- expanded `/crates/signal-host-local/tests/public_host_edge_sandbox_broker.rs`
  - added a focused AU broker roundtrip proof alongside the existing VST3 proof
- expanded `/crates/signal-host-server/tests/public_host_edge_sandbox_broker.rs`
  - added the matching AU broker roundtrip proof
- hardened `/crates/signal-host-server/tests/support/public_host_edge_sandbox_broker.rs`
  - serialize broker environment mutation so server broker proof tests do not
    race each other

## Validation

- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`

## Outcome

The broker-backed execution lane now covers:

- local host VST3
- local host AU
- server host VST3
- server host AU

That is a real reduction in the “VST3-only” limitation called out in Batch 2.3.
The queue is still open because server-side LV2 remains outside the broker
lane, and earlier recovery admission/start logic still retains host-specific
control flow even though teardown and several cleanup paths now use the shared
runtime-owned broker helpers.

The next useful tranche should either widen the broker-backed path to LV2 in
`signal-host-server`, or move the shared broker/runtime layer earlier into
replacement-session start failure or overlap-prepare rollback so the shared
ownership story covers more than steady-state and teardown episodes.
