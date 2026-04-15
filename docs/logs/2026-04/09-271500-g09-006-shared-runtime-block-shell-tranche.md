# 2026-04-09 - g09.006 shared runtime block shell tranche

## Summary

Took the next broad `g09.006` consolidation seam by extracting the shared
brokered block shell out of both host `runtime_block.rs` files into the
runtime-owned host unification support layer.

## Changes

- extended
  `~/Dev/projects/signal/crates/signal-runtime/src/host_unification_support.rs`
  with a shared brokered block-execution shell
- changed both hosts so `runtime_block.rs` now supplies only the genuinely
  different phases:
  - request preparation
  - engine completion/application
- kept the duplicated broker dispatch, slot-transition recording, payload
  write/readback, event summary recording, automation summary recording,
  dispatch receipt recording, and watchdog restart handling in one shared
  implementation
- preserved the local-only plugin render/output pump path and the server-only
  synthetic engine-input path as explicit host methods

## Validation

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_drive_broker_backed_vst3_crash_recovery -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_drive_broker_backed_lv2_crash_recovery -- --exact --nocapture --test-threads=1`
- `effigy health`

## Outcome

The shared execution substrate now covers:

- `runtime_cycle.rs`
- `boot_recovery_helpers.rs`
- the brokered block shell in `runtime_block.rs`

That materially narrows the remaining duplication. The biggest broad seam left
for `g09.006` is `sandbox_sessions.rs`, where shared broker session and
transport orchestration still sit inside parallel host wrappers around smaller
format and environment differences.

## Next Task

Continue `g09.006` with one broad consolidation batch on the next
highest-leverage seam: consolidate the shared broker session and transport
orchestration now concentrated in `sandbox_sessions.rs`, starting with the
common attach/record/teardown shell while keeping format-specific preparation
and host-edge environment differences explicit, then rerun focused local/server
broker recovery proofs.
