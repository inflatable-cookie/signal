---
title: Expanded Lifecycle Sequence Control Plane
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, lifecycle, control-plane, supervisor-tools]
---

# Summary

Expanded the shared runtime lifecycle sequence so supervisor reporting now
covers more of the plugin-sandbox control plane, not just recovery teardown.

# Changes

- Expanded `PluginSandboxLifecycleStage` to cover sandbox ensure/handshake,
  plugin-type load, instance create/prepare/activate, transport attach, and
  the existing teardown/restart milestones.
- Fixed local and server host lifecycle emission so stages are derived from
  host control requests rather than inferred from sandbox responses.
- Emitted `TransportAttached` from prepare metadata extraction and
  `TransportTornDown` during recovery teardown so attach/detach boundaries are
  explicit in the shared event stream.
- Updated watchdog and mixed-soak assertions so lifecycle stage counts are
  checked through the shared runtime report and supervisor export path.
- Updated the Signal README, package map, and supervisor export contract to
  freeze the broader `lifecycle_sequence` meaning.

# Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-host-server --no-run`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`

# Next Task

Add typed broker attach/detach fault milestones and heartbeat control-loop
markers into the same runtime event stream so soak analysis can correlate
lifecycle transitions with transport churn and watchdog boundaries.
