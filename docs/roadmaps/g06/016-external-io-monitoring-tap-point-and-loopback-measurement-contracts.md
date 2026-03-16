# 016 - External I/O, Monitoring Tap-Point, And Loopback Measurement Contracts

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g06.014, g06.015
Vision tags: `HARDWARE`, `MONITORING`, `IO`

## Problem

Chorus still needs stronger reusable external-I/O, monitoring, and measurement
substrate. Signal does not yet expose enough runtime-owned truth for monitor
paths, tap points, loopback, or calibration-friendly observation.

## Goals

- [ ] define external-I/O, monitoring, tap-point, and loopback measurement
  semantics
- [ ] expose host-consumable runtime-owned monitoring and endpoint state
- [ ] support later calibration, monitoring, and hardware-depth work without
  host-local models

## Non-Goals

- [ ] no full control-surface or room-correction product scope
- [ ] no network-audio topology work yet

## Execution Plan

### Batch 16.1 - Monitoring And Loopback Contract

- [x] define external-I/O roles, monitor tap points, loopback, and measurement
  vocabulary
- [x] decide what belongs in runtime-facing versus supervisor export surfaces

### Batch 16.2 - Runtime I/O Depth

- [x] materialize monitoring and loopback receipts on top of the stronger
  endpoint-topology substrate
- [x] keep local and server host consumers aligned to the same model

### Batch 16.3 - Consumer Proof

- [x] add focused proofs that downstream consumers can inspect monitoring and
  loopback state without local reconstruction

## Acceptance Criteria

- [ ] Signal has explicit monitoring, tap-point, and loopback contracts
- [ ] products can observe external-I/O and monitor state through reusable surfaces
- [ ] later Loophole monitoring and hardware work can build on Signal substrate

## Risks And Mitigations

- Risk: monitoring work becomes product UX scope.
- Mitigation: keep the milestone on runtime-owned endpoint and measurement truth.
- Risk: loopback/calibration semantics stay ad hoc and local.
- Mitigation: freeze typed roles and receipts first.

## Evidence Requirements

- [x] log each meaningful external-I/O tranche
- [x] run focused validation for monitoring and loopback surfaces
- [ ] record deferred control-surface breadth explicitly

## Batch 16.1 Outcome

Batch 16.1 froze the first runtime-owned monitoring and loopback contract in
`docs/contracts/027-external-io-monitoring-tap-point-and-loopback-measurement-contract.md`.
That contract now fixes the authority line between `signal-hardware`,
`signal-runtime`, and shared host surfaces for external-I/O roles, monitor tap
points, loopback paths, measurement sessions, and reference-path meaning. It
also makes the runtime-versus-supervisor split explicit before any DTO work
widens, so Batch 16.2 can deepen one bounded monitoring boundary instead of
reopening clock-topology or supervision semantics.

## Batch 16.2 Outcome

Batch 16.2 made the external-I/O seam real instead of contract-only. Signal
now carries runtime-owned monitoring, tap-point, loopback, and primary-role
meaning on `RuntimeExternalIoSnapshot`, and that receipt is present even when
no live host observation is available through an explicit `Unavailable`
classification rather than a missing field. `RuntimeObservationReport` now
exports the shared snapshot directly, `signal-host-local` feeds the live host-I/O
summary into that runtime-owned receipt family, and `signal-host-server`
stays aligned by exporting the same snapshot shape with bounded unavailable
state instead of inventing a private server-only monitoring model.

Focused validation covered the receipt builder in `signal-runtime`, the
topology-aware and degraded local host cases, the explicit unavailable server
host case, and compile coverage for the touched supervisor path without
widening into unrelated suites.

## Batch 16.3 Outcome

Batch 16.3 closed the shared consumer proof boundary for external-I/O,
monitoring, tap-point, and loopback receipts. The downstream-style runtime
proof now covers `RuntimeObservationReport::external_io_snapshot`, the stable
local host edge proves direct and faulted external-I/O truth through
`LocalRuntimeHost::supervisor_report()`, and the stable server host edge proves
the same runtime-owned receipt family exposes explicit `Unavailable`
monitoring and loopback state instead of a host-private fallback model.
`signal-supervisor-tools` now exposes a machine-readable
`signal.runtime.external-io-boundary` descriptor, and the repo-owned
`effigy acceptance:external-io-boundary --repo .` task keeps that proof seam
runnable.

## Next Task

Continue `g06.018` with Batch 18.1 by freezing the first reusable
analysis-metadata and library-service descriptor family on top of the closed
media-service boundary.
