# g01.006 Routed Graph Execution Baseline

Date: 2026-03-11
Owner: core-product

## Summary

Landed the first real routed execution tranche for `g01.006`. `signal-graph`
now executes node plans through graph-owned buses instead of mutating one shared
buffer in sequence, and it reports direct-edge, fan-in, fan-out, and routed
latency information directly from the graph seam.

## Work completed

- replaced the old shared-buffer execution path in
  `crates/signal-graph/src/lib.rs` with routed bus execution
- added graph-owned routing diagnostics and latency reporting:
  - `GraphRoutingSummary`
  - routed bus counts
  - direct-edge / fan-in / fan-out / mixed-bus counts
  - output bus latency and max bus latency
  - silent source bus count in `GraphBlockReport`
- changed `GraphPreparedDispatch` to carry prepared bus state instead of only a
  single buffer so anticipative work can hand intermediate routed buses into the
  realtime pass
- extended graph validation to classify routing problems that the current
  baseline does not support:
  - missing upstream producers
  - forward references that imply unsupported feedback/cycle-like ordering
  - inconsistent producer channel layouts on the same output bus
- added deterministic routing fixtures and tests for:
  - direct edge chaining
  - fan-in mixdown
  - fan-out split
  - send/return-style routed execution
  - unsupported forward-reference classification
- updated `signal-runtime` to pass the new graph routing summary into the
  graph-owned execution path

## Why this tranche matters

The previous graph contract work made node metadata explicit, but execution still
behaved like a linear stage list. This tranche makes the graph seam actually
behave like a graph. Future runtime scheduling, latency policy, send/return
semantics, and parameter-event splitting can now build on routed buses instead
of a placeholder execution model that hid mixing and ordering assumptions.

## Realtime contract notes

- routed execution still clones and adapts buffers inside the current baseline,
  so it is not yet the final zero-allocation hot-path shape
- the important contract work here is ownership and determinism: routing,
  unsupported-order classification, and latency meaning now live in
  `signal-graph` instead of wrapper lore
- full tail propagation and a tighter scratch-buffer strategy remain deferred

## Deferred after this tranche

- graph-owned tail reporting
- richer cycle/feedback handling beyond explicit unsupported forward-reference
  classification
- bounded sub-block parameter-event application on top of the routed execution
  seam
- tighter scratch-buffer reuse to reduce execution-path allocation

## Validation

- `cargo test -p signal-graph`
- `cargo test -p signal-runtime --no-run`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy test --repo .`
- touched-file `git diff --check`

## Notes

- repo-wide `git diff --check` is still blocked by the unrelated blank line at
  EOF in `CMakeLists.txt`; the touched graph/runtime/roadmap/log files pass
  cleanly
- this batch intentionally leaves tail semantics unchecked in the roadmap until
  graph-owned tail contribution is surfaced alongside the new latency path

## Next Task

Finish the remaining `g01.006` routing core by adding graph-owned tail
reporting and then start bounded sub-block parameter-event application on top of
the routed execution seam.
