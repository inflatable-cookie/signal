# 007 - LV2 Adapter Baseline And Linux-Native Plugin Lifecycle Depth

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g06.011
Vision tags: `PLUGINS`, `LINUX`, `LV2`

## Problem

Chorus still points to LV2 as part of the Linux plugin story, but Signal's
current planned breadth stops at CLAP, VST3, and AU.

## Goals

- [ ] introduce the first real LV2 adapter baseline
- [ ] make Linux-native plugin breadth explicit rather than implied
- [ ] keep lifecycle, capability, and sandbox meaning aligned with the existing
  backend-neutral plugin contract

## Non-Goals

- [ ] no Linux-only product workflow behavior
- [ ] no adapter-private behavior promoted accidentally

## Execution Plan

### Batch 7.1 - LV2 Contract Alignment

- [ ] map LV2-specific details onto the backend-neutral capability and lifecycle contract
- [ ] record explicit contract gaps before runtime realization widens

### Batch 7.2 - Runtime Adapter Baseline

- [ ] add the first LV2 adapter path with runtime-owned discovery, lifecycle, and transport integration
- [ ] keep supervisor export and host-edge surfaces aligned with the new path

### Batch 7.3 - Conformance Proof

- [ ] add focused proofs for Linux-native LV2 discovery, lifecycle, and export behavior

## Acceptance Criteria

- [ ] Signal has a real LV2 adapter baseline
- [ ] Linux-native plugin breadth is explicit at the runtime boundary
- [ ] LV2 lifecycle and capability surfaces align with the shared contract

## Risks And Mitigations

- Risk: LV2 work reopens adapter-private ownership.
- Mitigation: force widened behavior through the backend-neutral contract.

## Evidence Requirements

- [ ] log each meaningful LV2 tranche
- [ ] run focused LV2 discovery, lifecycle, and export validation
- [ ] record deferred LV2 breadth explicitly

## Next Task

Continue `g07.008` by reconciling Linux cross-adapter parity and sandbox policy
across CLAP, VST3, and LV2.

