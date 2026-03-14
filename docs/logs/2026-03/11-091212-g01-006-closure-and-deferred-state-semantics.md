# 2026-03-11 — g01.006 closure and deferred state semantics

## Summary

Closed `g01.006` by making the current dynamic-stage state model explicit in
code and diagnostics, then recording the deliberately deferred mixer and
cross-block state semantics that should not be inferred from the current graph
surface.

## What Changed

- added `GraphDynamicStageStateModel` and surfaced it through
  `GraphBlockReport`, so graph execution now states that dynamic filter/delay
  kernels are currently rebuilt per block
- added `dynamic_kernel_stage_count` alongside the state model so graph and
  runtime diagnostics can distinguish static stage counts from state-bearing DSP
  kernels
- propagated the new graph fields into `RuntimeEngineBlockSnapshot` and all
  compact, multiline, and JSON observation rendering paths
- added regression tests proving the current deferred behavior explicitly:
  low-pass and delay dynamic stages do not retain state across separate graph
  block executions yet
- completed the roadmap record for deferred console/lane/mixer semantics and
  moved the generation sequencing surface on to `g01.007`

## Deferred Semantics Made Explicit

- console-node, strip, and bus mix semantics remain deferred beyond the generic
  graph routing and parameter contract
- plugin-backed node parameter interpretation remains a runtime/trust-edge
  concern for now
- cross-block retention of dynamic filter/delay kernel state is deferred until
  prepared-dispatch and prework paths can carry explicit node-state snapshots

## Validation

- `cargo test -p signal-graph`
- `cargo test -p signal-runtime --no-run`
- `effigy health`
- `effigy validate`
- `effigy test`
- touched-file `git diff --check`

## Outcome

`g01.006` is now complete. The next active milestone is `g01.007`, which should
make `signal-runtime` the authority for transport progression, block-clock
truth, and scheduler invalidation on top of the graph seam completed here.
