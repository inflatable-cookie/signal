# 006 - Surround Bed, Object, And Mix-Policy Expansion

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g07.005
Vision tags: `SPATIAL`, `MULTICHANNEL`, `MIX`

## Problem

A narrow spatial baseline is not enough for later routing and immersive depth.
Signal needs a bounded but reusable follow-on surface for richer bed, object,
and mix-policy behavior.

## Goals

- [ ] widen spatial execution into richer bed, object, and mix-policy meaning
- [ ] keep immersive or surround behavior runtime-owned and inspectable
- [ ] prepare the stack for later product-level spatial workflows without host shims

## Non-Goals

- [ ] no exhaustive immersive-format certification matrix
- [ ] no product-local speaker-room workflow here

## Execution Plan

### Batch 6.1 - Expanded Spatial Contract

- [ ] define bed, object, and mix-policy semantics on top of the baseline adapter model
- [ ] keep fallback and unsupported-state behavior explicit

### Batch 6.2 - Runtime Expansion

- [ ] implement the first bounded richer spatial path
- [ ] keep multichannel and routing receipts aligned with the widened model

### Batch 6.3 - Focused Proof

- [ ] add focused proofs for richer spatial and mix-policy behavior

## Acceptance Criteria

- [ ] Signal has an explicit richer spatial expansion path
- [ ] later product workflows can build on runtime-owned spatial truth
- [ ] immersive or surround behavior remains inspectable and bounded

## Risks And Mitigations

- Risk: richer spatial work drifts into product UX or room-design scope.
- Mitigation: keep the queue on execution, policy, and receipts only.

## Evidence Requirements

- [ ] log each meaningful richer-spatial tranche
- [ ] run focused spatial and mix-policy validation
- [ ] record deferred immersive breadth explicitly

## Next Task

Continue `g07.007` by widening plugin breadth on Linux through the LV2 adapter
baseline.

