# 008 - Deferred-Work Scheduler Priority, Backpressure, And Cancellation Policy

Status: complete
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

- [x] define deferred-work classes, priority rules, starvation signals, and
  cancellation semantics
- [x] align the policy with timing and pressure receipts from earlier milestones

Batch 8.1 freezes the bounded scheduler-policy vocabulary in
`docs/contracts/019-deferred-work-scheduler-priority-backpressure-and-cancellation-contract.md`.
It keeps deferred-work classes, priority bands, starvation meaning,
backpressure, and cancellation semantics inside `signal-runtime`, and it
explicitly composes that policy with the closed block-timing and critical-path
receipt families instead of leaving scheduler pressure to host-local queue
heuristics.

### Batch 8.2 - Runtime Orchestration Depth

- [x] implement stronger scheduler/orchestration state and policy receipts
- [x] keep local and server host consumers aligned to the same runtime-owned
  orchestration model

Batch 8.2 lands the first richer runtime-owned scheduler-policy receipts in
`signal-runtime`. `RuntimeDeferredServiceReceipt` now carries typed priority
band, blocking-priority, backpressure source, starvation, and cancellation
fields, and the same policy state now rolls into `RuntimePerformanceSnapshot`
and `RuntimePerformanceTraceReceipt` so orchestration pressure can be cited
through existing timing and hotspot surfaces instead of host-local queue
reclassification.

### Batch 8.3 - Focused Policy Proof

- [x] add focused proofs showing priority, backpressure, and cancellation remain
  observable without host-local policy forks

Batch 8.3 closes the shared consumer boundary. The widened deferred-work
scheduler-policy receipts are now proven through public runtime reexports,
stable local and server host edges, and the machine-readable
`signal-supervisor-tools` deferred-work policy boundary descriptor plus a
repo-owned Effigy acceptance task. Consumers can now inspect priority,
backpressure, starvation, cancellation, and bounded trace evidence without
private queue-state helpers or host-local policy forks.

## Acceptance Criteria

- [x] Signal has an explicit deferred-work priority and backpressure model
- [x] later products can observe orchestration state without owning it
- [x] profiling and soak work can cite typed starvation/cancellation evidence

## Risks And Mitigations

- Risk: orchestration policy sprawls into a generic job framework.
- Mitigation: keep the milestone on runtime-owned classes and receipts only.
- Risk: host layers keep shadow policies anyway.
- Mitigation: require policy visibility through shared consumer surfaces.

## Evidence Requirements

- [x] log each meaningful deferred-work policy tranche
- [x] run focused orchestration validation
- [x] record deferred distributed-orchestration scope explicitly

## Next Task

Continue `g06.009` with Batch 9.1 by aligning the VST3 adapter baseline to the
existing backend-neutral capability and lifecycle contract before runtime
realization widens.
