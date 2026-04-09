# 2026-04-08 - g09.002 shared broker helper consolidation tranche

## Summary

Consolidated the duplicated broker-backed sandbox client and receipt-mapping
logic out of `signal-host-local` and `signal-host-server` into a shared runtime
helper surface.

This keeps Batch 2.3 moving in the right direction: both hosts still prove the
same VST3 broker path, but the process client and runtime receipt mapping are
now shared instead of copied.

## Work Completed

- added `/crates/signal-runtime/src/sandbox_broker_support.rs`
  - owns broker process spawning from environment
  - validates startup receipts
  - drives attach and teardown commands
  - parses broker receipt lines
  - records prepared and detached runtime sandbox state through shared helpers
- exported the new shared surface from `/crates/signal-runtime/src/lib.rs`
- rewired `/crates/signal-host-local/src/host_support/sandbox_sessions.rs`
  - now delegates broker client and runtime receipt mapping to
    `signal-runtime`
- rewired `/crates/signal-host-server/src/host_support/sandbox_sessions.rs`
  - now delegates the same broker client and runtime receipt mapping to the
    same `signal-runtime` helper surface

## Validation

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`

## Outcome

The shared broker helper surface is now real and exercised by both hosts.

The next useful tranche is not more consolidation for its own sake. The queue
should spend this helper on a deeper behavior path, most likely one recovery
cleanup / overlap teardown lane or one additional plugin format, so Batch 2.3
starts proving ownership handoff and detach-fault behavior through the shared
broker surface.
