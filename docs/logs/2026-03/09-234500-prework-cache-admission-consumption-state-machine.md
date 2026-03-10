---
title: prework cache admission and consumption state machine
date: 2026-03-09
status: done
owner: codex
---

## Summary

Aligned the runtime-owned anticipative prework cache with the newer scheduler
surface already exposed in the interfaces. The cache now reports admitted and
consumed states explicitly instead of collapsing both into one prepared state.

## Changes

- updated `signal-runtime` so prework misses that prepare a reusable dispatch
  increment `prework_cache_admissions` and leave the cache in `Admitted`
  state
- updated cache hits so reused prework increments
  `prework_cache_consumptions` and leaves the cache in `Consumed` state
- preserved invalidation behavior while keeping admission/consumption history
  visible in the shared engine snapshot
- updated runtime tests and local/server host timeout-recovery assertions to
  reflect the new state machine
- refreshed the README, architecture notes, contract docs, and roadmap note so
  the scheduler-facing semantics match the runtime behavior

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_executes_applied_graph_block_and_updates_snapshot -- --nocapture`
- `cargo test -p signal-runtime runtime_reuses_prework_cache_for_matching_adjacent_block -- --nocapture`
- `cargo test -p signal-runtime runtime_invalidates_prework_cache_on_parameter_and_transport_changes -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `git diff --check`
- `effigy validate --repo .`
- `effigy health --repo .`

## Next

Push the prework lifecycle further into a stronger scheduler model by
separating prework admission from later consumption policy, most likely with a
background-lane freshness state machine or tighter reuse constraints than the
current short epoch window.
