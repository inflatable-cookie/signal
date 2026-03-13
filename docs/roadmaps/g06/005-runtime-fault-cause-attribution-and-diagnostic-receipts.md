# 005 - Runtime Fault-Cause Attribution And Diagnostic Receipts

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g06.001, g06.003, g06.004
Vision tags: `RUNTIME`, `DIAGNOSTICS`, `RECOVERY`

## Problem

Signal exposes useful diagnostics today, but later optimization and soak work
still need clearer causal receipts that explain why a runtime entered a given
degraded, recovering, or faulted posture.

## Goals

- [ ] define runtime-owned causal diagnostic receipts above raw counters
- [ ] connect fault classification to interruption and recovery semantics
- [ ] support downstream consumers that need typed fault evidence rather than
  log parsing or heuristic summaries

## Non-Goals

- [ ] no broad observability platform or fleet telemetry scope
- [ ] no product-specific diagnostics UI expansion

## Execution Plan

### Batch 5.1 - Causal Receipt Contract

- [ ] define causal receipt families for xrun, callback, plugin, device, and
  deferred-work pressure faults
- [ ] align them with readiness, interruption, and recovery state

### Batch 5.2 - Runtime Diagnostic Depth

- [ ] materialize the causal receipts in runtime and supervisor surfaces
- [ ] keep local and server host exports aligned with the new meaning

### Batch 5.3 - Consumer Proof

- [ ] add focused proof that causal fault receipts remain consumable through
  Signal-owned boundaries without private host logic

## Acceptance Criteria

- [ ] Signal exposes typed runtime fault-cause receipts
- [ ] later profiling and soak work can cite causal evidence directly
- [ ] host products no longer need to infer cause from unrelated counters

## Risks And Mitigations

- Risk: causal receipts duplicate existing diagnostics noisily.
- Mitigation: freeze a small, explainable receipt family first.
- Risk: products keep preferring legacy summaries.
- Mitigation: prove the new receipts through public runtime/export surfaces.

## Evidence Requirements

- [ ] log each meaningful causal-diagnostics tranche
- [ ] run focused runtime/export validation for causal receipts
- [ ] record deferred diagnostics breadth explicitly

## Next Task

Continue `g06.006` by turning fault and recovery state into measurable per-block
timing and pressure observations.
