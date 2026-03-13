# 011 - Backend Capability Parity, Linux Plugin Support, And Cross-Adapter Conformance Depth

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g06.009, g06.010
Vision tags: `PLUGINS`, `BACKENDS`, `CONFORMANCE`

## Problem

Adding VST3 and AU baselines is not enough by itself. Signal still needs one
cross-adapter capability and conformance surface so consumers know which plugin
behaviors are genuinely portable and which remain adapter-private.

## Goals

- [ ] define cross-adapter capability parity expectations across CLAP, VST3,
  and AU
- [ ] make Linux-hosted plugin support explicit where CLAP or VST3 are
  expected to carry it, instead of leaving Linux platform breadth as an
  unstated side effect of adapter work
- [ ] make runtime-owned portability and fallback behavior explicit
- [ ] keep format-scoped plugin isolation behavior explicit where policy rules
  depend on adapter identity such as CLAP, VST3, or AU
- [ ] keep discovery, lifecycle, render, and failure semantics coherent across
  the widened adapter set

## Non-Goals

- [ ] no feature-matrix marketing artifact detached from runtime reality
- [ ] no product-local fallback rules

## Execution Plan

### Batch 11.1 - Capability Parity Contract

- [ ] define the portable capability and fallback matrix across CLAP, VST3, and AU
- [ ] classify what remains adapter-private after the widened baseline

### Batch 11.2 - Runtime Parity Depth

- [ ] align discovery, lifecycle, render, and failure receipts across adapters
- [ ] align adapter identity with the shared placement-policy surface so
  by-format isolation remains runtime-owned rather than host-invented
- [ ] add explicit platform-coverage and unsupported-platform reporting where
  Linux differs from macOS or Windows adapter breadth
- [ ] keep supervisor export and host-edge surfaces on one cross-adapter vocabulary

### Batch 11.3 - Cross-Adapter Proof

- [ ] add focused proofs that the widened adapter set stays consumable through
  Signal-owned capability and export surfaces

## Acceptance Criteria

- [ ] Signal has an explicit cross-adapter capability parity surface
- [ ] Linux plugin support is explicit and inspectable at the same consumer
  boundary as format breadth
- [ ] by-format isolation policy remains explicit and reusable across the
  widened adapter set
- [ ] wider plugin support does not reopen host-local ownership
- [ ] later consumers can rely on one portable capability vocabulary

## Risks And Mitigations

- Risk: parity work devolves into adapter-specific edge-case sprawl.
- Mitigation: freeze one bounded portable capability/fallback contract first.
- Risk: consumers overread unsupported parity claims.
- Mitigation: require explicit runtime-owned fallback and unsupported-state receipts.

## Evidence Requirements

- [ ] log each meaningful parity tranche
- [ ] run focused cross-adapter conformance validation
- [ ] record explicit unsupported parity that remains out of scope

## Next Task

Continue `g06.012` by widening the generic event layer so the broader adapter
surface is not limited to the current narrower CLAP-first packet semantics.
