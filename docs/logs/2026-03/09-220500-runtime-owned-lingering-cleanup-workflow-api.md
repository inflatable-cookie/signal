---
title: runtime owned lingering cleanup workflow api
status: completed
owner: codex
updated: 2026-03-09
tags: [signal, runtime, recovery, lingering, transport]
---

## Summary

Promoted lingering cleanup from runtime state plus host-local loops into an
explicit runtime workflow API.

## Changes

- Added public `LingeringCleanupPlan` to `signal-runtime`.
- Replaced raw runtime candidate access with
  `plan_lingering_cleanup_for_sandbox(...)`.
- Added runtime-owned cleanup completion API via
  `complete_lingering_cleanup_success(...)`, while failure continues through
  `record_lingering_cleanup_failure(...)`.
- Switched local/server hosts to execute runtime-produced cleanup plans instead
  of iterating cleanup candidates directly from runtime state.
- Updated the runtime cleanup test to pin plan metadata as well as candidate
  ordering and cleanup failure state.

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
retries themselves to be modeled as runtime-owned state transitions or queued
work items instead of remaining host-local broker teardown orchestration that
executes a runtime-produced cleanup plan.
