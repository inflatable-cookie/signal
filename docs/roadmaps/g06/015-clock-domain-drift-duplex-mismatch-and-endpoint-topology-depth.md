# 015 - Clock-Domain Drift, Duplex Mismatch, And Endpoint-Topology Depth

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g06.014
Vision tags: `HARDWARE`, `CLOCKING`, `IO`

## Problem

Signal's hardware baseline still needs a stronger answer for drift,
discontinuity, duplex mismatch, endpoint changes, and partial device topology
availability so products can reason about real hardware behavior without local
guesswork.

## Goals

- [x] define runtime-owned clock-domain and endpoint-topology semantics
- [x] expose drift, resync, duplex mismatch, and partial availability receipts
- [x] prepare the substrate for external-I/O and monitoring depth

## Non-Goals

- [ ] no network-audio or remote clock-distribution scope yet
- [ ] no hardware-control-surface feature work

## Execution Plan

### Batch 15.1 - Clock And Endpoint Contract

- [x] define drift, discontinuity, duplex mismatch, and endpoint-topology
  vocabulary
- [x] align the contract with supervision and recovery semantics from `g06.014`

### Batch 15.2 - Runtime Portability Depth

- [x] deepen runtime clock-domain and endpoint-topology receipts
- [x] keep host-edge and supervisor exports aligned to the same meaning

### Batch 15.3 - Focused Portability Proof

- [x] add focused proofs for drift, duplex mismatch, and endpoint-topology
  observation paths

## Acceptance Criteria

- [x] Signal has explicit clock-domain and endpoint-topology receipts
- [x] downstream consumers can observe drift and mismatch behavior clearly
- [x] later external-I/O work can build on one runtime-owned topology model

## Risks And Mitigations

- Risk: clocking detail becomes backend-specific noise.
- Mitigation: freeze bounded runtime-owned semantics instead of backend internals.
- Risk: products keep encoding endpoint topology locally.
- Mitigation: require shared topology receipts at the consumer boundary.

## Evidence Requirements

- [x] log each meaningful clocking/topology tranche
- [x] run focused validation for drift and endpoint observations
- [x] record deferred network-audio scope explicitly

## Batch 15.1 Outcome

Batch 15.1 froze the first bounded runtime-owned contract for clock drift,
discontinuity, duplex mismatch, endpoint topology, partial availability, and
resync in
`docs/contracts/026-clock-domain-drift-duplex-mismatch-and-endpoint-topology-contract.md`.
That contract now fixes the authority chain between `signal-hardware`,
`signal-runtime`, and shared host surfaces, and explicitly composes drift or
topology meaning with the closed `g06.014` device-supervision boundary instead
of leaving later hardware work free to invent a parallel fault model.

## Batch 15.2 Outcome

Batch 15.2 turned that contract into a real receipt family by extending
`RuntimeHostClockingSummary` and the derived `RuntimeExternalIoSnapshot` with
typed drift, discontinuity, duplex-mismatch, endpoint-topology, and
partial-availability classification. `signal-host-local` now derives those
fields in one bounded host-I/O path and reuses the same `host_io` receipt for
both the shared supervisor observation and the stable host-edge wrapper, so the
first live report no longer diverges between inner and outer clocking state.

## Batch 15.3 Outcome

Batch 15.3 closed the shared clock-topology consumer seam around one bounded
machine-readable boundary:

- public runtime proof now covers drift, duplex-mismatch, and endpoint-topology
  truth on `RuntimeHostObservationReport` and `RuntimeHostSupervisorReport`
- the stable local host edge now proves steady and explicit faulted
  clock-topology export through `LocalRuntimeHost::host_supervisor_report()`
- `signal-supervisor-tools` now exposes
  `signal.runtime.clock-topology-boundary` and the repo-owned acceptance task
  `effigy acceptance:clock-topology-boundary --repo .`
- the richer duplex cross-clock and partial-availability cases stay aligned to
  the same shared boundary through focused local-host proofs instead of
  host-local reconstruction

## Next Task

Continue `g06.016` with Batch 16.1 by freezing the external-I/O, monitoring
tap-point, and loopback measurement contract on top of the closed `g06.015`
clock-domain and endpoint-topology boundary.
