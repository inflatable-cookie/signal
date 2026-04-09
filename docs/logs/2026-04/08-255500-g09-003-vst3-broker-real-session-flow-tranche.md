# 2026-04-08 - g09.003 VST3 Broker Real Session Flow Tranche

## Summary

Replaced the remaining CLAP-lifecycle dependency under the VST3 broker mode
with a real VST3-oriented broker session flow.

## Landed

- added per-process broker spawn config in
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-runtime/src/sandbox_broker_support.rs`
  so host-side VST3 broker sessions can pass `plugin_type_id`, `module_root`,
  and `instance_id` into the spawned broker without global env mutation
- rewired local and server VST3 broker ensure paths in
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-host-local/src/host_support/sandbox_sessions.rs`
  and
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-host-server/src/host_support/sandbox_sessions.rs`
  to use that per-process VST3 spawn context
- updated
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-plugin-sandbox/src/broker.rs`
  so VST3 mode now:
  - resolves a real discovered bundle through `signal-plugin-vst3`
  - instantiates the VST3 plugin
  - stores state
  - activates it
  - builds a real prepared session plan
  - emits teardown truth from the real VST3 teardown record
- hardened broker shutdown so a broker process that has already completed cleanly
  during recovery teardown does not surface a misleading stdout EOF

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

The VST3 broker path now owns real discovery, instantiation, state, activation,
session planning, and teardown. The remaining depth in `g09.003` is actual
bounded block execution behind that prepared VST3 broker session.
