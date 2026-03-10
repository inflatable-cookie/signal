---
title: Transport Session Concurrency Summary
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, supervisor, transport-session]
---

# Summary

Extended `transport_session_summary` so it can describe concurrent healthy
transport sessions instead of implying a single active sandbox/lease path.

# Changes

- Added `ActiveTransportSessionRecord` to `signal-runtime`.
- Extended `TransportSessionSummary` with:
  - `current_attached_session_count`
  - `max_concurrent_attached_sessions`
  - `active_sessions`
- Derived those fields by replaying the existing transport sequence and
  tracking active attached/detach-requested sessions by sandbox/lease/region.
- Kept the existing single active identity fields as convenience views while
  making concurrent healthy-path state explicit in the shared summary.
- Added a focused runtime test that exercises two concurrent active transport
  sessions and pins the resulting summary shape.
- Updated the shared JSON export and supervisor-tool fixture so the concurrency
  fields are part of the frozen exported schema.
- Updated the README, package map, contract, and roadmap so
  `transport_session_summary` is documented as concurrency-aware rather than
  single-session-only.

# Validation

- `cargo check -p signal-runtime -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-supervisor-tools --no-run`
- `cargo fmt --all`
- `git diff --check`

# Next Task

Decide whether `transport_session_summary` is now stable enough to freeze for
schema version 1, or whether it still needs one more round of session-state
detail such as per-session heartbeat/dispatch liveness before the split can be
considered complete.
