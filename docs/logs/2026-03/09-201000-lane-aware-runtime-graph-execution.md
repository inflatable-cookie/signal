---
title: Lane Aware Runtime Graph Execution
status: done
owner: codex
updated: 2026-03-09
tags: [signal, runtime, graph, execution, scheduling]
---

## Summary

Extended the phase-ordered graph path into an explicit lane-aware execution
model.

`signal-graph` now derives `GraphExecutionLane` ordering from the current
planning groups:

- `Anticipative`
- `Realtime`

The executable graph processes phases inside those lanes instead of only a flat
node order or a phase list without lane meaning. `signal-runtime` now snapshots
that policy through:

- `lane_count`
- `anticipative_lane_count`
- `lane_order`

The runtime-owned execution context already carries `anticipative_enabled`, so
the active lane policy is controlled by runtime config rather than by host-side
inference.

Current proof points:

- local host still runs with anticipative mode enabled and now reports two
  lanes
- server host still runs with anticipative mode disabled and now reports a
  single realtime lane

## Validation

- `cargo check -p signal-graph -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-graph latency_nodes_become_anticipative_candidates_when_enabled -- --nocapture`
- `cargo test -p signal-runtime runtime_replans_graph_when_anticipative_mode_changes -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`

## Notes

This is still one-threaded execution. The useful next step is not more report
shape but using lane boundaries for actual runtime scheduling or dispatch
policy.

## Next Task

Use the new lane model for real runtime scheduling policy, most likely by
adding explicit anticipative dispatch boundaries or background/realtime lane
handoff inside `signal-runtime`.
