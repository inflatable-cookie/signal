# 012 - MIDI 2.0, MPE, And Richer Controller-Expression Depth

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g07.011, g06.012
Vision tags: `MIDI`, `EVENTS`, `EXPRESSION`

## Problem

The current generic event layer is broader than the earliest CLAP-first path,
but it still does not cover the fuller controller-expression surface Loophole
will eventually want.

## Goals

- [ ] widen the generic event model toward MIDI 2.0, MPE, and richer controller expression
- [ ] keep plugin, device, and runtime event meaning aligned across the widened surface
- [ ] avoid reopening adapter-private event semantics in consumer code

## Non-Goals

- [ ] no full notation or score-editing workflow work
- [ ] no product-local expressive-controller UX here

## Execution Plan

### Batch 12.1 - Event Contract Expansion

- [ ] define the widened controller-expression and MIDI depth contract
- [ ] align plugin and device receipts with the new event meaning

### Batch 12.2 - Runtime And Adapter Depth

- [ ] implement the first credible widened event path
- [ ] keep runtime, plugin, and hardware surfaces on one event vocabulary

### Batch 12.3 - Focused Proof

- [ ] add focused proofs for widened controller-expression behavior

## Acceptance Criteria

- [ ] Signal has an explicit richer controller-expression surface
- [ ] later control-surface and device work can reuse the same event vocabulary
- [ ] hosts do not need adapter-specific event shims to consume the new depth

## Risks And Mitigations

- Risk: event breadth becomes speculative and detached from runtime use.
- Mitigation: require direct mapping to runtime, plugin, and hardware receipts.

## Evidence Requirements

- [ ] log each meaningful controller-expression tranche
- [ ] run focused widened-event validation
- [ ] record deferred expressive-event breadth explicitly

## Next Task

Continue `g07.013` by turning the stronger MIDI and hardware substrate into a
control-surface transport, mapping, and feedback baseline.

