---
title: Host Recovery Intent And Stop Reason Visibility
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, host-local, host-server, supervisor-tools, recovery]
---

# Summary

Exposed host-issued recovery intent and degraded restart stop reasons through the Signal host summaries and supervisor-tool export.

# Changes

- Added `last_recovery_intent` and `last_stop_reason` to the grouped execution summaries for both `signal-host-local` and `signal-host-server`.
- Threaded crash-driven recovery and watchdog-driven recovery through those summary fields without widening the runtime-owned control surface.
- Updated `signal-supervisor-tools` text and JSON renderers so recovery intent and requested stop reason are visible in the host-facing export.
- Tightened host recovery tests so repeated watchdog episodes assert runtime control stop/start counts and degraded stop reasons across multi-epoch soak paths.
- Updated the supervisor export contract and active docs to record the limited host-summary exception for host-issued recovery detail.

# Validation

- `cargo fmt --all`
- `cargo check -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-host-server --no-run`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`

# Next Task

Move the same recovery visibility into supervisor-facing runtime events or diagnostics so restart episodes can be traced as a sequence, not only as final summary state, then pin that shape in the soak-oriented reporting path.
