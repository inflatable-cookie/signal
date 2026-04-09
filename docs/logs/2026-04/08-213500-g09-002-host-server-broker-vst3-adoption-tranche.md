# 2026-04-08 - g09.002 host-server broker VST3 adoption tranche

## Summary

Widened the first broker-backed sandbox host path from `signal-host-local` into
`signal-host-server`.

The same typed broker contract now powers one bounded VST3 ensure/teardown path
in both hosts, giving Batch 2.3 a real cross-host adoption seam instead of a
single host-local experiment.

## Work Completed

- rewired `/crates/signal-host-server/src/host_support/sandbox_sessions.rs`
  - added an opt-in broker-backed VST3 ensure path gated by
    `SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND`
  - added broker receipt parsing and process management for the server host
  - recorded runtime-owned attached, detached, transport-torn-down, and
    instance-destroyed state from broker outcomes
- updated `/crates/signal-host-server/src/host_support.rs`
  - reexport the new server-side broker session management surface
- updated `/crates/signal-host-server/src/host.rs`
  - store live broker-backed sandbox sessions in the host
  - drive broker teardown during `teardown_plugin_sandbox(...)`
- added `/crates/signal-host-server/tests/support/public_host_edge_sandbox_broker.rs`
  - scoped broker environment setup for integration tests
- added `/crates/signal-host-server/tests/public_host_edge_sandbox_broker.rs`
  - prove server-side VST3 scan, ensure-through-broker, and teardown-through-
    broker behavior

## Validation

- `cargo check -p signal-host-server`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`

## Outcome

Batch 2.3 now has a real broker-backed VST3 path in both hosts.

The next useful step is not another host-specific copy. The duplication between
local and server broker attach/teardown handling is now obvious, so the queue
should extract a shared helper surface and then route either another plugin
format or one recovery cleanup path through that shared layer.
