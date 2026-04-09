# 2026-04-08 - g09.002 overlap finish shared detach helper tranche

## Summary

Moved the old-transport teardown fault mapping in overlap-finish recovery out
of duplicated host control flow and into the shared runtime-owned broker helper
layer.

## Work Completed

- extended `/crates/signal-runtime/src/sandbox_broker_support.rs`
  - added a shared helper that interprets overlap old-transport teardown
    outcomes and records the matching detach bookkeeping
- updated `/crates/signal-runtime/src/lib.rs`
  - exported the new overlap-finish detach helper and outcome enum
- rewired `/crates/signal-host-local/src/host_support/recovery_overlap_finish.rs`
  - overlap-finish teardown now delegates runtime-facing old-transport detach
    and fault mapping to the shared helper
- rewired `/crates/signal-host-server/src/host_support/recovery_overlap_finish.rs`
  - matching server overlap-finish teardown now uses the same shared helper
- updated `/docs/roadmaps/g09/002-shared-plugin-hosting-substrate-and-hardened-sandbox-execution.md`
  - added Batch 2.3 Tranche 15 outcome and refreshed the next task
- updated `/docs/roadmaps/g09/README.md`
  - refreshed the generation next task toward broker-backed recovery proof
    widening

## Validation

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `effigy health`

## Outcome

The shared runtime-owned broker layer now covers another of the remaining
overlap recovery seams:

- detach-requested bookkeeping for the retiring transport
- deferred old-transport teardown failure handling
- destroy-region failure handling after ownership has already shifted
- transport teardown failure handling after ownership has already shifted
- successful detach and retired transport-session end

That leaves far less duplicated recovery ownership logic in the hosts. The next
useful tranche should shift from helper extraction to proof coverage: exercise
broker-backed recovery behavior through one or more remaining non-VST3 host
proof lanes so the shared process contract is verified beyond steady-state
ensure.
