# 019 - Fault-Injection Harnesses And Multi-Backend Acceptance Depth

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g06.005, g06.011, g06.016, g06.018
Vision tags: `ACCEPTANCE`, `RECOVERY`, `BACKENDS`

## Problem

`g06` will widen both runtime hardening and actual feature breadth. The
generation therefore needs a stronger reusable acceptance surface that can
inject realistic faults and prove behavior across multiple adapters and
runtime-service lanes.

## Goals

- [x] define reusable fault-injection and integrated acceptance scenarios for
  the widened `g06` surface
- [x] cover recovery, adapter breadth, hardware, and media-service behavior in
  one stronger acceptance lane
- [x] keep the evidence machine-readable and repo-owned

## Non-Goals

- [ ] no product-specific acceptance dashboards
- [ ] no full certification matrix across every environment

## Execution Plan

### Batch 19.1 - Harness Scope Contract

- [x] define the key fault-injection and multi-backend acceptance scenarios
- [x] separate required acceptance depth from optional longer-running soak paths

### Batch 19.2 - Harness Implementation

- [x] implement reusable fault-injection fixtures and acceptance tasks
- [x] keep outputs typed through supervisor tools and Effigy surfaces

### Batch 19.3 - Integrated Evidence Proof

- [x] add focused proofs that the widened runtime and adapter surface now has
  meaningful integrated evidence rather than only milestone-local checks

## Acceptance Criteria

- [x] Signal has reusable fault-injection and integrated acceptance depth
- [x] multi-backend, hardware, and media-service behavior have cross-cutting evidence
- [x] later closeout and downstream consumers can rely on typed acceptance receipts

## Risks And Mitigations

- Risk: acceptance breadth becomes vague integration sprawl.
- Mitigation: freeze a bounded scenario set with required versus optional depth.
- Risk: harnesses depend on private scripts or local operator steps.
- Mitigation: keep outputs repo-owned, typed, and runnable through Effigy/tasks.

## Evidence Requirements

- [x] log each meaningful fault-injection tranche
- [x] run focused integrated acceptance validation
- [x] record explicit deferred soak depth that remains optional

## Batch 19.1 Outcome

Batch 19.1 freezes the first reusable integrated acceptance policy for `g06`
in `docs/contracts/030-fault-injection-harness-and-multi-backend-acceptance-contract.md`.
That contract fixes the authority chain between closed `g06` boundaries, typed
runtime receipts, machine-readable `signal-supervisor-tools` descriptors, and
repo-owned Effigy tasks for fault-injection and integrated acceptance claims.

It also freezes five scenario families that later batches must compose:
recovery and fault attribution, scheduling and execution pressure, adapter and
portability breadth, hardware and external-I/O continuity, and media plus
analysis-service continuity. Most importantly, the contract now makes
`required`, `advisory`, and `deferred` integrated evidence explicit so later
implementation does not blur bounded acceptance with the longer-session soak
policy reserved for `g06.020`.

## Batch 19.2 Outcome

Batch 19.2 turns that contract into a real shared lane. `signal-supervisor-tools`
now exposes the machine-readable
`signal.runtime.integrated-acceptance-lane` descriptor, and Effigy now owns a
grouped `acceptance:integrated-acceptance-lane` task that composes the required
cross-family path from the already-closed interruption, diagnostics,
critical-path, deferred-work, plugin-continuity, parity, supervision,
clock-topology, external-I/O, media-service, and analysis-metadata boundaries.

The lane also keeps advisory depth explicit instead of quietly burying it:
recording continuity, offline render continuity, VST3, AU, generic-event, and
recall-portability checks remain visible but non-blocking. During this batch,
the grouped lane exposed stale watchdog-restart expectations in the public and
internal interruption proofs, and those proofs were repaired to match the
current safe-mode restart threshold rather than silently removing interruption
from the required path.

## Batch 19.3 Outcome

Batch 19.3 closes `g06.019` with a real integrated evidence artifact. The
shared lane now points at a focused `signal-supervisor-tools` export proof that
surfaces recovery and interruption state, deferred-work pressure, adapter
breadth, device supervision, external-I/O clocking, and media plus
analysis-library receipts together in one `signal.supervisor.export` payload.

That means the bounded integrated lane is no longer just a grouped set of
boundary tasks: it now has one machine-checked cross-family export proof and a
matching Effigy lane that keep the required path tied to combined runtime-owned
receipts rather than isolated milestone-local checks.

## Next Task

Continue `g06.020` with Batch 20.1 by freezing the bounded long-session soak,
promotion-gate, and Loophole-readiness policy on top of the now-closed
integrated acceptance lane.
