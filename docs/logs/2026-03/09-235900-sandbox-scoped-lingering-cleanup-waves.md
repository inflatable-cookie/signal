---
title: sandbox scoped lingering cleanup waves
status: completed
owner: codex
updated: 2026-03-09
tags: [signal, runtime, recovery, lingering, scheduling]
---

## Summary

Added sandbox-scoped cleanup-wave identity to the runtime-owned lingering
cleanup workflow.

## Changes

- Extended runtime lingering cleanup work with `cleanup_wave` so one cleanup
  cycle can survive across queued work and deferred retries.
- Extended `LingeringCleanupPlan` and active transport concurrency sessions
  with `cleanup_wave` / `last_cleanup_wave`.
- Added `PendingLingeringCleanupWaveSummary` and surfaced
  `transport_concurrency_snapshot.pending_cleanup_waves` in the shared runtime
  report/export layer.
- Kept host cleanup execution unchanged in responsibility: hosts still perform
  broker teardown, but wave identity now comes entirely from runtime state.
- Updated the runtime cleanup scheduler test to pin wave allocation for the
  initial cleanup pass, deferred retry reuse of the same wave, and creation of
  a new wave for a later cleanup cycle.

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_orders_lingering_cleanup_candidates_by_provenance_then_attach_sequence -- --nocapture`
- `cargo test -p signal-host-local late_lingering -- --nocapture`
- `cargo test -p signal-host-server late_lingering -- --nocapture`
- `cargo test -p signal-host-local adjacent_overlap -- --nocapture`
- `cargo test -p signal-host-server adjacent_overlap -- --nocapture`
- `git diff --check`
- `effigy health`
- `effigy validate`

## Next Task

Push cleanup-wave progression further into runtime-owned control state, so
late-detach reconciliation and deferred retries can advance through explicit
runtime wave phases instead of hosts only draining queued work items when they
become due.
