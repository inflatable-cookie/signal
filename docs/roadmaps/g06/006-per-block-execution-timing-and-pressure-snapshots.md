# 006 - Per-Block Execution Timing And Pressure Snapshots

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g06.005
Vision tags: `RUNTIME`, `PROFILING`, `SCHEDULING`

## Problem

Signal has scheduler and engine surfaces, but Loophole's next runtime phase
needs actionable per-block timing and pressure evidence instead of anecdotal
performance diagnosis.

## Goals

- [ ] define per-block timing, pressure, and budget snapshots
- [ ] expose bounded runtime-owned measurements suitable for products and
  acceptance harnesses
- [ ] support later optimization and soak work without requiring internal-only
  tracing first

## Non-Goals

- [ ] no full tracing platform or continuous profiler UI
- [ ] no micro-optimization campaign before instrumentation lands

## Execution Plan

### Batch 6.1 - Timing Snapshot Contract

- [x] define block timing, deadline pressure, and budget-overrun observations
- [x] decide which metrics belong in runtime-facing versus supervisor exports

Batch 6.1 froze the bounded timing vocabulary in
`docs/contracts/017-per-block-execution-timing-and-pressure-snapshot-contract.md`.
That contract makes `RuntimeEngineBlockSnapshot` the authoritative per-block
measurement seam, keeps `RuntimeSchedulerSnapshot` as lifecycle and control
context, and freezes `RuntimePerformanceSnapshot` plus
`RuntimePerformanceTraceReceipt` as the narrower consumer and automation digests.
It also explicitly treats host callback cadence as advisory timing evidence
rather than a competing timing authority.

### Batch 6.2 - Runtime Instrumentation Baseline

- [x] instrument runtime block execution and expose bounded timing snapshots
- [x] keep host-edge and export surfaces aligned to the same measurements

Batch 6.2 landed the first real measurement seam in `signal-runtime`.
`process_engine_block()` now records bounded per-block execution timing and
derives deadline budget, utilization, pressure classification, and budget
overrun onto `RuntimeEngineBlockSnapshot`, then threads that same truth into
`RuntimeBlockExecutionSummary`, `RuntimePerformanceSnapshot`, and
`RuntimePerformanceTraceReceipt`. The runtime-owned timing path now updates
`cpu_load_percent` and `graph_latency_ms` from measured block execution instead
of leaving them as manual-only placeholders once real blocks have been
processed.

### Batch 6.3 - Focused Observation Proof

- [x] add focused proofs showing timing and pressure snapshots are stable and
  consumable without private tracing hooks

Batch 6.3 closed the bounded measurement seam with one consumer-facing proof
spine instead of another runtime-only claim. `signal-runtime` now has a
downstream-style public proof for `RuntimeEngineBlockSnapshot`,
`RuntimePerformanceSnapshot`, and `RuntimePerformanceTraceReceipt`; both stable
host edges prove `supervisor_report()` forwards the same timing truth; and
`signal-supervisor-tools --describe-block-timing-boundary` plus
`effigy acceptance:block-timing-boundary` make the boundary inspectable and
runnable without private tracing hooks.

## Acceptance Criteria

- [x] Signal has explicit per-block timing and pressure snapshots
- [x] downstream consumers can reason about runtime pressure from typed receipts
- [x] later hot-path optimization can start from runtime data

## Risks And Mitigations

- Risk: instrumentation becomes expensive or noisy.
- Mitigation: start with bounded, high-value snapshot metrics only.
- Risk: products depend on unstable internal counters.
- Mitigation: freeze the public snapshot meaning first.

## Evidence Requirements

- [x] log each meaningful profiling tranche
- [x] run focused tests or traces for timing snapshots
- [x] record deferred profiling breadth explicitly

## Next Task

Continue `g06.007` with Batch 7.1 by freezing graph critical-path, hot-node,
and worker-lane instrumentation semantics on top of the now-closed per-block
timing boundary, keeping scheduler attribution runtime-owned and bounded before
deeper instrumentation lands.
