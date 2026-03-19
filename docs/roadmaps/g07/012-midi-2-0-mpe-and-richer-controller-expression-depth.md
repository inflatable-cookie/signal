# 012 - MIDI 2.0, MPE, And Richer Controller-Expression Depth

Status: complete
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

- [x] define the widened controller-expression and MIDI depth contract
- [x] align plugin and device receipts with the new event meaning

### Batch 12.2 - Runtime And Adapter Depth

- [x] implement the first credible widened event path
- [x] keep runtime, plugin, and hardware surfaces on one event vocabulary

### Batch 12.3 - Focused Proof

- [x] add focused proofs for widened controller-expression behavior

## Acceptance Criteria

- [x] Signal has an explicit richer controller-expression surface
- [x] later control-surface and device work can reuse the same event vocabulary
- [x] hosts do not need adapter-specific event shims to consume the new depth

## Risks And Mitigations

- Risk: event breadth becomes speculative and detached from runtime use.
- Mitigation: require direct mapping to runtime, plugin, and hardware receipts.

## Evidence Requirements

- [x] log each meaningful controller-expression tranche
- [x] run focused widened-event validation
- [x] record deferred expressive-event breadth explicitly

## Batch 12.1 Outcome

Batch 12.1 freezes the widened controller-expression contract in
`docs/contracts/043-midi-2-0-mpe-and-richer-controller-expression-contract.md`.

Signal now has one shared contract for:

- runtime-owned MIDI 2.0-adjacent, MPE-aware, and richer controller-
  expression meaning instead of adapter-private packet models becoming the
  consumer boundary
- bounded widened expression families, capability posture, guarded widening,
  and note-scoped versus channel-scoped meaning that later runtime work must
  target
- explicit reuse of the closed generic event contract and the just-closed
  external MIDI endpoint boundary instead of inventing a second device or
  event shell

That gives Batch 12.2 one fixed runtime and adapter target for widened
expressive-event receipt work without drifting into product-local controller
UX or backend-private MIDI 2.0 transport semantics.

## Batch 12.2 Outcome

Batch 12.2 materializes the first runtime-owned widened controller-expression
receipt family across plugin summary, runtime observation, and bounded external
MIDI capability surfaces.

Signal now carries one shared widened baseline for:

- richer note-expression family counts split into pressure, timbre, and tuning
  instead of treating all widened note expression as one opaque total
- runtime-owned MPE and MIDI 2.0 posture on `RuntimePluginEventSnapshot`,
  derived from shared event evidence instead of adapter-private packet shells
- external MIDI capability baselines that can explicitly say note-pressure,
  note-timbre, note-tuning, and MPE are unsupported or guarded rather than
  leaving richer controller depth implicit

That gives Batch 12.3 a concrete shared receipt family to prove through public
runtime, supervisor, and stable host-edge surfaces without reopening packet or
device reconstruction in consumer code.

## Batch 12.3 Outcome

Batch 12.3 closes the widened controller-expression consumer seam through
public runtime, both stable host edges, `signal-supervisor-tools`, and a
repo-owned Effigy acceptance lane.

Signal now proves that:

- widened note-expression family totals and runtime-owned `MPE` / `MIDI 2.0`
  posture remain consumable through shared runtime and supervisor reports
  instead of adapter-private packet reconstruction
- bounded external MIDI controller-expression capability posture remains
  runtime-owned when it appears on the shared device boundary
- both stable host edges forward the same widened controller-expression truth
  instead of host-local capability or packet heuristics

That closes `g07.012` as a bounded reusable controller-expression milestone and
lets `g07.013` start from a stable widened device and event substrate instead
of reopening event ownership.

## Next Task

Continue `g07.013` with Batch 13.1 by freezing the runtime-owned
control-surface transport, mapping, feedback, and capability contract on top
of the now-closed external MIDI endpoint and widened controller-expression
boundaries.
