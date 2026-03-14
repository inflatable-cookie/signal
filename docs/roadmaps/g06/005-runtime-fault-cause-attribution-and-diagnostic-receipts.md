# 005 - Runtime Fault-Cause Attribution And Diagnostic Receipts

Status: complete
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

- [x] define causal receipt families for xrun, callback, plugin, device, and
  deferred-work pressure faults
- [x] align them with readiness, interruption, and recovery state

Batch 5.1 froze the shared contract in
`docs/contracts/016-runtime-fault-cause-attribution-and-diagnostic-receipt-contract.md`,
keeping primary-cause ownership in `signal-runtime` while explicitly treating
host callback and backend counters as advisory evidence rather than a competing
host taxonomy.

### Batch 5.2 - Runtime Diagnostic Depth

- [x] materialize the causal receipts in runtime and supervisor surfaces
- [x] keep local and server host exports aligned with the new meaning

Batch 5.2 added `RuntimeFaultDiagnosticReceipt`,
`RuntimeFaultContributionReceipt`, and the shared runtime-versus-host authority
split directly to `signal-runtime`, then threaded that receipt family through
`RuntimeObservationReport`, `RuntimeSupervisorReport`, and
`RuntimeProfilingReceipt`. The runtime now selects one canonical
`primary_family` from runtime-owned fault posture while preserving xrun,
plugin-boundary, device-path, deferred-work, and advisory callback evidence as
typed contributions instead of forcing products to infer cause from unrelated
counters. Downstream-style runtime proofs and stable host-edge proofs now read
that same receipt family without host-local reclassification.

### Batch 5.3 - Consumer Proof

- [x] add focused proof that causal fault receipts remain consumable through
  Signal-owned boundaries without private host logic

Batch 5.3 added the first dedicated consumer-facing boundary for fault-cause
receipts. Downstream-style runtime proofs now exercise canonical
`primary_family` export directly, stable local/server host-edge proofs forward
the same receipt through `supervisor_report()`, and
`signal-supervisor-tools --describe-fault-diagnostic-boundary` plus
`effigy acceptance:fault-diagnostic-boundary --repo .` make the proof boundary
runnable without private implementation detail.

## Acceptance Criteria

- [x] Signal exposes typed runtime fault-cause receipts
- [x] later profiling and soak work can cite causal evidence directly
- [x] host products no longer need to infer cause from unrelated counters

## Risks And Mitigations

- Risk: causal receipts duplicate existing diagnostics noisily.
- Mitigation: freeze a small, explainable receipt family first.
- Risk: products keep preferring legacy summaries.
- Mitigation: prove the new receipts through public runtime/export surfaces.

## Evidence Requirements

- [x] log each meaningful causal-diagnostics tranche
- [x] run focused runtime/export validation for causal receipts
- [x] record deferred diagnostics breadth explicitly

## Next Task

Continue `g06.006` with Batch 6.1 by defining the first runtime-owned per-block
execution timing and pressure snapshot contract so the newly closed
fault-diagnostic boundary can feed into bounded timing instrumentation rather
than counter-only performance anecdotes.
