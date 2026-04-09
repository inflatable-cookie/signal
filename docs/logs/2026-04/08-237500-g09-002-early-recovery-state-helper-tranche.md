# 2026-04-08 - g09.002 early recovery-state helper tranche

## Summary

Pushed the shared runtime-owned broker layer into earlier recovery state
transitions instead of keeping it limited to steady-state attach/teardown and
late cleanup paths.

## Work Completed

- extended `/crates/signal-runtime/src/sandbox_broker_support.rs`
  - added a shared helper for entering recovery overlap
  - added a shared helper for rolling recovery overlap back to zero
  - added a shared helper for completing overlap restart and promoting a
    recovered transport session back to steady state when one exists
- updated `/crates/signal-runtime/src/lib.rs`
  - exported the new early recovery-state helpers
- rewired `/crates/signal-host-local/src/host_support/recovery_overlap_prepare.rs`
  - overlap preparation now uses the shared runtime helper to enter
    multi-sandbox overlap state
- rewired `/crates/signal-host-server/src/host_support/recovery_overlap_prepare.rs`
  - matching server overlap preparation now uses the same shared helper
- rewired `/crates/signal-host-local/src/host_support/recovery_overlap_restart.rs`
  - replacement restart rollback and successful overlap restart now use the
    shared runtime-owned overlap state helpers
- rewired `/crates/signal-host-server/src/host_support/recovery_overlap_restart.rs`
  - matching server overlap restart now uses the same shared helper layer
- rewired `/crates/signal-host-local/src/host_support/recovery_runtime.rs`
  - lingering-session restart now uses the shared recovery state helpers
- rewired `/crates/signal-host-server/src/host_support/recovery_runtime.rs`
  - matching server lingering-session restart now uses the same shared helpers
  - this also restores steady-state transport promotion on successful recovery
    to match the local host behavior

## Validation

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `effigy health`

## Outcome

The shared runtime-owned broker layer now covers:

- steady-state broker attach and teardown
- lingering cleanup and overlap-finish teardown
- rollback/origin-abort teardown outcomes
- early overlap-state entry, rollback, and successful recovery restart

This is the first tranche where the queue moved meaningfully above teardown
bookkeeping and into earlier recovery-control state. The next useful tranche
should target the remaining duplicated rollback choreography in overlap-prepare
contention handling or replacement-start failure so the hosts stop duplicating
the same recovery control flow after the underlying broker/runtime state
transitions are already shared.
