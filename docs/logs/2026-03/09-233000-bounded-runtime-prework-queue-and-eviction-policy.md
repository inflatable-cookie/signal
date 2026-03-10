---
title: bounded runtime prework queue and eviction policy
status: closed
owner: codex
updated: 2026-03-09
tags: [signal, runtime, scheduler, prework]
---

# Summary

Promoted runtime prework from one cached prepared result into a small bounded
queue of future anticipative work, with explicit queue-depth reporting and
oldest-entry eviction when future admissions exceed capacity.

# What changed

- Replaced the single runtime prework slot with a bounded runtime-owned queue.
- Added queue metrics to `RuntimeEngineBlockSnapshot`:
  `prework_cache_queue_capacity`, `prework_cache_queue_depth`, and
  `prework_cache_peak_queue_depth`.
- Updated block execution so runtime can consume matching queued entries from
  the bounded queue instead of only reusing one prepared result.
- Added explicit `QueueCapacityExceeded` invalidation/retirement reporting when
  a new future admission evicts the oldest queued entry.
- Tightened parameter/transport invalidation so runtime only retires
  current-ready queued work on those control updates, leaving later future
  entries available for subsequent matching blocks.

# Validation

- `cargo test -p signal-runtime runtime_prework_queue -- --nocapture`
- `cargo check -p signal-host-local -p signal-host-server -p signal-supervisor-tools`

# Notes

- This batch did not hit any stale Effigy locks.
- The current queue is intentionally small and bounded; it is a scheduler step
  forward, not yet a full background-lane execution worker.
