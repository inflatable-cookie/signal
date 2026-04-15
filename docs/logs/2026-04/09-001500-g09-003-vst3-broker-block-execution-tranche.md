# 2026-04-09 - g09.003 VST3 Broker Block Execution Tranche

## Summary

Introduced the first bounded adapter-owned VST3 block execution receipt behind
the broker-prepared VST3 session.

## Landed

- added `Vst3BlockProcessingRecord` plus `execute_block(...)` in
  `~/Dev/projects/signal/crates/signal-plugin-vst3/src/vst3_host_adapter/model.rs`
  and
  `~/Dev/projects/signal/crates/signal-plugin-vst3/src/vst3_host_adapter/session.rs`
- extended the VST3 adapter proof in
  `~/Dev/projects/signal/crates/signal-plugin-vst3/src/lib.rs`
  to validate bounded block execution over a real discovered temp bundle
- updated
  `~/Dev/projects/signal/crates/signal-plugin-sandbox/src/broker.rs`
  so the broker-owned VST3 execution state retains the real instantiated
  control surface, prepared session plan, and state snapshot, and `run-vst3`
  now calls the adapter `execute_block(...)` path rather than reporting only
  prepared-session truth with a synthetic counter

## Validation

- `cargo check -p signal-plugin-vst3`
- `cargo test -p signal-plugin-vst3 --lib`
- `cargo test -p signal-plugin-sandbox`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Remaining Gap

The broker-backed VST3 lane now emits a real adapter-owned execution receipt
for one bounded block path, but execution is still minimal. The next tranche
should widen that into a short execution stream and promote at least one of
those execution receipts across the broker boundary into host-facing proof.
