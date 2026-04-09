# 2026-04-09 - g09.006 shared cycle and watchdog helper tranche

## Summary

Started `g09.006` by inventorying the duplicated host execution and recovery
seams, then extracting the first shared support layer out of the highest-value
identical code.

## Changes

- activated `g09.006` in the roadmap surfaces
- added a runtime-owned shared support module at
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-runtime/src/host_unification_support.rs`
- moved the shared recovery plan DTOs into `signal-runtime`:
  - `RepeatedWatchdogRecoveryPlan`
  - `TimeoutRecoveryRetryPlan<'a, Failure>`
- replaced the duplicated host `runtime_cycle.rs` bodies with one shared
  macro-backed implementation layer
- replaced the duplicated host `boot_recovery_helpers.rs` bodies with one
  shared macro-backed implementation layer
- kept host-specific sandbox assembly types, fault-envelope instance ids, and
  host-only behavior explicit at the call site instead of flattening them into
  runtime

## Validation

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_drive_broker_backed_vst3_crash_recovery -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_drive_broker_backed_lv2_crash_recovery -- --exact --nocapture --test-threads=1`
- `effigy health`

## Outcome

The milestone is no longer only an audit note. The most obviously duplicated
execution-cycle and watchdog-retry policy is now shared, and the inventory for
the next pass is clearer: `runtime_block.rs` and `sandbox_sessions.rs` are the
remaining large seams where shared broker and recovery policy still sit inside
parallel host-specific wrappers.

## Next Task

Continue `g09.006` with one broad consolidation batch on the next
highest-leverage seam: extract the shared brokered block shell from
`runtime_block.rs` into the same runtime-owned support layer while keeping the
local host’s plugin-render/output-pump path and the server host’s synthetic
engine-input path explicit at the edges, then rerun focused local/server
recovery proofs.
