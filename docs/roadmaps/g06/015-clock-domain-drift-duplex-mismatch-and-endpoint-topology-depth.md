# 015 - Clock-Domain Drift, Duplex Mismatch, And Endpoint-Topology Depth

Status: planned
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

- [ ] define runtime-owned clock-domain and endpoint-topology semantics
- [ ] expose drift, resync, duplex mismatch, and partial availability receipts
- [ ] prepare the substrate for external-I/O and monitoring depth

## Non-Goals

- [ ] no network-audio or remote clock-distribution scope yet
- [ ] no hardware-control-surface feature work

## Execution Plan

### Batch 15.1 - Clock And Endpoint Contract

- [ ] define drift, discontinuity, duplex mismatch, and endpoint-topology
  vocabulary
- [ ] align the contract with supervision and recovery semantics from `g06.014`

### Batch 15.2 - Runtime Portability Depth

- [ ] deepen runtime clock-domain and endpoint-topology receipts
- [ ] keep host-edge and supervisor exports aligned to the same meaning

### Batch 15.3 - Focused Portability Proof

- [ ] add focused proofs for drift, duplex mismatch, and endpoint-topology
  observation paths

## Acceptance Criteria

- [ ] Signal has explicit clock-domain and endpoint-topology receipts
- [ ] downstream consumers can observe drift and mismatch behavior clearly
- [ ] later external-I/O work can build on one runtime-owned topology model

## Risks And Mitigations

- Risk: clocking detail becomes backend-specific noise.
- Mitigation: freeze bounded runtime-owned semantics instead of backend internals.
- Risk: products keep encoding endpoint topology locally.
- Mitigation: require shared topology receipts at the consumer boundary.

## Evidence Requirements

- [ ] log each meaningful clocking/topology tranche
- [ ] run focused validation for drift and endpoint observations
- [ ] record deferred network-audio scope explicitly

## Next Task

Continue `g06.016` by turning stronger endpoint topology into external-I/O and
monitoring contracts products can actually consume.
