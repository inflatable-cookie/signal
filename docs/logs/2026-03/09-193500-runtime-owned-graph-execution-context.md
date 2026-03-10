---
title: Runtime-Owned Graph Execution Context
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, graph, engine, hosts]
---

## Summary

Moved the executable graph slice from “runtime runs a stage chain against a
buffer” toward a clearer engine contract. `signal-graph` now has an explicit
execution context/request model, `signal-runtime` now builds that context from
runtime-owned state, and both local/server hosts seed runtime transport and
parameter state so graph execution reflects more than a bare block sequence.

## Changes

- Added `GraphExecutionContext` and `GraphExecutionRequest` to `signal-graph`.
- Extended graph reports so execution context travels with processed blocks.
- `signal-runtime` now builds engine execution context from:
  - processing epoch
  - block sequence
  - projection epoch
  - parameter epoch
  - configured block size
  - transport playing/tempo/timeline state
- `signal-runtime` now advances transport position after successful engine
  block processing, including loop wrap behavior.
- Both `signal-host-local` and `signal-host-server` now seed runtime transport
  and parameter input state before executing graph blocks, and both timeout
  recovery proofs assert that the runtime-owned execution context is present.

## Validation

- Passed: `cargo check -p signal-graph -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- Passed: `cargo test -p signal-runtime runtime_executes_applied_graph_block_and_updates_snapshot -- --nocapture`
- Passed: `cargo test -p signal-graph executable_graph_carries_execution_context -- --nocapture`
- Passed: `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- Passed: `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- Passed: `git diff --check`
- Not rerun in this batch: `effigy health --repo .` and `effigy validate --repo .`, because the current repo-level legacy C++ blocker is already known and unrelated to these Rust engine changes.

## Next

Push the execution contract further by giving `signal-graph` and
`signal-runtime` a clearer scheduler-facing node/plan model instead of one flat
stage chain, then thread that plan through both hosts so the runtime owns more
of actual engine structure rather than only per-block execution context.
