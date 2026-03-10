---
title: runtime owned lingering cleanup work queue
status: completed
owner: codex
updated: 2026-03-09
tags: [signal, runtime, recovery, lingering, transport]
---

## Summary

Moved lingering cleanup retries and cleanup issuance timing into a runtime-owned
work queue.

## Changes

- Added `LingeringCleanupTrigger` and expanded `LingeringCleanupPlan` with
  `work_id`, `trigger`, and `retry_count`.
- Added runtime-owned pending cleanup work in transport concurrency state.
- Replaced direct cleanup-plan generation with queue operations:
  `enqueue_lingering_cleanup_work(...)` and
  `dequeue_lingering_cleanup_work_for_sandbox(...)`.
- Added automatic `DeferredRetry` work issuance when best-effort post-start
  lingering cleanup fails.
- Added `pending_cleanup_work_items` to
  `RuntimeTransportConcurrencySnapshot` and the shared JSON/report surface.
- Switched local/server hosts to enqueue cleanup work by trigger and drain
  runtime-issued work items instead of deciding retry timing entirely in host
  code.

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
especially if the engine needs late-detach reconciliation and deferred cleanup
retries to graduate from queued runtime work into richer runtime-owned cleanup
state transitions, such as work aging, retry backoff, or sandbox-scoped cleanup
epochs, instead of remaining immediate host-driven execution of dequeued work.
