---
title: runtime queued prework admission between blocks
status: closed
owner: codex
updated: 2026-03-09
tags: [signal, runtime, scheduler, prework]
---

# Summary

Promoted anticipative prework from a same-block cache path into a runtime-owned
queued admission surface that can be primed between block iterations and
consumed on the following block.

# What changed

- Added `SignalRuntime::prepare_engine_prework_for_block(...)` so hosts can
  queue anticipative work for a target future block.
- Extended `RuntimeEngineBlockSnapshot` with queued-ahead admission and
  consumption counters plus origin block fields.
- Added `SupersededByAdmission` cache retirement/invalidation handling so newly
  queued prework can explicitly retire older pending work.
- Updated local host block sequencing to allocate the next block sequence early
  and prime queued prework between block iterations.
- Added a runtime proof that a primed next block is consumed cross-block, and
  kept the host timeout-recovery proofs green with the new scheduler surface.

# Validation

- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_reuses_prework_cache_for_matching_adjacent_block -- --nocapture`
- `cargo test -p signal-runtime runtime_consumes_primed_prework_for_the_next_block -- --nocapture`
- `cargo test -p signal-runtime runtime_invalidates_prework_cache_on_parameter_and_transport_changes -- --nocapture`
- `cargo test -p signal-runtime runtime_prework_cache_expires_by_block_sequence_window -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`

# Notes

- Local host still retires some queued prework under per-block parameter churn;
  that is now explicit engine behavior rather than hidden cache loss.
- No stale Effigy locks showed up in this batch.
