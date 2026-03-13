# 014 - Device Supervision, Restart-State Machine, And Fault-Boundary Depth

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g06.005
Vision tags: `HARDWARE`, `RECOVERY`, `RUNTIME`

## Problem

Signal already owns basic hardware portability and device-backed runtime state,
but Loophole's next runtime lane needs deeper reusable supervision and restart
policy so products do not keep carrying too much restart logic.

## Goals

- [ ] define a stronger runtime-owned device supervision and restart-state model
- [ ] make recovering-versus-faulted hardware behavior explicit
- [ ] support later monitoring, external-I/O, and soak work on top of one
  supervision substrate

## Non-Goals

- [ ] no product-specific device setup UX
- [ ] no exhaustive hardware certification matrix

## Execution Plan

### Batch 14.1 - Supervision Contract

- [ ] define restart states, recovery exhaustion, and fault-boundary semantics
- [ ] align device supervision with the shared interruption and fault taxonomy

### Batch 14.2 - Runtime Supervision Depth

- [ ] materialize stronger device supervision and restart receipts in runtime
- [ ] keep host-edge and supervisor exports aligned with the new state model

### Batch 14.3 - Focused Recovery Proof

- [ ] add focused proofs for recovery, exhaustion, and explicit faulted hardware
  outcomes

## Acceptance Criteria

- [ ] Signal has a stronger device supervision and restart-state model
- [ ] products can observe hardware recovery truth without owning restart policy
- [ ] later hardware/monitoring work builds on reusable runtime supervision

## Risks And Mitigations

- Risk: supervision policy stays backend-private.
- Mitigation: freeze backend-neutral restart and fault semantics first.
- Risk: recovery logic still lives mostly in host wrappers.
- Mitigation: require runtime-owned receipts for restart and exhaustion outcomes.

## Evidence Requirements

- [ ] log each meaningful hardware-supervision tranche
- [ ] run focused validation for recovery and exhaustion behavior
- [ ] record deferred hardware-matrix breadth explicitly

## Next Task

Continue `g06.015` by deepening clock-domain and endpoint-topology behavior on
top of the stronger supervision substrate.
