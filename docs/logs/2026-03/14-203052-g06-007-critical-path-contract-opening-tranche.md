# g06.007 - Critical-Path Contract Opening Tranche

Date: 2026-03-14
Milestone: `g06.007`
Batch: `7.1`
Status: complete

## Summary

Opened `g06.007` by freezing the first bounded contract for graph critical-path,
hot-node, and worker-lane instrumentation. The new contract keeps hotspot and
lane attribution runtime-owned, anchored to the closed per-block timing seam
from `g06.006`, and explicit about what remains deferred until runtime depth
lands.

## What changed

- added `docs/contracts/018-graph-critical-path-hot-node-and-worker-lane-instrumentation-contract.md`
- froze the first authority hierarchy for:
  - `RuntimeEngineBlockSnapshot` as explanatory graph and lane context
  - `RuntimePerformanceSnapshot` as the bounded hot-node, hot-group, and
    worker-lane width digest
  - `RuntimePerformanceTraceReceipt` as the bounded peak-hotspot digest
- made the current `hot_latency_*` fields an explicit shared boundary rather
  than incidental implementation detail
- kept host callback and OS scheduler data advisory rather than canonical
  hotspot authority
- rolled roadmap and reference surfaces forward so Batch 7.2 is now the active
  queue

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Deferred

- per-node elapsed-time instrumentation
- explicit critical-path DTOs beyond the current bounded hot-node and hot-group
  digest family
- per-lane occupancy percentages, thread ids, and host scheduler telemetry
- public proof and acceptance seams for this milestone

## Next

Continue `g06.007` with Batch 7.2 by implementing richer runtime-owned
critical-path, hot-node, and worker-lane instrumentation on top of the frozen
bounded contract while keeping scheduler attribution inside `signal-runtime`.
