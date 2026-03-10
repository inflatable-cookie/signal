---
title: runtime owned lingering cleanup scheduling state
status: completed
owner: codex
updated: 2026-03-09
tags: [signal, runtime, recovery, lingering, transport]
---

## Summary

Promoted lingering cleanup scheduling state into `signal-runtime`.

## Changes

- Added `LingeringCleanupMode` to the public `signal-runtime` interface.
- Extended runtime transport concurrency sessions with cleanup attempt count,
  last cleanup mode, cleanup-in-progress, last cleanup epoch, and last cleanup
  error.
- Changed `lingering_cleanup_candidates_for_sandbox(...)` into a mutating
  runtime preparation step that marks cleanup scheduling state before hosts
  execute transport teardown.
- Added runtime APIs for recording lingering cleanup failure and clearing
  in-progress cleanup state after successful teardown.
- Updated local/server hosts so strict pre-attach cleanup and best-effort
  post-start reconciliation consume the runtime cleanup scheduling API instead
  of tracking that state only in host-local control flow.

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
especially if the engine needs deferred cleanup execution and late-detach
reconciliation themselves to move behind runtime-owned workflow APIs instead of
remaining host-local teardown orchestration layered on top of runtime cleanup
scheduling state.
