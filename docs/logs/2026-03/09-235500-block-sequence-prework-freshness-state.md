---
title: block-sequence prework freshness state
date: 2026-03-09
status: done
owner: codex
---

## Summary

Promoted prework reuse policy from coarse processing-epoch reuse into an
explicit block-sequence freshness model.

## Changes

- added a runtime-owned `RuntimePreworkFreshnessState` surface so snapshots can
  report `Fresh`, `Expiring`, `Exhausted`, and `Invalidated` prework states
- keyed prework reuse expiry to `block_sequence` with an explicit block
  freshness window, while preserving processing-epoch invalidation as a
  separate safety boundary
- exposed block-sequence freshness metadata in `engine_block_snapshot`,
  including valid-until block sequence, source/admission/consumption block
  sequences, and remaining valid blocks
- tightened runtime and host tests so the prework path now proves fresh,
  expiring, exhausted, and expired-by-block-sequence behavior

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_executes_applied_graph_block_and_updates_snapshot -- --nocapture`
- `cargo test -p signal-runtime runtime_reuses_prework_cache_for_matching_adjacent_block -- --nocapture`
- `cargo test -p signal-runtime runtime_prework_cache_expires_by_block_sequence_window -- --nocapture`
- `cargo test -p signal-runtime runtime_invalidates_prework_cache_on_parameter_and_transport_changes -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `git diff --check`
- `effigy validate`
- `effigy health`

## Next

Turn the current block-sequence freshness model into a stronger scheduler
surface by separating prework admission from background-lane execution policy,
most likely with a queued prework state machine or explicit prework retirement
when graph/parameter churn races with anticipated future consumption.
