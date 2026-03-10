---
title: Runtime Owned Prework Cache Validity Window
status: done
owner: codex
updated: 2026-03-09
tags: [signal, runtime, graph, execution, anticipative, cache]
---

## Summary

Extended the new anticipative prework handoff into a runtime-owned reusable
engine state.

`signal-runtime` now:

- computes a stable input signature for engine blocks
- caches prepared anticipative work when the graph exposes an anticipative
  dispatch
- reuses that prepared work on the next matching block when:
  - graph id matches
  - projection epoch matches
  - parameter epoch matches
  - configured block size matches
  - frame/channel shape matches
  - input signature matches
  - processing epoch is still inside the short validity window

The shared engine snapshot now exposes:

- `prework_cache_enabled`
- `prework_cache_hits`
- `prework_cache_misses`
- `last_prework_cache_hit`
- `prework_cache_valid_until_processing_epoch`
- `last_prework_source_processing_epoch`

This turns the anticipative/realtime split into reusable engine state rather
than only a per-block graph-execution handoff.

## Validation

- `cargo check -p signal-graph -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_reuses_prework_cache_for_matching_adjacent_block -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`

## Notes

The current cache is intentionally conservative. It is a short adjacent-block
reuse path, not a general scheduler or background worker. It proves runtime
ownership of prework validity and reuse without claiming that anticipative
execution has become threaded or long-lived.

## Next Task

Promote the current short-lived prework cache into a more explicit scheduler
surface, for example by separating cache admission from consumption, adding
stronger invalidation on transport and parameter churn, or turning the cache
into a background-lane state machine owned by `signal-runtime`.
