---
title: Anticipative Prework Handoff In Runtime Graph Path
status: done
owner: codex
updated: 2026-03-09
tags: [signal, runtime, graph, execution, dispatch, anticipative]
---

## Summary

Turned the current dispatch model into a real prework/realtime handoff without
introducing threads yet.

`signal-graph` now:

- runs anticipative dispatches first into a prepared buffer
- hands that prepared result into the realtime dispatch path
- reports:
  - `prepared_dispatch_count`
  - `realtime_dispatch_count`
  - `dispatch_handoff_count`
  - `prework_output_peak`
  - `realtime_input_peak`

`signal-runtime` now snapshots the same handoff surface through
`engine_block_snapshot`, so the shared runtime report can distinguish:

- local host: one anticipative prework dispatch plus one realtime dispatch
- server host: realtime-only dispatch with no prework handoff

## Validation

- `cargo check -p signal-graph -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-graph anticipative_dispatch_prepares_buffer_before_realtime_pass -- --nocapture`
- `cargo test -p signal-runtime runtime_executes_applied_graph_block_and_updates_snapshot -- --nocapture`

## Notes

This is still a single-threaded engine path. The important change is that the
runtime now has a genuine execution boundary between anticipative preparation
and realtime consumption instead of only metadata that says such a boundary
could exist.

## Next Task

Move the prework/realtime split further into runtime-owned scheduling behavior,
for example by caching anticipative-prework results across block boundaries or
adding a runtime-owned validity window so the handoff becomes reusable engine
state instead of only a per-block graph-execution step.
