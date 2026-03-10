---
title: Transport And Heartbeat Event Stream
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, transport, heartbeat, supervisor-tools]
---

# Summary

Extended the shared runtime event stream so supervisor reporting now captures
broker transport churn and heartbeat control-loop markers alongside lifecycle
and recovery events.

# Changes

- Added typed transport records for broker attach, detach request, detach
  completion, and detach fault milestones.
- Added typed heartbeat records for request, response, and miss markers with
  per-block context where available.
- Updated local and server hosts to emit transport and heartbeat events at the
  real attach, detach, response, and miss sites instead of leaving that detail
  trapped in host counters.
- Extended runtime diagnostics and supervisor export rendering so
  `transport_sequence` and `heartbeat_sequence` are available in shared text
  and JSON reports.
- Tightened runtime, host, and supervisor-tool tests so the new transport and
  heartbeat paths are asserted through the shared event stream.

# Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-host-server --no-run`
- `cargo test -p signal-supervisor-tools --no-run`

# Next Task

Add typed block-dispatch and lease-rollover milestones to the same runtime
event stream so soak analysis can correlate broker churn with actual render
work and lease generation changes across restart episodes.
