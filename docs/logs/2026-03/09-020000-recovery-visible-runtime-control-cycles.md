---
title: Recovery Visible Runtime Control Cycles
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, host-local, host-server, recovery, control]
---

# Summary

Bound plugin-sandbox recovery to the hardened runtime control contract in both Signal host assemblies.

# Changes

- Updated `signal-host-local` sandbox recovery to stop the runtime with `StopReason::DegradedModeRecovery` before sandbox teardown and start it again after sandbox restart.
- Updated `signal-host-server` sandbox recovery the same way so local and server runtime behavior stay aligned.
- Extended the heartbeat-watchdog recovery tests in both hosts to assert the shared runtime control snapshot instead of relying on implicit host behavior.
- Documented that degraded sandbox recovery is now visible in the runtime-owned control snapshot and supervisor report surfaces.

# Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-host-server --no-run`
- `git diff --check`

# Next Task

Thread restart intent and last stop reason through the host recovery summaries and supervisor-tool output, then tighten recovery tests around repeated watchdog episodes so degraded-mode restart sequencing is visible across multi-epoch soak paths as well as single recovery events.
