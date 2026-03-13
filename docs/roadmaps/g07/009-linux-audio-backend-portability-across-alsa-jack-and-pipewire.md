# 009 - Linux Audio Backend Portability Across ALSA, JACK, And PipeWire

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g06.014, g06.015
Vision tags: `LINUX`, `HARDWARE`, `BACKENDS`

## Problem

Signal's current backend portability work does not yet provide a deliberate
Linux-native hardware story across ALSA, JACK, and PipeWire.

## Goals

- [ ] define the first explicit Linux audio backend portability surface
- [ ] support ALSA, JACK, and PipeWire under one runtime-owned contract
- [ ] keep hardware, diagnostics, and restart semantics coherent across Linux backends

## Non-Goals

- [ ] no exhaustive distro certification matrix
- [ ] no product-specific Linux setup UX

## Execution Plan

### Batch 9.1 - Linux Backend Contract

- [ ] define backend identity, capability, and lifecycle meaning across ALSA, JACK, and PipeWire
- [ ] align the contract with the existing hardware portability model

### Batch 9.2 - Backend Baselines

- [ ] add the first credible Linux backend baselines as needed
- [ ] keep diagnostics, restart policy, and host-edge receipts aligned

### Batch 9.3 - Focused Proof

- [ ] add focused proofs for Linux backend portability and fallback behavior

## Acceptance Criteria

- [ ] Signal has an explicit Linux hardware backend portability surface
- [ ] Linux hardware behavior stays runtime-owned and inspectable
- [ ] later endpoint-topology and control-surface work can build on the same base

## Risks And Mitigations

- Risk: Linux backend work fragments into backend-private shells.
- Mitigation: freeze one hardware contract first and prove widened paths through it.

## Evidence Requirements

- [ ] log each meaningful Linux backend tranche
- [ ] run focused ALSA, JACK, and PipeWire validation as available
- [ ] record deferred Linux backend breadth explicitly

## Next Task

Continue `g07.010` by reconciling clocking, duplex, and endpoint-topology
behavior across the widened Linux backend set.

