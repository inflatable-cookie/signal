---
title: Transport Session Summary Companion Surface
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, supervisor, transport-session]
---

# Summary

Added a shared healthy-path `transport_session_summary` to the supervisor
surface so the runtime can expose transport/session visibility without
weakening the explicitly fault-adjacent meaning of `transport_fault_sequence`
and `transport_fault_summary`.

# Changes

- Added `TransportSessionBoundaryMode` and `TransportSessionSummary` to
  `signal-runtime`.
- `RuntimeObservationReport` now carries `transport_session_summary`,
  derived from:
  - `transport_sequence`
  - `heartbeat_sequence`
  - `block_dispatch_sequence`
- The shared session summary reports:
  - attach/detach/detach-fault counts
  - heartbeat request/respond/miss counts
  - block-dispatch request/complete/timeout counts
  - first/last epoch and block
  - last sandbox/lease/region identity
- Kept `transport_fault_summary` and `transport_fault_sequence` explicitly
  fault-adjacent only.
- Updated compact, multiline, and JSON rendering so downstream tooling can use
  the healthy-path session summary directly instead of inferring it from the
  full event sequences.
- Re-exported the new shared session summary types from `signal-runtime`.
- Updated runtime and supervisor-tool fixtures to pin the new summary.
- Updated the README, package map, supervisor export contract, and roadmap so
  the split between fault and healthy-path transport surfaces is explicit.

# Validation

- `cargo check -p signal-runtime -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-supervisor-tools --no-run`
- `cargo fmt --all`
- `git diff --check`

# Next Task

Decide whether `transport_session_summary` should now be treated as the stable
healthy-path companion for schema version 1, or whether it still needs more
session identity/state detail such as currently attached/active state before
the split can be considered frozen.
