---
title: Per-Session Transport Liveness In Active Sessions
date: 2026-03-09
status: closed
---

## Summary

Finished the `transport_session_summary` deepening batch by carrying
heartbeat freshness, dispatch state, and last active block sequence into each
concurrent `active_sessions` record instead of leaving those signals only on
the top-level session summary.

## Changes

- extended `signal-runtime` active transport-session records with per-session
  `currently_attached`, `heartbeat_freshness`, `dispatch_state`, and
  `active_block_sequence`
- updated the transport-session summarizer to preserve liveness across attach
  and detach-requested transitions and to apply heartbeat/dispatch events to
  the matching active session
- pinned the new shape in the concurrent-session runtime fixture and in the
  supervisor export fixture so JSON exports prove the per-session liveness
  fields serialize inside `active_sessions`
- updated the README, package map, contract, and roadmap notes to describe the
  new active-session liveness guarantee

## Validation

- `cargo check -p signal-runtime -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-supervisor-tools --no-run`

## Next Task

Decide whether `transport_session_summary` is now stable enough to freeze for
schema version 1, or whether it still needs one more round of session-state
detail such as per-session fault freshness/history before the split can be
considered complete.
