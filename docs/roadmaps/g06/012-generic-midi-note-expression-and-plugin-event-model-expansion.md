# 012 - Generic MIDI, Note-Expression, And Plugin-Event Model Expansion

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g06.009, g06.010, g06.011
Vision tags: `MIDI`, `PLUGINS`, `EVENTS`

## Problem

Signal's current generic event layer is still relatively narrow and shaped by
the CLAP-first path. Chorus still points to MIDI/editor and bounded plugin
feature depth, so Signal needs a stronger reusable MIDI and event model before
products build richer behavior on adapter-local packet semantics.

## Goals

- [ ] define a stronger generic MIDI, note-expression, and plugin-event model
- [ ] give `signal-midi`-class functionality a real reusable runway inside the
  Signal workspace
- [ ] keep event translation and transport semantics runtime-owned across
  adapters

## Non-Goals

- [ ] no MIDI editor or arranger UX work
- [ ] no product-local controller mapping surface

## Execution Plan

### Batch 12.1 - Generic Event Contract

- [ ] define the widened MIDI, note-expression, and plugin-event vocabulary
- [ ] classify what remains adapter-private versus shared event meaning

### Batch 12.2 - Runtime And Adapter Depth

- [ ] materialize the widened event model through runtime, adapter, and
  host-edge surfaces
- [ ] keep transport and scheduling semantics aligned with the richer event path

### Batch 12.3 - Boundary Proof

- [ ] add focused proofs that downstream consumers can inspect and use the
  richer event model without CLAP/VST3/AU packet reconstruction

## Acceptance Criteria

- [ ] Signal has a stronger generic MIDI and plugin-event model
- [ ] later products can build richer MIDI/plugin workflows on reusable surfaces
- [ ] widened event semantics remain adapter-neutral at the consumer boundary

## Risks And Mitigations

- Risk: event expansion drifts into editor features instead of runtime substrate.
- Mitigation: keep the milestone on reusable event contracts and transport semantics.
- Risk: adapter-specific packet shapes leak into the public boundary.
- Mitigation: require one generic event vocabulary first.

## Evidence Requirements

- [ ] log each meaningful MIDI/event tranche
- [ ] run focused validation for widened event translation and export
- [ ] record deferred MIDI/control-surface depth explicitly

## Next Task

Continue `g06.013` by making plugin state and preset interchange more portable
across the now-wider adapter set.
