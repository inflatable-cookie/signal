# 009 - VST3 Adapter Baseline And Runtime-Owned Lifecycle Depth

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g05.001, g06.003
Vision tags: `PLUGINS`, `BACKENDS`, `VST3`

## Problem

Signal's plugin contract is backend-neutral, but actual adapter realization is
still CLAP-first. Loophole's feature front still needs a real VST3 path inside
Signal's runtime-owned lifecycle model.

## Goals

- [ ] introduce the first real VST3 adapter baseline inside Signal-owned crates
- [ ] keep lifecycle, fault, discovery, and capability meaning aligned with the
  existing backend-neutral contract
- [ ] make the VST3 path credible on Linux as well as other supported host
  platforms rather than leaving Linux plugin support implicit
- [ ] avoid pushing VST3 ownership into product-local wrappers

## Non-Goals

- [ ] no product-specific plugin browser or preset UX
- [ ] no format-specific behavior promoted to the shared contract by accident

## Execution Plan

### Batch 9.1 - VST3 Adapter Contract Alignment

- [ ] map VST3-specific details onto the existing backend-neutral capability
  and lifecycle contract
- [ ] record any explicit contract gaps before runtime realization widens

### Batch 9.2 - Runtime Adapter Baseline

- [ ] add the first VST3 adapter path with runtime-owned discovery, lifecycle,
  and transport/session integration
- [ ] cover platform-specific scan/load paths needed for Linux-hosted VST3 use
  without changing the shared runtime contract
- [ ] keep supervisor export and host-edge receipts aligned with the new path

### Batch 9.3 - Conformance Proof

- [ ] add focused proofs showing the VST3 path remains consumable through
  Signal-owned runtime/export surfaces without host-local reconstruction

## Acceptance Criteria

- [ ] Signal has a real VST3 adapter baseline
- [ ] the VST3 path includes explicit Linux-hosted plugin coverage rather than
  only package-map intent
- [ ] VST3 lifecycle and capability surfaces align with the shared contract
- [ ] later cross-adapter breadth can build on runtime-owned receipts

## Risks And Mitigations

- Risk: VST3 work reopens format-specific ownership.
- Mitigation: force all widened surfaces through the existing backend-neutral contract.
- Risk: CLAP-first assumptions leak into VST3 behavior silently.
- Mitigation: require explicit conformance proof on the widened path.

## Evidence Requirements

- [ ] log each meaningful VST3 tranche
- [ ] run focused validation for runtime-owned VST3 discovery/lifecycle/export
- [ ] record explicit deferred VST3 breadth that remains out of scope

## Next Task

Continue `g06.010` by opening the same baseline for AU so the bounded
AU/CLAP/VST3 scope becomes real rather than aspirational.
