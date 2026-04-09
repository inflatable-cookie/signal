# 2026-04-08 - g09.003 VST3 broker lifecycle summary tranche

## Summary

Completed the next `g09.003` batch by routing the bounded VST3 lifecycle/state
hooks through the shared broker-backed host path and proving that the broker
VST3 lanes now expose VST3-specific lifecycle truth.

## Work Completed

- updated `/crates/signal-runtime/src/sandbox_broker_support.rs`
  - extended `SandboxBrokerSession` so broker-backed sessions can carry
    adapter-produced prepared and teardown summaries
- updated `/crates/signal-host-local/src/host_support/sandbox_sessions.rs`
  and `/crates/signal-host-server/src/host_support/sandbox_sessions.rs`
  - broker-backed VST3 ensure now instantiates the discovered bundle,
    stores state, activates the instance, and passes that lifecycle detail into
    the shared broker session summary
  - broker-backed VST3 teardown now appends bounded VST3 teardown detail to the
    shared detach outcome
- updated `/crates/signal-host-local/tests/public_host_edge_sandbox_broker.rs`
  and `/crates/signal-host-server/tests/public_host_edge_sandbox_broker.rs`
  - strengthened the broker-backed VST3 public proofs to assert state and
    activation detail is visible in the exported host report
- updated `/docs/roadmaps/g09/003-real-vst3-discovery-instantiation-and-lifecycle-proof.md`
  - added `Batch 3.2 Tranche 3 Outcome`
  - advanced the next task from adapter/host integration into broker execution
    depth

## Validation

- `cargo check -p signal-plugin-vst3`
- `cargo test -p signal-plugin-vst3 --lib`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`

## Outcome

The broker-backed VST3 host path now exposes VST3-specific lifecycle/state
detail instead of only generic broker transport truth. The remaining `g09.003`
gap is the broker execution core itself, which still runs a demo CLAP-oriented
path instead of a real VST3 execution surface.
