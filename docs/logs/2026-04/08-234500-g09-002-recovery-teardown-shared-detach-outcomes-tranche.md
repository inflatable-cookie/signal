# 2026-04-08 - g09.002 recovery teardown shared-detach outcomes tranche

## Summary

Extended the shared broker/runtime helper layer into replacement rollback and
origin-abort teardown, and raised the helper surface from raw transition
records to higher-level detach-failure and detach-completion outcomes.

## Work Completed

- extended `/crates/signal-runtime/src/sandbox_broker_support.rs`
  - added shared broker detach-failure recording that emits broker-failure and
    detach-fault state together
  - added shared detach-completion recording that emits detached,
    transport-torn-down, and transport-session closure together
- updated `/crates/signal-runtime/src/lib.rs`
  - exported the new higher-level shared detach outcome helpers
- rewired `/crates/signal-host-local/src/host_support/recovery_teardown.rs`
  - replacement rollback and origin-abort teardown now use the shared runtime
    detach outcome helpers
- rewired `/crates/signal-host-server/src/host_support/recovery_teardown.rs`
  - the matching server recovery teardown path now uses the same shared helper
    layer
- tightened `/crates/signal-host-local/src/host_support/recovery_cleanup_transport.rs`
  - lingering cleanup now uses the new higher-level detach-failure and
    completion helpers instead of the lower-level pair
- tightened `/crates/signal-host-server/src/host_support/recovery_cleanup_transport.rs`
  - matching server lingering cleanup now uses the same higher-level helpers

## Validation

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `effigy health`

## Outcome

The shared broker/runtime layer now owns a broader slice of recovery transport
truth:

- steady-state broker attach and teardown
- lingering cleanup detach transitions
- overlap-finish teardown transitions
- replacement rollback and origin-abort teardown outcomes

This meaningfully reduces duplicated host teardown bookkeeping, but Batch 2.3
is still open because shared ownership has not yet been pushed into earlier
recovery admission/start failure paths and the broker-backed execution path is
still limited to the bounded VST3 branch.

The next useful tranche should either move the shared layer into
replacement-session start failure or overlap-prepare rollback, or widen the
broker-backed execution path to another plugin format so shared ownership keeps
moving from teardown-only coverage toward full recovery and multi-format use.
