# 007 - Graph Critical-Path, Hot-Node, And Worker-Lane Instrumentation

Status: planned
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

- [ ] define critical-path and hot-node observation semantics
- [ ] decide what worker-lane occupancy and queue detail belong in shared
  Signal-owned surfaces

### Batch 7.2 - Runtime Instrumentation Depth

- [ ] implement graph/node/worker-lane instrumentation on top of block timing
- [ ] keep multicore and anticipative execution semantics aligned with receipts

### Batch 7.3 - Public Proof

- [ ] add focused proof that downstream consumers can inspect the bounded
  critical-path surface without private runtime hooks

## Acceptance Criteria

- [ ] Signal can identify bounded critical-path and worker-lane timing pressure
- [ ] later optimization and soak evidence can cite node/lane sources directly
- [ ] instrumentation remains runtime-owned rather than host-local

## Risks And Mitigations

- Risk: detailed instrumentation becomes unstable across internal refactors.
- Mitigation: freeze only bounded semantic summaries at the consumer boundary.
- Risk: multicore telemetry leaks internal scheduler details unnecessarily.
- Mitigation: keep lane occupancy and hot-path summaries high-value and typed.

## Evidence Requirements

- [ ] log each meaningful graph-instrumentation tranche
- [ ] run focused validation for critical-path and lane observations
- [ ] record deferred trace breadth explicitly

## Next Task

Continue `g06.008` by turning the new observations into stronger deferred-work
policy and backpressure semantics.
