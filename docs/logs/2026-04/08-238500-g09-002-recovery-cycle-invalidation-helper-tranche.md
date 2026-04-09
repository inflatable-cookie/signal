# 2026-04-08 - g09.002 recovery-cycle invalidation helper tranche

## Summary

Moved the duplicated recovery-cycle and invalidation setup block out of the
host recovery entrypoints and into the shared runtime-owned broker helper
surface.

## Work Completed

- extended `/crates/signal-runtime/src/sandbox_broker_support.rs`
  - added a shared helper that begins a brokered recovery cycle
  - the helper records the recovery cycle, completion-slot invalidation, and
    broker invalidation receipts from one runtime-owned path
- updated `/crates/signal-runtime/src/lib.rs`
  - exported the new recovery-cycle helper
- rewired `/crates/signal-host-local/src/host_support/recovery_sandbox.rs`
  - `recover_sandbox(...)` now delegates recovery-cycle and invalidation setup
    to the shared runtime helper
- rewired `/crates/signal-host-server/src/host_support/recovery_sandbox.rs`
  - the matching server recovery entrypoint now uses the same shared helper

## Validation

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `effigy health`

## Outcome

The shared runtime-owned broker layer now covers another earlier recovery step:

- recovery-cycle recording
- completion-slot invalidation bookkeeping
- broker invalidation bookkeeping

That leaves less host-specific logic in the `recover_sandbox(...)` entrypoints
and moves the remaining duplication further up toward contention and rollback
control flow rather than transport/session truth.

The next useful tranche should target the remaining overlap-prepare contention
handling or replacement-start rollback choreography so the hosts stop
duplicating the same recovery-control decisions once the underlying runtime
state transitions are already shared.
