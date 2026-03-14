# 2026-03-11 16:31:42 GMT - g01.009 plugin node runtime bridge tranche

## Summary

Advanced `g01.009` into the first real `009.3` render-path batch by wiring a
plugin-backed node to the graph/runtime seam through a typed block-scoped
render override, then proving `signal-host-local` can run sandboxed plugin
audio through the engine path instead of treating the sandbox lifecycle and the
engine render loop as separate simulations.

This tranche does not close `009.3` yet because transport truth, parameter
truth, and degraded recovery still need to be exercised directly on the bound
plugin node path, but it removes the biggest structural gap: sandbox block
audio can now enter the runtime-owned graph at the bound plugin node boundary
with explicit bypass and latency/tail semantics.

## What changed

- extended `crates/signal-graph/src/lib.rs` with a typed
  `GraphNodeRenderOverride` path so realtime execution can:
  - consume external render output on plugin-backed nodes
  - bypass the node explicitly without deleting it from the graph
  - project latency/tail effects from the external render onto the routed bus
    state
- extended `crates/signal-runtime/src/interfaces.rs` and
  `crates/signal-runtime/src/runtime.rs` with a block-scoped
  `PluginNodeRenderBatch` contract that:
  - validates node IDs against the active graph and plugin bindings
  - stores render batches by `(processing_epoch, block_sequence)`
  - consumes the matching batch during engine block execution
- updated `crates/signal-host-local/src/host.rs` so each brokered sandbox block
  outcome is translated into a runtime plugin-node render batch before the
  engine block is processed
- made the local demo graph put its anticipative node upstream of the
  plugin-backed node so the bound plugin path now feeds a realtime suffix that
  reaches `main:out`
- added focused tests that pin:
  - graph-level external render injection and bypass behavior
  - runtime consumption of block-scoped plugin render batches
  - local host end-to-end routing of sandboxed plugin audio through the bound
    engine node

## Validation

- `cargo test -p signal-graph`
- `cargo test -p signal-runtime`
- `cargo test -p signal-host-local`
- `cargo fmt`
- `effigy validate`
- `effigy test`
- touched-file `git diff --check`

## Ownership notes

- `signal-graph` now owns the generic execution seam for externally rendered
  node output without learning CLAP or sandbox details
- `signal-runtime` now owns the block-scoped contract that lets trust-edge
  hosts inject plugin-backed node output into the engine on a typed boundary
- `signal-host-local` remains the trust-edge owner of sandbox block dispatch,
  output capture, and fault handling, but it no longer needs to fake plugin
  participation in the engine path

## Follow-on

The next `009.3` batch should route transport and parameter truth into the
same bound plugin-node render path, then exercise watchdog timeout/restart and
fallback behavior while the plugin-backed node remains attached to the engine
graph.
