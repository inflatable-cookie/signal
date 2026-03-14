---
title: Runtime Plugin Binding Projection And Bound Session Gating
status: closed
owner: codex
updated: 2026-03-10
tags: [signal, runtime, scheduler, plugin, transport]
---

## Summary

Connected plugin-backed scheduler policy to real runtime/plugin lifecycle state
by binding plugin-backed graph nodes to sandbox ids and deriving
plugin-constrained prework gating from the live transport-session state of
those bound sandboxes.

## What Changed

- added `PluginBackedNodeBindingProjection` to the runtime projection surface,
  so hosts can bind plugin-backed node ids to sandbox ids without changing the
  core graph crate
- extended runtime planned-node export with `plugin_sandbox_id`, and added
  bound/active/degraded/missing plugin-backed sandbox counts to the shared
  engine snapshot/report/export surface
- taught `signal-runtime` to recompute plugin-constrained scheduler policy
  from bound transport-session state instead of relying only on a coarse
  active-sandbox count
- recomputed that policy on transport attach, detach, detach-fault, and
  session end so plugin-heavy scheduler behavior follows real sandbox churn
- updated the local/server host demo flows to apply plugin-backed node
  bindings for their realtime plugin nodes and assert that those bindings show
  up in the runtime snapshot
- added focused runtime tests proving bound-session projection and degraded
  bound-session gating

## Validation

- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_plugin_bindings_project_into_snapshot_and_track_bound_sessions -- --nocapture`
- `cargo test -p signal-runtime runtime_degraded_bound_plugin_session_gates_prework_lane -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`
- `effigy validate`
- `effigy health`

## Notes

- no stale Effigy locks showed up in this batch
- the only waiting/ownership contention was normal Cargo/build-directory
  locking during overlapping local Rust commands
- this moves plugin-backed scheduling closer to a real engine boundary:
  runtime can now tell whether plugin-heavy future work is tied to an active,
  degraded, or missing sandbox session rather than only seeing a plugin-ish
  graph shape plus a global sandbox count
