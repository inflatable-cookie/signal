# 018 Graph Critical-Path, Hot-Node, And Worker-Lane Instrumentation Contract

Status: active
Owner: core-product
Updated: 2026-03-14
Related contracts: `docs/contracts/004-runtime-multicore-scheduling-and-anticipative-execution-contract.md`, `docs/contracts/017-per-block-execution-timing-and-pressure-snapshot-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first shared contract for bounded graph critical-path, hot-node, and
worker-lane instrumentation so later `g06.007` runtime work can deepen causal
timing evidence without pushing scheduler attribution or hotspot
reinterpretation back into host-local tooling.

## Authority hierarchy

Critical-path and worker-lane instrumentation have one authority chain:

1. `signal-runtime` owns the canonical bounded attribution meaning:
   - the processed block identity already frozen in contract `017`
   - the graph-planning and dispatch shape already frozen in contract `004`
   - the bounded hot-node and hot-group summary for the current or recent block
   - the bounded worker-lane and dispatch-shape context that explains where
     the hottest work was allowed to execute
2. supervisor and stable host-edge surfaces may expose that meaning, but they
   must not reinterpret it:
   - `RuntimeEngineBlockSnapshot`
   - `RuntimePerformanceSnapshot`
   - `RuntimePerformanceTraceReceipt`
   - `RuntimeObservationReport`
   - `RuntimeSupervisorReport`
   - stable host-edge `supervisor_report()` surfaces
3. hosts, products, and acceptance tools may add advisory callback or workload
   context, but they must not become the authority for:
   - the runtime-owned hot-node answer
   - the runtime-owned hottest planning-group or worker-lane summary
   - whether a block should be treated as critical-path pressured versus merely
     adjacent to host scheduling noise

If a critical-path or hotspot claim cannot be explained through Signal-owned
reports or receipts, it is not yet part of the shared instrumentation
contract.

## Shared terms

This contract freezes seven shared terms.

### Critical-path observation

A critical-path observation is the bounded runtime-owned answer to which graph
or scheduler region most strongly explains the current or recent block's timing
pressure.

This milestone does not require full elapsed-time attribution for every node or
lane. It freezes the bounded summary meaning first so later instrumentation can
deepen the same seam instead of replacing it.

### Hot node

A hot node is the currently exported runtime-owned node summary that best
explains the bounded hot-path answer for the active block context.

For the current baseline this means the hottest exported node proxy carried by:

- `RuntimePerformanceSnapshot::hot_latency_node_id`
- `RuntimePerformanceSnapshot::hot_latency_node_group`
- `RuntimePerformanceSnapshot::hot_latency_node_topology_role`
- `RuntimePerformanceSnapshot::hot_latency_node_plugin_sandbox_id`
- `RuntimePerformanceSnapshot::hot_latency_node_samples`

Batch 7.1 freezes those fields as the shared consumer seam. Later batches may
switch from a latency-weighted proxy toward richer elapsed-time attribution, but
they must preserve the contract that runtime owns the answer and exports it
through the same bounded family.

### Hot group

A hot group is the bounded planning-group or execution-class aggregate that best
explains current hotspot pressure at a coarser level than one node.

For the current baseline this means the exported hot-group proxy carried by:

- `RuntimePerformanceSnapshot::hot_latency_group`
- `RuntimePerformanceSnapshot::hot_latency_group_total_samples`
- `RuntimePerformanceTraceReceipt::peak_hot_latency_group`
- `RuntimePerformanceTraceReceipt::peak_hot_latency_group_total_samples`

### Worker-lane observation

A worker-lane observation is the bounded runtime-owned explanation of how graph
work was distributed across realtime, prepared, anticipative, or handoff lanes
for the measured block context.

This milestone freezes worker-lane meaning around typed lane and dispatch
surfaces first, not around raw thread ids, CPU-core affinity, or host-side
thread tracing.

### Lane occupancy summary

Lane occupancy summary is the bounded typed answer to how much execution width
and dispatch structure runtime attempted for the active graph context.

The first shared occupancy families are:

- `lane_count`
- `anticipative_lane_count`
- `lane_order`
- `dispatch_count`
- `prepared_dispatch_count`
- `realtime_dispatch_count`
- `dispatch_handoff_count`

This is occupancy or width context, not yet a per-lane elapsed-time trace.

### Hotspot trace digest

A hotspot trace digest is the narrower multi-observation receipt that preserves
peak hot-node and hot-group evidence across a bounded observation window.

For the current baseline that digest is:

- `RuntimePerformanceTraceReceipt::peak_hot_latency_node_id`
- `RuntimePerformanceTraceReceipt::peak_hot_latency_node_group`
- `RuntimePerformanceTraceReceipt::peak_hot_latency_node_samples`
- `RuntimePerformanceTraceReceipt::peak_hot_latency_group`
- `RuntimePerformanceTraceReceipt::peak_hot_latency_group_total_samples`

### Advisory host execution evidence

Advisory host execution evidence is callback cadence, callback overrun, or
backend scheduling detail that may sharpen diagnosis but does not outrank the
runtime-owned hot-node, hot-group, or worker-lane answer.

This mirrors the advisory split already frozen in contracts `016` and `017`.

## Contract rules

This contract freezes seven shared rules.

### Rule 1: hotspot ownership stays downstream of block timing

Critical-path and hot-node meaning must remain attached to the same
runtime-owned block boundary already frozen in contract `017`.

### Rule 2: scheduler attribution stays runtime-owned

Worker-lane and dispatch-shape meaning must compose with contract `004`
surfaces instead of being reconstructed from host-local thread knowledge.

### Rule 3: bounded summaries precede full traces

`g06.007` starts by freezing bounded hotspot and lane summaries first. Full
node-by-node elapsed-time traces, flamegraph-style exports, or arbitrary trace
buffers remain later work.

### Rule 4: current hot-node fields are semantic, not incidental

The existing `hot_latency_*` fields are now part of the shared instrumentation
surface. Later runtime depth may improve how runtime computes them, but
consumers should continue to treat them as the bounded canonical answer rather
than a disposable convenience field.

### Rule 5: worker-lane depth stops at typed width and dispatch context

This milestone freezes lane-count, lane-order, dispatch-count, and handoff
meaning, but does not yet freeze per-thread occupancy percentages, thread ids,
or host scheduler internals.

### Rule 6: public and automation digests stay narrower than runtime internals

`RuntimeEngineBlockSnapshot` may carry the richest bounded graph and lane
context, while `RuntimePerformanceSnapshot` and
`RuntimePerformanceTraceReceipt` remain the narrower consumer and automation
digests.

### Rule 7: later instrumentation may deepen without changing authority

Batch 7.2 may add richer actual timing attribution, worker-lane occupancy, or
critical-path summary DTOs, but it must extend this runtime-owned hierarchy
instead of inventing a second hotspot taxonomy in hosts or tools.

## Current runtime mapping

The repo already contains the bounded surfaces this contract is intended to
stabilize.

### RuntimeEngineBlockSnapshot

`RuntimeEngineBlockSnapshot` is already the runtime-owned graph and lane context
authority for:

- planning groups and phase order
- lane counts and lane order
- dispatch counts and handoff count
- planned node inventory including execution class, topology role, latency, and
  sandbox binding context

This milestone freezes that snapshot as the explanatory context for later
critical-path or worker-lane instrumentation.

### RuntimePerformanceSnapshot

`RuntimePerformanceSnapshot` is already the bounded consumer digest for:

- scheduler lane and dispatch width
- hot node proxy through `hot_latency_node_*`
- hot group proxy through `hot_latency_group*`
- the block timing and deadline-pressure seam from contract `017`

Batch 7.1 freezes those fields as the first shared hotspot boundary rather than
leaving them as incidental implementation detail.

### RuntimePerformanceTraceReceipt

`RuntimePerformanceTraceReceipt` already preserves peak hot-node and hot-group
evidence across a bounded observation window. This contract freezes that role so
later automation and soak work can cite peak hotspot evidence without private
trace plumbing.

## Explicitly deferred

Batch 7.1 does not yet freeze:

- per-node elapsed-time instrumentation
- explicit critical-path DTOs separate from the current hot-node and hot-group
  digest family
- per-lane occupancy percentages or lane queue depth over time
- host thread identifiers, CPU-core affinity, or OS scheduler telemetry
- flamegraph-style or arbitrary history exports

Those belong to Batch 7.2 runtime depth, Batch 7.3 public proof work, or later
acceptance and soak milestones.

## Batch 7.1 outcome

Batch 7.1 freezes the shared bounded instrumentation hierarchy:

- `RuntimeEngineBlockSnapshot` remains the explanatory graph and lane context
  authority
- `RuntimePerformanceSnapshot` is the shared consumer digest for hot-node,
  hot-group, and worker-lane width context
- `RuntimePerformanceTraceReceipt` is the bounded peak hotspot digest across an
  observation window
- host callback cadence and host scheduler detail remain advisory evidence
  rather than a competing hotspot authority

## Batch 7.2 outcome

Batch 7.2 deepens the frozen bounded seam without changing authority:

- `RuntimePerformanceSnapshot` now carries:
  - `hot_latency_group_node_count`
  - `critical_path_lane`
  - `critical_path_lane_node_count`
  - `critical_path_lane_plugin_backed_node_count`
  - `critical_path_lane_planning_group_count`
  - `critical_path_lane_total_latency_samples`
  - `worker_lane_summaries`
- `RuntimeWorkerLaneInstrumentationSummary` is now the typed per-lane summary
  DTO for bounded lane instrumentation:
  - lane identity
  - node count
  - plugin-backed node count
  - planning-group count
  - total latency samples
  - maximum node latency samples
- `RuntimePerformanceTraceReceipt` now preserves peak critical-lane evidence
  alongside the existing peak hot-node and hot-group digest:
  - `peak_hot_latency_group_node_count`
  - `peak_critical_path_lane`
  - `peak_critical_path_lane_node_count`
  - `peak_critical_path_lane_plugin_backed_node_count`
  - `peak_critical_path_lane_total_latency_samples`

This keeps worker-lane and hotspot attribution derived from
`RuntimeEngineBlockSnapshot` planning and lane-order truth instead of creating
host-local lane or hotspot heuristics.

## Batch 7.3 outcome

Batch 7.3 proves the widened seam stays consumable without private hooks:

- downstream-style runtime proof:
  - `public_runtime_critical_path_boundary_reports_bounded_hotspot_receipts`
- stable host-edge proofs:
  - `local_shared_host_edge_exports_runtime_critical_path_truth`
  - `server_shared_host_edge_exports_runtime_critical_path_truth`
- machine-readable boundary and repo-owned acceptance:
  - `signal-supervisor-tools --describe-critical-path-boundary`
  - `effigy acceptance:critical-path-boundary`

The bounded hot-node, hot-group, critical-path lane, and typed worker-lane
summary family is therefore closed as a shared runtime, supervisor, and stable
host-edge consumer boundary.

## Next Task

Continue `g06.008` with Batch 8.1 by freezing deferred-work scheduler
priority, backpressure, starvation, and cancellation semantics on top of the
closed timing, hotspot, and orchestration receipt families.
