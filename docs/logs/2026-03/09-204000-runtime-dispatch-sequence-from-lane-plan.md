---
title: Runtime Dispatch Sequence From Lane Plan
status: done
owner: codex
updated: 2026-03-09
tags: [signal, runtime, graph, execution, dispatch]
---

## Summary

Promoted the lane-aware execution model into an explicit runtime dispatch
sequence.

`signal-graph` now derives a dispatch plan from the current lane order and
executes by:

- dispatch lane
- phase inside lane
- node inside phase

`signal-runtime` snapshots that policy through:

- `dispatch_count`
- `dispatch_boundary_count`
- `dispatch_order`

This means the active block execution path now distinguishes:

- local host: two dispatches (`Anticipative`, then `Realtime`)
- server host: one dispatch (`Realtime`)

## Validation

- `cargo check -p signal-graph -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-graph latency_nodes_become_anticipative_candidates_when_enabled -- --nocapture`
- `cargo test -p signal-runtime runtime_replans_graph_when_anticipative_mode_changes -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`

## Notes

The remaining gap is no longer “is there a dispatch model?” but “are those
dispatches backed by distinct runtime execution paths?”. Right now the dispatch
sequence is still executed synchronously in one thread.

## Next Task

Use the new dispatch sequence for actual runtime execution separation, most
likely by introducing a real background/realtime handoff or anticipative
prework boundary inside `signal-runtime`.
