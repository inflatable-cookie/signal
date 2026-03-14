# 007 - Graph Critical-Path, Hot-Node, And Worker-Lane Instrumentation

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g06.006
Vision tags: `RUNTIME`, `PROFILING`, `MULTICORE`

## Problem

Per-block pressure alone is not enough for deeper optimization or soak
evidence. Signal needs stronger graph and worker-lane instrumentation that can
identify where timing pressure actually comes from.

## Goals

- [ ] expose critical-path, hot-node, and worker-lane occupancy observations
- [ ] connect multicore and anticipative execution behavior to measurable
  runtime evidence
- [ ] support downstream consumers that need bounded causal timing insight

## Non-Goals

- [ ] no product-specific profiler UX
- [ ] no full arbitrary trace viewer or flamegraph system

## Execution Plan

### Batch 7.1 - Critical-Path Contract

- [x] define critical-path and hot-node observation semantics
- [x] decide what worker-lane occupancy and queue detail belong in shared
  Signal-owned surfaces

Batch 7.1 freezes the bounded hotspot hierarchy in
`docs/contracts/018-graph-critical-path-hot-node-and-worker-lane-instrumentation-contract.md`.
It makes `RuntimeEngineBlockSnapshot` the explanatory graph-and-lane context
authority, freezes `RuntimePerformanceSnapshot` as the shared consumer digest
for hot-node, hot-group, and worker-lane width context, and keeps
`RuntimePerformanceTraceReceipt` as the bounded peak-hotspot digest across an
observation window. It also explicitly keeps host callback and OS scheduler
detail advisory so Batch 7.2 can deepen runtime-owned instrumentation without
drifting into host-local attribution.

### Batch 7.2 - Runtime Instrumentation Depth

- [x] implement graph/node/worker-lane instrumentation on top of block timing
- [x] keep multicore and anticipative execution semantics aligned with receipts

Batch 7.2 turns the bounded contract into a real runtime-owned hotspot seam.
`RuntimePerformanceSnapshot` now carries richer hot-group and
critical-path-lane attribution plus typed per-lane summaries, and
`RuntimePerformanceTraceReceipt` now preserves the peak critical lane alongside
the existing peak hot-node and hot-group fields. The implementation reuses
`RuntimeEngineBlockSnapshot` planning and lane-order truth so multicore and
anticipative execution evidence stay derived inside `signal-runtime` rather
than being reconstructed by hosts or tools.

### Batch 7.3 - Public Proof

- [x] add focused proof that downstream consumers can inspect the bounded
  critical-path surface without private runtime hooks

Batch 7.3 closes the bounded consumer seam. The widened hotspot and lane
receipts are now proven through downstream-style runtime tests, stable local
and server host-edge tests, and a machine-readable
`signal-supervisor-tools --describe-critical-path-boundary` descriptor plus the
repo-owned `effigy acceptance:critical-path-boundary` task.

## Acceptance Criteria

- [x] Signal can identify bounded critical-path and worker-lane timing pressure
- [x] later optimization and soak evidence can cite node/lane sources directly
- [x] instrumentation remains runtime-owned rather than host-local

## Risks And Mitigations

- Risk: detailed instrumentation becomes unstable across internal refactors.
- Mitigation: freeze only bounded semantic summaries at the consumer boundary.
- Risk: multicore telemetry leaks internal scheduler details unnecessarily.
- Mitigation: keep lane occupancy and hot-path summaries high-value and typed.

## Evidence Requirements

- [x] log each meaningful graph-instrumentation tranche
- [x] run focused validation for critical-path and lane observations
- [x] record deferred trace breadth explicitly

## Next Task

Continue `g06.008` with Batch 8.1 by defining the deferred-work scheduler
priority, backpressure, starvation, and cancellation contract on top of the
closed timing, hotspot, and orchestration receipt families.
