# 2026-04-08 - g09.002 recovery cleanup detach shared-helper tranche

## Summary

Extended the new shared broker/runtime helper layer into the first real
ownership-handoff cleanup path.

The lingering transport cleanup logic in both hosts now uses shared runtime
helpers for detach-requested, detach-fault, and detached-plus-torn-down
recording instead of hand-rolling those transitions independently.

## Work Completed

- extended `/crates/signal-runtime/src/sandbox_broker_support.rs`
  - added shared detach-requested recording
  - added shared detach-fault recording
  - generalized detached recording so callers can choose whether instance
    destruction should also be recorded
- updated `/crates/signal-runtime/src/lib.rs`
  - exported the new shared cleanup recording helpers
- rewired `/crates/signal-host-local/src/host_support/recovery_cleanup_transport.rs`
  - now uses the shared runtime helper for detach state transitions
- rewired `/crates/signal-host-server/src/host_support/recovery_cleanup_transport.rs`
  - now uses the same shared runtime helper for the matching server cleanup
    path
- hardened `/crates/signal-host-local/tests/support/public_host_edge_sandbox_broker.rs`
  - serialize broker environment mutation so the local proof surface does not
    race itself

## Validation

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`

## Outcome

The shared broker layer now covers both steady-state broker attach/teardown and
the first lingering cleanup ownership-handoff seam.

The next useful tranche should push the same layer into a full recovery episode
such as overlap restart or lingering session recovery, or widen it to another
plugin format so the queue keeps reducing host-specific sandbox control logic
instead of only centralizing bookkeeping.
