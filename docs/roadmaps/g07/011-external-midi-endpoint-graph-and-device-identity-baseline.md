# 011 - External MIDI Endpoint Graph And Device-Identity Baseline

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g06.012, g06.016
Vision tags: `MIDI`, `HARDWARE`, `IDENTITY`

## Problem

Loophole's next hardware and controller depth needs a reusable Signal-owned
model for external MIDI endpoints, identity, capability, and routing.

## Goals

- [ ] define a reusable external MIDI endpoint graph and identity surface
- [ ] support runtime-owned MIDI endpoint discovery and routing semantics
- [ ] keep host-visible device and endpoint state explicit

## Non-Goals

- [ ] no product-specific MIDI browser or mapping UX
- [ ] no control-surface scripting depth yet

## Execution Plan

### Batch 11.1 - Endpoint Contract

- [ ] define MIDI endpoint identity, topology, capability, and lifecycle meaning
- [ ] align the contract with existing hardware and event models

### Batch 11.2 - Runtime Baseline

- [ ] implement the first credible external MIDI endpoint graph baseline
- [ ] keep discovery, health, and routing observation aligned with the contract

### Batch 11.3 - Focused Proof

- [ ] add focused proofs for external MIDI endpoint discovery and routing behavior

## Acceptance Criteria

- [ ] Signal has an explicit external MIDI endpoint graph and identity surface
- [ ] later control-surface and richer controller-expression work can build on it
- [ ] hosts can observe MIDI endpoint truth without local shims

## Risks And Mitigations

- Risk: MIDI hardware depth gets rebuilt as app-local glue.
- Mitigation: freeze one reusable endpoint graph and capability contract first.

## Evidence Requirements

- [ ] log each meaningful external-MIDI tranche
- [ ] run focused endpoint and routing validation
- [ ] record deferred MIDI device breadth explicitly

## Next Task

Continue `g07.012` by widening the event layer into MIDI 2.0, MPE, and richer
controller-expression depth.

