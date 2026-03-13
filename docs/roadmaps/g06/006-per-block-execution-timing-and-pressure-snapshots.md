# 006 - Per-Block Execution Timing And Pressure Snapshots

Status: planned
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

- [ ] define block timing, deadline pressure, and budget-overrun observations
- [ ] decide which metrics belong in runtime-facing versus supervisor exports

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

Continue `g06.007` by identifying critical-path, hot-node, and worker-lane
sources of the timing pressure now visible.
