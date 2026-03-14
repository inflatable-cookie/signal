# 006 - Per-Block Execution Timing And Pressure Snapshots

Status: active
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

- [ ] instrument runtime block execution and expose bounded timing snapshots
- [ ] keep host-edge and export surfaces aligned to the same measurements

### Batch 6.3 - Focused Observation Proof

- [ ] add focused proofs showing timing and pressure snapshots are stable and
  consumable without private tracing hooks

## Acceptance Criteria

- [ ] Signal has explicit per-block timing and pressure snapshots
- [ ] downstream consumers can reason about runtime pressure from typed receipts
- [ ] later hot-path optimization can start from runtime data

## Risks And Mitigations

- Risk: instrumentation becomes expensive or noisy.
- Mitigation: start with bounded, high-value snapshot metrics only.
- Risk: products depend on unstable internal counters.
- Mitigation: freeze the public snapshot meaning first.

## Evidence Requirements

- [ ] log each meaningful profiling tranche
- [ ] run focused tests or traces for timing snapshots
- [ ] record deferred profiling breadth explicitly

## Next Task

Continue `g06.006` with Batch 6.2 by instrumenting bounded per-block execution
timing, deadline pressure, and budget-overrun fields on the frozen runtime-owned
measurement seam, then align supervisor and host-edge export to the same
observations before widening into `g06.007` hot-node and worker-lane depth.
