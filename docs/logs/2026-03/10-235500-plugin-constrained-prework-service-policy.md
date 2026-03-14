---
title: Plugin Constrained Prework Service Policy
status: closed
owner: codex
updated: 2026-03-10
tags: [signal, runtime, scheduler, graph, plugin]
---

## Summary

Extended the anticipative prework scheduler so plugin-backed graph shape now
constrains elevated-pressure background servicing instead of treating all
non-latency-focused graphs the same.

## What Changed

- added `PluginBacked` as a first-class `GraphNodeExecutionClass` in
  `signal-graph`
- kept plugin-backed nodes on the realtime/stateful planning path while
  exposing `plugin_backed_node_count` in planning and block reports
- threaded plugin-backed node counts and planned-node execution classes into
  `signal-runtime` snapshots and JSON export
- added `PluginConstrained` to the runtime prework semantic policy so mixed
  graphs with plugin-backed realtime nodes narrow elevated-pressure service
  scope instead of widening it from latency hints alone
- added focused runtime coverage proving plugin-constrained elevated-pressure
  servicing leaves future work pending while balanced/latency-focused paths
  keep their existing behavior

## Validation

- `cargo check -p signal-graph -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-graph plugin_backed_nodes_remain_realtime_and_are_counted_in_planning -- --nocapture`
- `cargo test -p signal-runtime runtime_plugin_backed_graph_constrains_elevated_pressure_service_scope -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`
- `effigy validate`
- `effigy health`

## Notes

- no stale Effigy locks showed up in this batch
- one workspace lock conflict was self-inflicted by launching `validate` and
  `health` in parallel; rerunning serially was clean
- this keeps plugin-backed work on the realtime side of the graph boundary
  while still letting the anticipative scheduler react to plugin-heavy graph
  shape under pressure
