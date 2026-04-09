# 2026-04-08 - g09.002 lingering restart shared helper tranche

## Summary

Moved the lingering-session restart and late-start rollback sequencing out of
duplicated host recovery control flow and into the shared runtime-owned broker
helper layer.

## Work Completed

- extended `/crates/signal-runtime/src/sandbox_broker_support.rs`
  - added a shared helper that sequences lingering-session restart success and
    late-start rollback
- updated `/crates/signal-runtime/src/lib.rs`
  - exported the new lingering-session restart helper
- rewired `/crates/signal-host-local/src/host_support/recovery_runtime.rs`
  - lingering-session recovery now delegates restart outcome sequencing and
    late-start rollback state handling to the shared runtime helper
- rewired `/crates/signal-host-server/src/host_support/recovery_runtime.rs`
  - matching server lingering-session recovery now uses the same shared helper
- updated `/docs/roadmaps/g09/002-shared-plugin-hosting-substrate-and-hardened-sandbox-execution.md`
  - added Batch 2.3 Tranche 14 outcome and refreshed the next task
- updated `/docs/roadmaps/g09/README.md`
  - refreshed the generation next task toward the remaining ownership hardening
    seam

## Validation

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `effigy health`

## Outcome

The shared runtime-owned broker layer now covers another one of the last
higher-level recovery wrapper seams:

- restart failure interpretation for lingering-session recovery
- pre-start overlap promotion back to one active sandbox
- recovered transport promotion back to steady state
- late-start rollback back to zero active overlap state

The host crates still own lingering cleanup, lifecycle rerun, and rollback
teardown itself, but they no longer duplicate the same restart-wrapper state
sequencing in parallel lingering-session recovery entrypoints.

The next useful tranche should either push the shared broker helper layer into
`recovery_overlap_finish.rs` old-transport teardown fault handling or widen the
broker-backed recovery proofs so the shared process contract is exercised
beyond the steady-state ensure path.
