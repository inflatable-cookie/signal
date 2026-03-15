# 012 - Generic MIDI, Note-Expression, And Plugin-Event Model Expansion

Status: complete
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

- [x] define the widened MIDI, note-expression, and plugin-event vocabulary
- [x] classify what remains adapter-private versus shared event meaning

### Batch 12.2 - Runtime And Adapter Depth

- [x] materialize the widened event model through runtime, adapter, and
  host-edge surfaces
- [x] keep transport and scheduling semantics aligned with the richer event path

### Batch 12.3 - Boundary Proof

- [x] add focused proofs that downstream consumers can inspect and use the
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

## Batch 12.1 Outcome

Batch 12.1 freezes the widened generic event contract in
`docs/contracts/023-generic-midi-note-expression-and-plugin-event-model-contract.md`.
The repo now has an explicit rule set for:

- treating `PluginEvent` and `EventPacket` as the shared Signal-owned event
  vocabulary instead of leaving generic event meaning CLAP-first and implicit
- separating portable, format-guarded, adapter-private, and unsupported event
  scope across CLAP, VST3, and AU
- keeping event timing, transport, and future export ownership inside
  `signal-plugin` and `signal-runtime` instead of host-local packet handling
- giving Batch 12.2 one fixed event boundary before deeper runtime and adapter
  realization work begins

## Batch 12.2 Outcome

Batch 12.2 materializes the widened generic event model through shared
Signal-owned runtime and adapter surfaces instead of leaving note-expression
and MIDI depth in host-private summaries.

The repo now has:

- explicit `supports_note_expression` capability on `PluginProcessingContract`
  across CLAP, VST3, AU, runtime discovery, and capability-coverage receipts
- a runtime-owned `RuntimePluginEventSnapshot` that tracks last-batch and
  aggregate parameter, note, note-expression, and three-byte MIDI event counts
  plus lease-based continuity
- stable host processing paths that feed existing `EventPacket::summary()`
  results back into runtime-owned observation and supervisor surfaces rather
  than reconstructing generic event truth only in host-local payload summaries
- focused runtime and host proofs that generic event continuity survives the
  bounded transport and recovery paths already owned by `signal-runtime`

## Batch 12.3 Outcome

Batch 12.3 closes the bounded generic-event consumer seam:

- downstream-style `signal-runtime` proofs now consume the widened generic
  event, note-expression, and capability receipts directly through shared
  observation and supervisor surfaces
- both stable host edges now prove that `supervisor_report()` forwards the
  same runtime-owned generic event truth without CLAP, VST3, or AU packet
  reconstruction
- `signal-supervisor-tools` now exposes a machine-readable
  `signal.runtime.generic-event-boundary` descriptor plus the repo-owned
  `effigy acceptance:generic-event-boundary` task
- `g06.013` can now deepen preset-state interchange, portable recall, and
  ARA-capable context work on top of one closed generic event baseline

## Next Task

Continue `g06.013` with Batch 13.1 by freezing plugin preset-state
interchange, portable recall, and ARA-capable context vocabulary before
runtime recall/export depth begins.
