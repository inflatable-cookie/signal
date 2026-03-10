---
title: Runtime Planning Groups And Anticipative Snapshot
status: done
owner: codex
updated: 2026-03-09
tags: [signal, runtime, graph, planning]
---

## Summary

Turned the graph execution-class and latency hints into an active runtime
planning surface instead of leaving them as graph-only metadata.

`signal-runtime` now refreshes a graph planning summary whenever a graph is
applied or runtime anticipative mode is reconfigured. The shared
`engine_block_snapshot` now exposes:

- `anticipative_planning_enabled`
- `inline_realtime_node_count`
- `stateful_realtime_node_count`
- `anticipative_eligible_node_count`
- `planned_nodes`

The host assemblies now prove two real planning modes:

- local host keeps anticipative planning enabled, so its latency-bearing node
  is reported as anticipative-eligible
- server host disables anticipative planning by default, so the same class of
  latency-bearing node is folded into the realtime/stateful plan

## Validation

- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_executes_applied_graph_block_and_updates_snapshot -- --nocapture`
- `cargo test -p signal-runtime runtime_replans_graph_when_anticipative_mode_changes -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `git diff --check`
- `effigy validate --repo .`

## Notes

`effigy validate --repo .` completed cleanly for this batch, including the
legacy C++ path. No stale Effigy lock issue showed up during this run.

## Next Task

Use the current planning groups for real runtime execution policy rather than
only snapshot/reporting, for example phased node ordering, anticipative
dispatch boundaries, or latency-aware execution-plan splitting inside
`signal-runtime`.
