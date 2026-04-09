# 2026-04-08 - g09.002 recovery overlap-finish shared-helper tranche

## Summary

Pushed the shared broker/runtime helper layer from lingering cleanup tails into
the first fuller overlap-recovery ownership-handoff path.

The overlap-finish teardown logic in both hosts now uses shared runtime helpers
for detach-requested, detach-fault, and detached-plus-transport-torn-down
recording instead of hand-rolling those transitions independently inside the
overlap recovery episode.

## Work Completed

- updated `/crates/signal-runtime/src/sandbox_broker_support.rs`
  - reused the shared detach-requested recording for overlap-finish teardown
  - reused the shared detach-fault recording for deferred teardown, destroy,
    injected old-transport cleanup, and CLAP transport teardown failures
  - reused the shared detached-plus-transport-torn-down recording after
    successful overlap cleanup
- rewired `/crates/signal-host-local/src/host_support/recovery_overlap_finish.rs`
  - now routes overlap-finish detach state transitions through the shared
    runtime helper layer
- rewired `/crates/signal-host-server/src/host_support/recovery_overlap_finish.rs`
  - now uses the same shared runtime helper for the matching server overlap
    recovery path
- cleaned stale imports exposed by the overlap-finish refactor

## Validation

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `effigy health`

Focused overlap lib-test execution remains blocked by pre-existing unresolved
split-test module paths in the host lib-test trees. Those failures were already
present before this tranche and were not widened here.

## Outcome

The shared broker/runtime layer now covers:

- steady-state broker attach and teardown
- lingering cleanup detach-requested and detach-fault transitions
- overlap-finish ownership handoff and teardown transitions

That is a meaningful expansion from bookkeeping-only reuse into a fuller
recovery episode, but Batch 2.3 still remains open because the shared layer is
not yet the default cross-format sandbox substrate and additional recovery
entrypoints still retain host-specific control flow.

The next useful tranche should either widen the shared broker path to another
plugin format or push the same runtime-owned layer into one more recovery path
such as lingering rollback or replacement-session start failure so ownership
handoff stops being split across shared and host-local logic.
