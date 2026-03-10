---
title: Plugin Backed Host Graphs And Sandbox Aware Prework Gating
status: closed
owner: codex
updated: 2026-03-10
tags: [signal, runtime, scheduler, plugin, host]
---

## Summary

Connected the plugin-backed scheduler policy to more real runtime state by
making active plugin sandbox count participate in prework-lane gating and by
updating the local/server demo graphs to exercise plugin-backed nodes in the
host timeout/recovery path.

## What Changed

- made `signal-runtime` derive `PluginConstrained` from both graph shape and
  active plugin sandbox count instead of from plugin-backed nodes alone
- added runtime-owned `prework_service_active_plugin_sandboxes` and
  `prework_service_plugin_gate_active` fields to the shared engine snapshot,
  compact report, multiline report, and JSON export surface
- gated the prework service lane so plugin-constrained graphs now yield the
  background lane entirely under non-normal pressure when more than one active
  plugin sandbox is present
- updated the local demo graph to include an explicit plugin-backed insert in
  the realtime path and aligned the local timeout/recovery proof with the new
  three-phase plan
- updated the server demo graph to classify its middle realtime node as
  plugin-backed so the shared scheduler/export path is exercised in both host
  assemblies
- added focused runtime coverage for policy switching as active plugin sandbox
  count changes and for full-lane yield under plugin-constrained pressure

## Validation

- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_plugin_backed_policy_tracks_active_plugin_sandbox_count -- --nocapture`
- `cargo test -p signal-runtime runtime_plugin_constrained_lane_yields_when_multiple_plugin_sandboxes_are_active -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`
- `effigy validate --repo .`
- `effigy health --repo .`

## Notes

- no stale Effigy locks showed up in this batch
- the only lock contention was normal Cargo/build-directory waiting from
  overlapping local Rust commands
- this keeps plugin-backed work on the realtime side of the engine boundary
  while letting active sandbox pressure directly narrow or fully gate
  anticipative background servicing
