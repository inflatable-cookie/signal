---
title: Node-Plan Graph Projection And Runtime Mapping
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, graph, runtime, hosts, engine]
---

## Summary

Moved the executable engine slice from a flat stage-chain projection to an
explicit node/plan projection model. `signal-graph` now represents executable
structure as nodes containing stages, `signal-runtime` maps graph projections
into that node plan, and both host assemblies execute the same runtime-owned
node-shaped engine contract.

## Changes

- Added `GraphNodeSpec` and `GraphExecutionPlan` to `signal-graph`.
- `ExecutableGraph` now executes a node/plan structure rather than one flat
  stage vector.
- Added `GraphNodeProjection` to `signal-runtime` interfaces.
- `GraphProjection` now carries `nodes` instead of a flat `stages` list.
- `RuntimeEngineBlockSnapshot` now carries `node_count` as well as
  `stage_count`.
- Updated runtime graph application logic to validate and map projected nodes
  into executable graph nodes.
- Updated local/server demo graph projections to use multiple nodes with
  explicit node ids.
- Kept the existing execution-context work intact so the runtime now owns both:
  - node-shaped graph structure
  - per-block execution context

## Validation

- Passed: `cargo check -p signal-graph -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- Passed: `cargo test -p signal-runtime runtime_executes_applied_graph_block_and_updates_snapshot -- --nocapture`
- Passed: `cargo test -p signal-graph executable_graph_carries_execution_context -- --nocapture`
- Passed: `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- Passed: `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`

## Next

Push the engine structure further by giving the node/plan model clearer
execution categories or scheduling semantics, for example distinguishing pure
sample transforms from stateful/latency-bearing nodes, then let runtime own
that planning rather than only mapping static node groups into a block pass.
