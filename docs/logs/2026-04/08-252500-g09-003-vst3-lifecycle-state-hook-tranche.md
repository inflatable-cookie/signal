# 2026-04-08 - g09.003 VST3 lifecycle and state hook tranche

## Summary

Completed the next `g09.003` batch by adding bounded VST3 activation, state,
and teardown hooks behind the real discovery and instantiation contract.

## Work Completed

- updated `/crates/signal-plugin-vst3/src/vst3_host_adapter/model.rs`
  - added `Vst3ActivationRecord`, `Vst3StateSnapshot`, and
    `Vst3TeardownRecord`
- updated `/crates/signal-plugin-vst3/src/vst3_host_adapter/session.rs`
  - added bounded `load_state_snapshot(...)`, `store_state_snapshot(...)`,
    `activate_instance(...)`, and `teardown_instance(...)` hooks
  - each hook revalidates the instantiated bundle/module/factory contract
  - prepared-session and lifecycle surfaces now carry component/controller
    truth instead of only generic summaries
- updated `/crates/signal-host-local/src/host_support/sandbox_sessions.rs`
  and `/crates/signal-host-server/src/host_support/sandbox_sessions.rs`
  - the non-broker VST3 ensure path now stores state and activates the VST3
    instance before recording prepared sandbox truth
  - prepared summaries now include VST3 state and activation details
- updated `/crates/signal-plugin-vst3/src/lib.rs`
  - extended the VST3 adapter test to cover store/load/activate/teardown over a
    real discovered temp bundle
- updated `/docs/roadmaps/g09/003-real-vst3-discovery-instantiation-and-lifecycle-proof.md`
  - added `Batch 3.2 Tranche 2 Outcome`
  - advanced the next task into the broker-backed VST3 lifecycle path

## Validation

- `cargo check -p signal-plugin-vst3`
- `cargo test -p signal-plugin-vst3 --lib`
- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_vst3`
- `cargo test -p signal-host-server --test public_host_edge_vst3`
- `effigy health`

## Outcome

The VST3 path now has bounded lifecycle and state hooks layered on top of the
real discovery and instantiation contract. The next remaining `g09.003` depth
is to route that lifecycle/state truth through the shared sandbox broker path
and runtime-owned receipts rather than leaving it adapter-local.
