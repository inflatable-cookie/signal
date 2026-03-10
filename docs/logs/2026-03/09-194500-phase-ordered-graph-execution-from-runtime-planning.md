---
title: Phase Ordered Graph Execution From Runtime Planning
status: done
owner: codex
updated: 2026-03-09
tags: [signal, runtime, graph, planning, execution]
---

## Summary

Promoted the graph planning groups into real execution structure.

`signal-graph` now derives explicit phase order from the current planning
groups and executes nodes by that phase order instead of a flat node list.
`signal-runtime` now propagates the active anticipative mode into
`GraphExecutionContext`, refreshes planning-phase metadata into
`engine_block_snapshot`, and exposes:

- `phase_count`
- `anticipative_phase_count`
- `phase_order`

The local and server hosts still prove different planning behavior:

- local host runs with anticipative mode enabled and reports an anticipative
  phase
- server host runs with anticipative mode disabled and reports only realtime
  phases

## Validation

- `cargo check -p signal-graph -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-graph latency_nodes_become_anticipative_candidates_when_enabled -- --nocapture`
- `cargo test -p signal-runtime runtime_replans_graph_when_anticipative_mode_changes -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`

## Notes

The remaining gap is no longer “does the runtime have a planning surface?” but
“does the runtime schedule differently because of it?”. The next useful step is
to turn the current phase order into distinct execution lanes or dispatch
boundaries.

## Next Task

Use the current phase order for actual runtime scheduling policy, most likely
by introducing explicit anticipative dispatch boundaries or separate
background/realtime execution lanes inside `signal-runtime`.
