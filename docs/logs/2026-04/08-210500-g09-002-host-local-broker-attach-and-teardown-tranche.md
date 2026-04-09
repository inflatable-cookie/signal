# 2026-04-08 - g09.002 host-local broker attach and teardown tranche

## Summary

Started the first real host-owned adoption path for the new sandbox broker in
`g09.002` Batch 2.3.

`signal-plugin-sandbox` now supports explicit attached-session lifecycle
commands, and `signal-host-local` can opt one VST3 sandbox path into that
broker process so runtime lifecycle and transport receipts come from a real
cross-process attach/teardown flow instead of only from in-process prepare
sessions.

## Work Completed

- extended `/crates/signal-plugin-sandbox/src/broker.rs`
  - added `attach-demo` and `teardown-demo` commands
  - preserved typed receipt states while making lease attachment and cleanup
    explicit for long-lived attached sessions
  - kept `run-demo` and `run-timeout-demo` working on top of the same attach
    and teardown path
- rewired `/crates/signal-host-local/src/host_support/sandbox_sessions.rs`
  - added an opt-in broker-backed VST3 ensure path gated by
    `SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND`
  - introduced broker receipt parsing and process management
  - recorded runtime-owned attach, detach, transport torn-down, and instance
    destroyed lifecycle events from broker outcomes
- updated `/crates/signal-host-local/src/host.rs`
  - store live broker-backed sandbox sessions in the host
- updated `/crates/signal-host-local/src/host_api.rs`
  - retain broker sessions returned from ensure
  - drive broker teardown on `teardown_plugin_sandbox(...)`
- expanded `/crates/signal-host-local/tests/public_host_edge_sandbox_broker.rs`
  - kept raw broker receipt proof coverage
  - added one end-to-end host-local VST3 scan, ensure-through-broker, and
    teardown-through-broker proof
- expanded `/crates/signal-host-local/tests/support/public_host_edge_sandbox_broker.rs`
  - added scoped environment setup for broker command, args, and workdir

## Validation

- `cargo test -p signal-plugin-sandbox`
- `cargo check -p signal-host-local`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `effigy health`

## Outcome

Batch 2.3 is still not complete, but it is no longer broker-local only.

The repo now proves one actual host-owned process boundary for plugin sandbox
attach and teardown. The next meaningful step is to widen that seam into
shared host/runtime cleanup behavior or a second host surface, rather than
adding more isolated broker commands.
