---
title: Node Execution Classes And Latency Semantics
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, graph, runtime, engine, scheduling]
---

## Summary

Extended the node/plan graph model with basic scheduling-relevant semantics.
Graph nodes now declare execution class and latency metadata, and
`signal-runtime` folds those into the shared engine snapshot so the runtime
plan says more than “which grouped stages exist.”

## Changes

- Added `GraphNodeExecutionClass` in `signal-graph` with:
  - `PureTransform`
  - `Stateful`
  - `LatencyBearing`
- Added `execution_class` and `latency_samples` to graph nodes and graph
  projections.
- Extended graph block reports with:
  - `stateful_node_count`
  - `latency_node_count`
  - `total_latency_samples`
  - `max_node_latency_samples`
- Added validation in `signal-runtime`:
  - pure-transform nodes must report zero latency
  - latency-bearing nodes must report non-zero latency
- Extended `RuntimeEngineBlockSnapshot` with the same aggregate node/latency
  metrics.
- Updated local/server demo graphs to use mixed execution classes and explicit
  latency-bearing output nodes.

## Validation

- Passed: `cargo check -p signal-graph -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- Passed: `cargo test -p signal-runtime runtime_executes_applied_graph_block_and_updates_snapshot -- --nocapture`
- Passed: `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- Passed: `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`

## Next

Use the current execution classes and latency hints for actual runtime planning
behavior, for example node ordering groups, latency aggregation policy, or
anticipative eligibility, so runtime moves from enriched plan metadata toward
real plan-driven execution structure.
