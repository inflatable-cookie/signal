# 2026-04-08 - g09.002 recovery teardown detach helper tranche

## Summary

Moved the abort-origin and replacement-rollback detach bookkeeping out of the
host recovery wrappers and into the shared runtime-owned broker helper layer.

## Work Completed

- extended `/crates/signal-runtime/src/sandbox_broker_support.rs`
  - added a shared helper that finalizes brokered recovery transport detach
    from destroy-region and teardown-active-transport outcomes
- updated `/crates/signal-runtime/src/lib.rs`
  - exported the new shared recovery-detach helper
- rewired `/crates/signal-host-local/src/host_support/recovery_teardown.rs`
  - collapsed abort-origin and replacement-rollback teardown onto one local
    wrapper that delegates detach outcome mapping to the runtime helper
- rewired `/crates/signal-host-server/src/host_support/recovery_teardown.rs`
  - matching server teardown now uses the same local wrapper plus shared
    runtime-owned detach helper
- updated `/docs/roadmaps/g09/002-shared-plugin-hosting-substrate-and-hardened-sandbox-execution.md`
  - added Batch 2.3 Tranche 13 outcome and refreshed the next task
- updated `/docs/roadmaps/g09/README.md`
  - refreshed the generation next task toward lingering-session restart failure
    and late-start rollback

## Validation

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `effigy health`

## Outcome

The shared runtime-owned broker layer now covers another one of the last
higher-level recovery wrapper seams:

- detach-requested bookkeeping for rollback teardown
- transport destroy failure bookkeeping for rollback teardown
- transport teardown failure bookkeeping for rollback teardown
- successful detach and transport-session end for rollback teardown

The host crates still own the format-local lifecycle teardown loop and the
decision about whether the origin sandbox should be torn down first, but they
no longer duplicate the detach outcome mapping themselves across both hosts and
both rollback entrypoints.

The next useful tranche should target lingering-session restart failure and
late-start rollback handling so the hosts stop duplicating the final restart
wrapper choreography after overlap contention, invalidation, and teardown truth
are already shared.
