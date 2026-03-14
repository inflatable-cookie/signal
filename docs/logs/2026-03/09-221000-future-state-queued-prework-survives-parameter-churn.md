---
title: future state queued prework survives parameter churn
status: closed
owner: codex
updated: 2026-03-09
tags: [signal, runtime, scheduler, prework]
---

# Summary

Extended queued prework admission so runtime can admit the next block against
the future parameter epoch and transport state it expects to consume, which
lets matching next-block control updates preserve queued prework instead of
retiring it immediately.

# What changed

- Added `SignalRuntime::prepare_engine_prework_for_block_with_future_state(...)`
  so queued prework can be admitted for a target block together with optional
  future parameter and transport overrides.
- Extended prework matching so cached queued work compares against the admitted
  future transport state as well as graph, projection, parameter epoch, and
  input signature.
- Tightened invalidation so `apply_parameter_batch(...)` and
  `apply_transport_projection(...)` only retire queued prework when the
  applied control state actually differs from the state the prework was
  admitted for.
- Updated the local host block loop to prime the next block with its expected
  future parameter epoch, which turns the queued-prework path into a real
  cross-block consumption proof under per-block parameter churn.
- Updated the local timeout-recovery proof to reflect the new scheduler
  behavior: queued admissions are now consumed later (`queued_consumptions=7`)
  instead of being retired immediately, with cache hits rising to `7` and
  misses dropping to `3`.

# Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_consumes_primed_prework_for_the_next_block -- --nocapture`
- `cargo test -p signal-runtime runtime_invalidates_prework_cache_on_parameter_and_transport_changes -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`
- `effigy validate`
- `effigy health`

# Notes

- This batch did not hit any stale Effigy locks. One earlier lock conflict was
  caused by running `effigy health` and `effigy validate` against the same
  build tree in parallel; rerunning them serially was clean.
- Transport override support is now present in runtime admission, but the host
  proofs currently exercise the future-state path mainly through the next
  block's parameter epoch rather than a changing future transport projection.
