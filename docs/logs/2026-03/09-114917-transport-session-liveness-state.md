---
title: Transport Session Liveness State
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, supervisor, transport-session]
---

# Summary

Extended `transport_session_summary` with operational liveness state so the
healthy-path companion surface now describes not only attachment state and
identity, but also heartbeat freshness and current dispatch state.

# Changes

- Added `TransportHeartbeatFreshness` and `TransportDispatchState` to
  `signal-runtime`.
- Extended `TransportSessionSummary` with:
  - `heartbeat_freshness`
  - `dispatch_state`
  - `active_block_sequence`
- Derived those fields from the existing healthy-path event sequences:
  - heartbeat freshness from `heartbeat_sequence`
  - dispatch state and active block from `block_dispatch_sequence`
- Kept `transport_fault_summary` and `transport_fault_sequence` unchanged as
  the explicit fault-adjacent surfaces.
- Updated compact, multiline, and JSON rendering so the session summary now
  carries both attachment state and operational liveness.
- Re-exported the new shared liveness enums from `signal-runtime`.
- Updated runtime and supervisor-tool fixtures so the liveness fields are
  pinned in both the direct runtime report and the exported schema.
- Updated the README, package map, contract, and roadmap so the healthy-path
  session surface is documented as including current liveness state.

# Validation

- `cargo check -p signal-runtime -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-supervisor-tools --no-run`
- `cargo fmt --all`
- `git diff --check`

# Next Task

Decide whether `transport_session_summary` is now stable enough to freeze for
schema version 1, or whether it still needs one more round of session-state
detail such as multi-sandbox/lease concurrency before the split can be
considered complete.
