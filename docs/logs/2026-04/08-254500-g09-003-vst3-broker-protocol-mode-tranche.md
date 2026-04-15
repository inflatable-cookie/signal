# 2026-04-08 - g09.003 VST3 Broker Protocol Mode Tranche

## Summary

Moved the VST3 lifecycle/state detail origin from host-appended summaries into
the sandbox broker protocol itself.

## Landed

- added explicit VST3 broker commands in
  `~/Dev/projects/signal/crates/signal-plugin-sandbox/src/broker.rs`
  for `attach-vst3`, `run-vst3`, `run-timeout-vst3`, and `teardown-vst3`
- made broker receipts carry VST3-flavored state-store, activation, and
  flushed-state markers directly
- introduced typed broker flavor selection in
  `~/Dev/projects/signal/crates/signal-runtime/src/sandbox_broker_support.rs`
- rewired both host VST3 broker paths to select the VST3 broker flavor instead
  of appending host-side lifecycle summaries onto generic broker transport
  detail
- updated the broker-backed public VST3 proof lanes to assert broker-originated
  VST3 markers

## Validation

- `cargo check -p signal-plugin-sandbox`
- `cargo test -p signal-plugin-sandbox`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Remaining Gap

The broker protocol is now VST3-aware, but the broker execution core is still
bounded demo transport underneath. The next tranche should replace that
remaining CLAP/demo processing core with a real VST3-oriented broker execution
path.
