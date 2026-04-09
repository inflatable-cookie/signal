# 2026-04-08 - g09.003 VST3 instantiation contract tranche

## Summary

Started `g09.003` Batch 3.2 by making VST3 instantiation validate and bind
against the real discovered bundle/module/factory contract instead of cloning
scaffold assumptions from the discovery record.

## Work Completed

- updated `/crates/signal-plugin-vst3/src/vst3_host_adapter/model.rs`
  - added component/controller instance truth fields to
    `Vst3InstanceControlSurface` and `Vst3ProcessSessionPlan`
- updated `/crates/signal-plugin-vst3/src/vst3_host_adapter/session.rs`
  - `instantiate_plugin(...)` is now fallible
  - the adapter now rereads bundle-local module and factory metadata during
    instantiation
  - instantiation now rejects drift between the discovered plugin record and
    the live bundle contract
  - prepared session summaries now expose component name, controller name, and
    factory export count
- updated `/crates/signal-host-local/src/host_support/sandbox_sessions.rs`
  and `/crates/signal-host-server/src/host_support/sandbox_sessions.rs`
  - VST3 host ensure paths now map fallible VST3 instantiation into explicit
    runtime request errors instead of assuming instantiation always succeeds
- updated `/crates/signal-plugin-vst3/src/lib.rs`
  - adapter tests now instantiate from a real discovered temp bundle instead of
    the old scaffold-only `discover_plugin_type(...)` shortcut
- updated `/docs/roadmaps/g09/003-real-vst3-discovery-instantiation-and-lifecycle-proof.md`
  - added `Batch 3.2 Tranche 1 Outcome`
  - advanced the next task into lifecycle/state execution depth

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

The VST3 path now has a real instantiation contract layered on top of the real
bundle discovery contract. The next remaining `g09.003` depth is lifecycle and
state execution, not whether the adapter can reopen and validate a discovered
component/controller pair.
