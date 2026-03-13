# 008 - Deferred-Work Scheduler Priority, Backpressure, And Cancellation Policy

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g06.006, g06.007
Vision tags: `RUNTIME`, `ORCHESTRATION`, `SCHEDULING`

## Problem

Signal already has deferred-work and prework depth, but Loophole's next runtime
lane needs a stronger, reusable answer for priority, starvation, backpressure,
and cancellation instead of letting products improvise orchestration policy.

## Goals

- [ ] freeze a stronger deferred-work priority and backpressure model
- [ ] expose runtime-owned orchestration state suitable for products and soak
  harnesses
- [ ] keep cancellation and starvation policy inside Signal rather than app
  hosts

## Non-Goals

- [ ] no distributed job system or remote queue protocol yet
- [ ] no host-specific task-runner abstractions

## Execution Plan

### Batch 8.1 - Policy Contract

- [ ] define deferred-work classes, priority rules, starvation signals, and
  cancellation semantics
- [ ] align the policy with timing and pressure receipts from earlier milestones

### Batch 8.2 - Runtime Orchestration Depth

- [ ] implement stronger scheduler/orchestration state and policy receipts
- [ ] keep local and server host consumers aligned to the same runtime-owned
  orchestration model

### Batch 8.3 - Focused Policy Proof

- [ ] add focused proofs showing priority, backpressure, and cancellation remain
  observable without host-local policy forks

## Acceptance Criteria

- [ ] Signal has an explicit deferred-work priority and backpressure model
- [ ] later products can observe orchestration state without owning it
- [ ] profiling and soak work can cite typed starvation/cancellation evidence

## Risks And Mitigations

- Risk: orchestration policy sprawls into a generic job framework.
- Mitigation: keep the milestone on runtime-owned classes and receipts only.
- Risk: host layers keep shadow policies anyway.
- Mitigation: require policy visibility through shared consumer surfaces.

## Evidence Requirements

- [ ] log each meaningful deferred-work policy tranche
- [ ] run focused orchestration validation
- [ ] record deferred distributed-orchestration scope explicitly

## Next Task

Continue `g06.009` by widening actual plugin functionality into the first
non-CLAP adapter baseline.
