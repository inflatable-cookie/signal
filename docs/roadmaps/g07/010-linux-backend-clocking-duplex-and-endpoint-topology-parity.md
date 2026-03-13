# 010 - Linux Backend Clocking, Duplex, And Endpoint-Topology Parity

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g07.009
Vision tags: `LINUX`, `CLOCKING`, `HARDWARE`

## Problem

Backend presence alone is not enough. Linux needs parity on clocking, duplex,
and endpoint-topology behavior before Loophole can trust it as a real runtime
target.

## Goals

- [ ] define Linux parity for clocking, duplex, and endpoint-topology behavior
- [ ] align Linux backend observation with the shared hardware recovery model
- [ ] keep host-visible latency, drift, and mismatch state explicit

## Non-Goals

- [ ] no network-audio topology work here
- [ ] no product-local device setup UX

## Execution Plan

### Batch 10.1 - Parity Contract

- [ ] define Linux parity expectations for clocking, duplex, and endpoint topology
- [ ] classify backend-private behavior explicitly

### Batch 10.2 - Runtime Depth

- [ ] align Linux backend observation and recovery receipts with the parity contract
- [ ] keep host-edge surfaces on one hardware vocabulary

### Batch 10.3 - Focused Proof

- [ ] add focused proofs for Linux clocking, duplex, and endpoint-topology behavior

## Acceptance Criteria

- [ ] Signal has explicit Linux hardware parity for clocking and topology
- [ ] later external-I/O and control-surface depth can rely on the same Linux base
- [ ] unsupported Linux behavior remains explicit rather than implied

## Risks And Mitigations

- Risk: Linux parity claims overreach actual runtime evidence.
- Mitigation: require focused proof and unsupported-state receipts.

## Evidence Requirements

- [ ] log each meaningful Linux parity tranche
- [ ] run focused Linux hardware parity validation
- [ ] record explicit unsupported parity explicitly

## Next Task

Continue `g07.011` by opening the external MIDI endpoint graph and device-identity baseline.

