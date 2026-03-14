# 2026-03-11 — g01.006 parameter-event application baseline

## Summary

Started `006.3` by giving `signal-graph` a graph-owned parameter batch surface
and a bounded block-local application model that rides on the routed execution
seam instead of runtime-specific glue.

## What Changed

- added `GraphParameterBatch`, `GraphParameterEvent`,
  `GraphParameterApplicationStrategy`, `GraphParameterTarget`, and
  `GraphStageParameter` to describe block-local parameter events in graph-owned
  terms
- extended `GraphExecutionRequest` and `GraphBlockReport` so parameter epoch,
  event counts, targeted nodes, ignored events, sub-block count, and coalesced
  events are all visible on the graph execution surface
- added bounded per-stage sub-block application by splitting only the affected
  node-stage work at parameter boundaries instead of replaying the whole graph
  for every event
- extended `GraphStageSpec` with first-pass dynamic `LowPass` and `Delay`
  stages, reusing `signal-dsp` kernels and block helpers for time-sensitive
  control inside graph execution
- updated the runtime seam only where needed so existing graph execution calls
  continue compiling with the new optional parameter-batch argument

## Timing Ownership

- runtime remains authoritative for transport state, block selection, and
  parameter batch epoch assignment
- graph owns only block-local interpretation of parameter events, including
  sample offsets relative to the current block and the bounded sub-block
  application strategy used inside node processing

## Evidence

Added deterministic fixtures proving:

- gain events split a block and update the execution report correctly
- low-pass cutoff events follow bounded sub-block processing and match the
  shared DSP helper behavior
- delay feedback events change recirculation within a block and match the shared
  DSP helper behavior

## Deferred

- explicit documentation of deferred console/lane/mixer semantics after this
  tranche is still open
- stateful filter/delay kernels are still recreated per graph block; this batch
  proves deterministic within-block event application, not cross-block state
  retention

## Validation

- `cargo test -p signal-graph`
- `cargo test -p signal-runtime --no-run`
- `effigy health`
- `effigy validate`
- `effigy test`
- touched-file `git diff --check`
