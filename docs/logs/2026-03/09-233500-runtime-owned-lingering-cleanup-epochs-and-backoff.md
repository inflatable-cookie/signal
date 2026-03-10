---
title: runtime owned lingering cleanup epochs and backoff
status: completed
owner: codex
updated: 2026-03-09
tags: [signal, runtime, recovery, lingering, scheduling]
---

## Summary

Added runtime-owned cleanup epochs and delayed retry readiness to the lingering
cleanup work queue.

## Changes

- Extended `LingeringCleanupPlan` with `cleanup_epoch` and
  `ready_at_processing_epoch`.
- Extended runtime transport concurrency snapshots with:
  `pending_deferred_retry_work_items`, `next_cleanup_epoch`, and
  `oldest_pending_cleanup_ready_epoch`.
- Added runtime-owned cleanup-epoch allocation for queued cleanup work.
- Changed deferred retry scheduling so `DeferredRetry` work is only drainable
  once its `ready_at_processing_epoch` has been reached.
- Updated runtime tests to pin cleanup epoch allocation, pending deferred retry
  visibility, and deferred dequeue timing.
- Kept host cleanup execution unchanged except for respecting runtime dequeue
  readiness when draining queued cleanup work.

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server`
- `cargo test -p signal-runtime runtime_orders_lingering_cleanup_candidates_by_provenance_then_attach_sequence -- --nocapture`
- `cargo test -p signal-host-local late_lingering -- --nocapture`
- `cargo test -p signal-host-server late_lingering -- --nocapture`
- `cargo test -p signal-host-local adjacent_overlap -- --nocapture`
- `cargo test -p signal-host-server adjacent_overlap -- --nocapture`
- `git diff --check`

## Next Task

Push lingering-session lifecycle management further into runtime state,
especially if the engine needs sandbox-scoped cleanup epochs or richer
late-detach reconciliation state machines, so runtime can distinguish one
cleanup wave from the next instead of only scheduling due work by processing
epoch and retry count.
