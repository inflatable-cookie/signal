---
title: runtime executable graph block path
status: completed
owner: codex
updated: 2026-03-09
tags: [signal, runtime, graph, execution, local-host]
---

## Summary

Moved the next Signal engine milestone from supervision-only control work into
real runtime execution by adding a concrete graph block-processing path.

## Changes

- Expanded `signal-graph` from a trait-only shell into a small executable
  stage-based graph surface with `GraphStageSpec`, `ExecutableGraph`, and
  block metrics.
- Extended `GraphProjection` so runtime can apply a real stage list instead of
  only carrying graph metadata.
- Added runtime-owned executable graph state plus `engine_block_snapshot` and a
  concrete `process_engine_block(...)` path in `signal-runtime`.
- Wired the local host to apply a demo graph projection and execute one runtime
  graph block per realtime cycle alongside the existing plugin-sandbox path.
- Exposed the resulting engine metrics in local host execution summaries and in
  the shared supervisor/export report surface.

## Validation

- `cargo fmt --all`
- `cargo check -p signal-graph -p signal-runtime -p signal-host-local -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_executes_applied_graph_block_and_updates_snapshot -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`
- `effigy health --repo .`
- `effigy validate --repo .`

## Next Task

Push this execution slice beyond the local proof host by giving
`signal-runtime` a richer executable graph service and threading the same block
path into more host/runtime scenarios, so graph/runtime maturity starts to
catch up with the already-strong supervision and recovery layer.
