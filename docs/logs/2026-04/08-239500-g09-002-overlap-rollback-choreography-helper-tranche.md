# 2026-04-08 - g09.002 overlap rollback choreography helper tranche

## Summary

Moved the remaining duplicated overlap contention and replacement-start outcome
interpretation out of the host recovery branches and into the shared
runtime-owned broker helper surface.

## Work Completed

- extended `/crates/signal-runtime/src/sandbox_broker_support.rs`
  - reshaped overlap contention handling into a shared value-driven helper
  - reshaped replacement restart vs injected replacement-start failure vs
    runtime restart handling into a shared value-driven helper
- updated `/crates/signal-host-local/src/host_support/recovery_overlap_prepare.rs`
  - overlap-prepare contention now performs host-specific lifecycle work first
    and then delegates shared contention interpretation to the runtime helper
- updated `/crates/signal-host-server/src/host_support/recovery_overlap_prepare.rs`
  - matching server overlap-prepare contention now uses the same shared helper
    sequencing
- updated `/crates/signal-host-local/src/host_support/recovery_overlap_restart.rs`
  - replacement restart and replacement-start failure handling now delegate the
    shared outcome sequencing to the runtime helper before applying rollback or
    successful transport promotion
- updated `/crates/signal-host-server/src/host_support/recovery_overlap_restart.rs`
  - matching server overlap restart now uses the same shared helper sequencing
- updated `/docs/roadmaps/g09/002-shared-plugin-hosting-substrate-and-hardened-sandbox-execution.md`
  - added Batch 2.3 Tranche 12 outcome and refreshed the next task
- updated `/docs/roadmaps/g09/README.md`
  - refreshed the generation next task to target the remaining rollback
    wrapper entrypoints

## Validation

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `effigy health`

## Outcome

The shared runtime-owned broker layer now covers another important part of the
recovery stack:

- recovery-cycle invalidation setup
- overlap-state entry, rollback, and successful restart
- overlap-prepare contention interpretation
- replacement restart vs injected replacement-start vs runtime restart outcome
  interpretation

That leaves the remaining host-specific recovery duplication narrower and
higher-level. The next useful tranche should target one of the last rollback
wrapper entrypoints, such as abort-origin recovery teardown or lingering
session restart failure, so the hosts stop carrying parallel recovery wrapper
control flow once the underlying broker/runtime decisions are already shared.
