---
title: runtime owned lingering provenance and cleanup scheduling
status: completed
owner: codex
updated: 2026-03-09
tags: [signal, runtime, recovery, lingering, transport]
---

## Summary

Moved lingering-session provenance and cleanup candidate ordering into
`signal-runtime`.

## Changes

- Exported `TransportSessionProvenance` from `signal-runtime`.
- Extended runtime transport concurrency state with lingering-session
  provenance, attach sequence, and attach epoch metadata.
- Added runtime-owned lingering cleanup candidate planning for one sandbox,
  ordered by provenance and attach order rather than host-local sorting.
- Switched local/server host cleanup paths to consume
  `lingering_cleanup_candidates_for_sandbox(...)`.
- Added a focused runtime test covering steady-origin versus
  recovery-replacement lingering cleanup order.
- Updated Signal docs to mark lingering provenance and cleanup ordering as a
  runtime-owned concern.

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server`
- `cargo test -p signal-runtime runtime_orders_lingering_cleanup_candidates_by_provenance_then_attach_sequence -- --nocapture`
- `cargo test -p signal-host-local adjacent_overlap -- --nocapture`
- `cargo test -p signal-host-server adjacent_overlap -- --nocapture`
- `git diff --check`

## Next Task

Push lingering-session lifecycle management further into runtime state,
especially if the engine needs runtime-owned cleanup scheduling and provenance
to extend from candidate ordering into deferred cleanup execution and late
detach reconciliation without relying on host-local orchestration alone.
