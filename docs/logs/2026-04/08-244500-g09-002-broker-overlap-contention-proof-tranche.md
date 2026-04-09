# 2026-04-08 - g09.002 broker overlap contention proof tranche

## Summary

Broadened Batch 2.3 from broker-backed crash-recovery proof into the first
broker-backed recovery-failure proof on the public host edge.

## Work Completed

- updated `/crates/signal-host-local/tests/public_host_edge_sandbox_broker.rs`
  - added a public broker-backed VST3 overlap-contention proof
  - the new proof drives `boot_with_recovery_overlap_contention()` under the
    broker demo-plugin override surface and asserts the public supervisor
    report exposes the expected overlap rejection and stop-state truth
- updated `/crates/signal-host-server/tests/public_host_edge_sandbox_broker.rs`
  - added a public broker-backed LV2 overlap-contention proof
  - the new proof verifies the same recovery-abort truth through the server
    host public edge
- updated `/docs/roadmaps/g09/002-shared-plugin-hosting-substrate-and-hardened-sandbox-execution.md`
  - added `Batch 2.3 Tranche 17 Outcome`
  - refreshed the roadmap next task toward deferred teardown or cleanup-retry
    proof
- updated `/docs/roadmaps/g09/README.md`
  - refreshed the generation next task to match the new Batch 2.3 proof target

## Validation

- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Outcome

Batch 2.3 now has repo-owned proof for both sides of the shared broker-backed
recovery contract:

- one successful crash-restart lane per host
- one overlap-contention failure lane per host

That means the queue is no longer proving only broker steady state and happy
restart. The next useful tranche should push into teardown-stage ownership
truth, most likely deferred old-transport teardown or deferred cleanup retry,
so Batch 2.3 can close on verified recovery semantics rather than only on
helper extraction and happy-path proof.
