# 016 - External I/O, Monitoring Tap-Point, And Loopback Measurement Contracts

Status: planned
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

- [ ] define external-I/O roles, monitor tap points, loopback, and measurement
  vocabulary
- [ ] decide what belongs in runtime-facing versus supervisor export surfaces

### Batch 16.2 - Runtime I/O Depth

- [ ] materialize monitoring and loopback receipts on top of the stronger
  endpoint-topology substrate
- [ ] keep local and server host consumers aligned to the same model

### Batch 16.3 - Consumer Proof

- [ ] add focused proofs that downstream consumers can inspect monitoring and
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

- [ ] log each meaningful external-I/O tranche
- [ ] run focused validation for monitoring and loopback surfaces
- [ ] record deferred control-surface breadth explicitly

## Next Task

Continue `g06.017` by building the media-service substrate that Loophole still
needs for waveform, preview, and asset readiness depth.
