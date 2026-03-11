# g01.006 Node/Buffer Contract Baseline

Date: 2026-03-11
Owner: core-product

## Summary

Opened `g01.006` with the first real graph-owned execution contract. `signal-graph`
now declares node-level buffer and topology metadata instead of treating every
node as an anonymous stage list over one shared block. The new surface makes
input/output buses, channel expectations, silence handling, reset lifecycle, and
topology role explicit enough for later routing and runtime work to build on one
declared contract.

## Work completed

- extended `crates/signal-graph/src/lib.rs` with explicit contract types:
  - `GraphNodeBusEndpoint`
  - `GraphNodeBufferContract`
  - `GraphNodeSilencePolicy`
  - `GraphChannelAdaptationMode`
  - `GraphNodeResetPolicy`
  - `GraphNodeTopologyRole`
  - `GraphNodeTopologyMetadata`
- added graph-owned contract reporting and validation:
  - `GraphContractSummary`
  - `GraphNodeContractSummary`
  - `GraphContractIssue`
  - adaptation classification and topology-aware issue detection
- updated `GraphNodeSpec` so executable nodes carry buffer and topology
  contract metadata directly
- extended `GraphBlockReport` so runtime-facing diagnostics can see:
  - contract issue count
  - silence-clear node count
  - adaptive-channel node count
  - resettable node count
  - scratch-buffer count
  - track/bus/send-return/console-role counts
- applied the first executable contract behavior in the processing path:
  - silent input now respects node silence policy (`Process`, `Bypass`,
    `ClearOutput`)
- updated `signal-runtime` to construct the richer `GraphNodeSpec` shape with
  graph-owned defaults when projections do not yet carry the new metadata
- added regression coverage for:
  - topology/adaptation contract summaries
  - send-node same-bus rejection
  - silent block clearing under node silence policy

## Why this tranche matters

Before this batch, `signal-graph` could execute a list of stages, but it still
did not say what a node expected from its input block or what kind of topology
role it played. That was too vague for future routing, latency, parameter, and
runtime scheduler work. The graph seam now has a declared node/buffer contract
that later fan-in, fan-out, send/return, and block-splitting work can extend
instead of replacing.

## Realtime contract notes

- node contract metadata is stored on the graph and reused at execution time
- silent-block policy handling is allocation-free
- channel-adaptation behavior is declared and validated now, but full routed
  buffer adaptation remains deferred to the next routing tranche so the graph
  can keep ownership of the rule set without smuggling ad hoc allocation-heavy
  behavior into the current hot path

## Deferred after this tranche

- deterministic multi-edge routing and mixing across fan-in/fan-out paths
- graph-owned latency/tail propagation through routed topologies
- sub-block parameter event application inside graph execution
- full channel-adaptation execution using graph-owned scratch buffers

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
- this batch intentionally opens `g01.006` without yet attempting the full
  routing matrix; the goal here was to make graph-owned node semantics explicit
  before more execution complexity lands

## Next Task

Land the first deterministic routing tranche in `g01.006`: direct edges,
fan-in, fan-out, send/return-style fixtures, and graph-owned latency reporting
on top of the new node/buffer contract.
