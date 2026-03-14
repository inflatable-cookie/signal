---
title: Graph Semantic Prework Service Policy
status: closed
owner: codex
updated: 2026-03-10
tags: [signal, runtime, scheduler, anticipative, graph]
---

## Summary

Tied elevated-pressure prework servicing to graph execution semantics instead
of only future block distance.

## What Changed

- added a runtime-owned prework service semantic policy with `Balanced` and
  `LatencyFocused` modes
- derived that policy from the current graph planning/latency shape inside
  `signal-runtime`
- widened elevated-pressure background service for latency-focused
  anticipative graphs while keeping the existing balanced backlog behavior for
  the current demo graphs
- exposed the semantic policy in the shared compact report, multiline report,
  and JSON export surfaces
- added a focused runtime proof for latency-focused elevated-pressure service

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime runtime_latency_focused_graph_expands_elevated_pressure_service_scope -- --nocapture`
- `cargo test -p signal-runtime runtime_elevated_pressure_preserves_deferred_prework_targets -- --nocapture`
- `cargo test -p signal-runtime runtime_prework_service_lane_throttles_under_elevated_pressure -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`
- `effigy validate`
- `effigy health`

## Notes

- no stale Effigy locks showed up in this batch
- this keeps the current local/server demo graphs on the balanced path while
  giving latency-heavier anticipative graphs a distinct scheduler behavior
