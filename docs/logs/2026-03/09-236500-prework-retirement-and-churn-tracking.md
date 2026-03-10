---
title: prework retirement and churn tracking
date: 2026-03-09
status: done
owner: codex
---

## Summary

Extended the runtime-owned prework scheduler surface so invalidation now
records explicit retirement state, not just cache clearing.

## Changes

- added runtime-owned prework retirement tracking so cache invalidation now
  records retirement count, retirement reason, and whether the retired
  prepared work had been consumed before churn retired it
- kept retirement sourced from the actual invalidation path, so graph,
  transport, parameter, input-signature, and expiry churn all produce a real
  retirement record instead of only a missing cache entry
- tightened runtime tests so parameter and transport invalidation now prove
  unconsumed retirement, while block-sequence expiry proves a consumed
  retirement path
- tightened local/server host timeout-recovery proofs so local now exposes the
  retirement-heavy churn profile while server still proves the disabled path

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_invalidates_prework_cache_on_parameter_and_transport_changes -- --nocapture`
- `cargo test -p signal-runtime runtime_prework_cache_expires_by_block_sequence_window -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `git diff --check`
- `effigy validate --repo .`
- `effigy health --repo .`

## Next

Turn the current freshness and retirement model into a stronger scheduler path
by moving beyond one inline prepared-result slot, most likely with queued
background-lane prework generation or explicit prework cancellation when graph
and parameter churn races with anticipated future dispatches.
