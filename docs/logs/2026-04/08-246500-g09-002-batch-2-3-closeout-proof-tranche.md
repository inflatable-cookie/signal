# 2026-04-08 - g09.002 Batch 2.3 closeout proof tranche

## Summary

Completed the public proof envelope for Batch 2.3 by adding one broker-backed
cleanup-retry recovery-success lane per host, then closed Batch 2.3 and moved
the remaining work into Batch 2.4.

## Work Completed

- updated `/crates/signal-host-local/tests/public_host_edge_sandbox_broker.rs`
  - added a public broker-backed VST3 deferred-cleanup-retry recovery-success
    proof
  - the new proof verifies the recovered public report returns to
    `AttachActive` while preserving the injected lingering-cleanup retry fault
    in broker failure history
- updated `/crates/signal-host-server/tests/public_host_edge_sandbox_broker.rs`
  - added a matching public broker-backed LV2 deferred-cleanup-retry
    recovery-success proof
- updated `/crates/signal-host-local/tests/support/public_host_edge_sandbox_broker.rs`
  - hardened the broker env lock so poisoned test state no longer cascades into
    follow-on broker tests
- updated `/crates/signal-host-server/tests/support/public_host_edge_sandbox_broker.rs`
  - applied the same poisoned-lock recovery hardening on the server side
- updated `/docs/roadmaps/g09/002-shared-plugin-hosting-substrate-and-hardened-sandbox-execution.md`
  - marked the last Batch 2.3 checklist item complete
  - marked the first two Batch 2.4 integration-proof items complete
  - marked the corresponding acceptance criteria complete
  - added `Batch 2.3 Tranche 19 Outcome`
  - advanced the roadmap next task into Batch 2.4 deferred-gap surfacing
- updated `/docs/roadmaps/g09/README.md`
  - refreshed the generation next task to match the Batch 2.4 start point

## Validation

- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Outcome

Batch 2.3 now has a complete enough repo-owned public proof envelope to close:

- steady-state broker attach and teardown
- successful broker-backed crash restart
- overlap-contention recovery failure
- deferred old-transport teardown failure
- deferred cleanup-retry recovery success

That is enough evidence to stop extending the ownership-hardening queue and
shift attention to Batch 2.4, where the remaining task is to make deferred
coverage gaps explicit in host-facing truth instead of leaving them as implicit
bounded behavior.
