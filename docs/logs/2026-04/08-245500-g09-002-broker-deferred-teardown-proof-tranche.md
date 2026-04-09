# 2026-04-08 - g09.002 broker deferred teardown proof tranche

## Summary

Broadened Batch 2.3 from broker-backed overlap-contention proof into teardown-
stage ownership proof on the public host edge.

## Work Completed

- updated `/crates/signal-host-local/tests/public_host_edge_sandbox_broker.rs`
  - added a public broker-backed VST3 deferred-teardown recovery-failure proof
  - the new proof drives `boot_with_recovery_deferred_teardown_failure()` under
    the broker demo-plugin override surface and asserts the exported public
    report exposes the expected lingering and detach-fault truth
- updated `/crates/signal-host-server/tests/public_host_edge_sandbox_broker.rs`
  - added a matching public broker-backed LV2 deferred-teardown recovery-
    failure proof
  - the new proof verifies the same teardown-stage ownership failure shape
    through the server host public edge
- updated `/docs/roadmaps/g09/002-shared-plugin-hosting-substrate-and-hardened-sandbox-execution.md`
  - added `Batch 2.3 Tranche 18 Outcome`
  - refreshed the roadmap next task toward an explicit Batch 2.3 closeout
    decision
- updated `/docs/roadmaps/g09/README.md`
  - refreshed the generation next task to match the same closeout threshold

## Validation

- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Outcome

Batch 2.3 now has public broker-backed proof for both the admission and
teardown sides of recovery failure:

- overlap-contention abort
- deferred old-transport teardown abort

Alongside the existing steady-state attach/teardown and crash-restart lanes,
that gives the queue a much more complete public proof envelope. The next
useful step should either add one final success-side lingering cleanup proof or
make the closeout call and move any remaining deeper recovery proof into Batch
2.4 instead of stretching Batch 2.3 indefinitely.
