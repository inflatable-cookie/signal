---
title: Runtime Recovery Sequence Export
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, recovery, supervision, supervisor-tools]
---

# Summary

Moved degraded restart tracing into the runtime observation stream and exported it as an ordered recovery sequence in the shared supervisor report.

# Changes

- Added a runtime-owned `RecoveryRestartIntent` vocabulary and `RuntimeEvent::RecoveryCycle` so restart episodes are emitted as typed runtime events rather than inferred only from summary state.
- Extended `RuntimeEventRecorder` and `RuntimeObservationDiagnostics` to retain ordered recovery events alongside supervision updates and plugin faults.
- Updated `RuntimeSupervisorReport` JSON and text rendering to expose recovery counts, the latest recovery record, and a full ordered `recovery_sequence`.
- Threaded recovery-cycle emission through both local and server host sandbox recovery paths while keeping the host execution summaries as convenience views.
- Tightened runtime and host soak tests so repeated watchdog recovery is asserted through the shared runtime event stream, not only through aggregate counters.

# Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-host-server --no-run`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`

# Next Task

Carry more of the plugin-sandbox lifecycle into the same event stream, especially typed activate/deactivate/reset/teardown milestones, so soak reporting can correlate recovery episodes with the control-path transitions that bracket them.
