---
title: g01.008 topology-aware host path closure
status: complete
owner: core-product
created: 2026-03-11
updated: 2026-03-11
tags: [signal, hardware, host, runtime, roadmap, g01, g01.008]
---

## Summary

Closed the remaining `008.2` item by making `signal-host-local` exercise a
runtime-owned track/bus/output graph shape through the existing CoreAudio-backed
output pump instead of only proving a flattened three-node path.

## What landed

- added a runtime-owned `GraphContractProjection` seam so projected graphs can
  attach explicit bus contracts and topology metadata without widening the
  basic `GraphProjection` shell
- `signal-runtime` now rebuilds the active `ExecutableGraph` with projected node
  contracts and refreshes planning/scheduler state after those contracts land
- `signal-host-local` now applies topology-aware graph contracts after the demo
  graph projection:
  - `track-input` and `plugin-insert` as `TrackLane` nodes on `track:lead`
  - `bus-main` as the `Bus` node for `mix:master`
  - `output-main` as the `ConsoleNode` writing back to `main:out`
- the default host boot path therefore exercises a named
  `main:in -> bus:track:lead -> bus:mix:tracks -> bus:console:main -> main:out`
  route through the host-owned output pump
- `signal-host-local` tests now prove that runtime’s execution-topology summary
  still reflects the intended track/bus/output shape after the host has booted,
  started sandboxes, and processed blocks through the pump
- the `signal-host-local` CLI summary now includes compact topology counts
  alongside the existing pump and runtime observation output

## Contract outcome

- topology ownership still lives with runtime/graph contracts rather than with
  host-local glue
- the host path now proves a real node-oriented mixer shape at the trust edge
  without adding a second host-only topology model
- `g01.008` can now move into diagnostics and failure handling with a stable
  topology-aware local host path already in place

## Validation

- `cargo test -p signal-runtime runtime_graph_contract_projection_updates_execution_topology_for_projected_graphs -- --nocapture`
- `cargo test -p signal-host-local local_host_executes_track_bus_output_topology_through_audio_pump -- --nocapture`
- `cargo test -p signal-runtime`
- `cargo test -p signal-host-local`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy test --repo .`

## Next

Start `008.3` as a broader diagnostics and failure-handling batch: promote the
new topology-aware local host path into richer shared export summaries, then
validate restart, device-loss, and steady-state smoke behavior against that
same exercised track/bus/output route.
