# 006 - Immersive Object Rendering And Room-Policy Substrate

Status: complete
Owner: core-product
Created: 2026-03-19
Depends on: g08.005
Vision tags: `IMMERSIVE`, `ROUTING`, `GRAPH`

## Problem

`g08.005` closes the bounded plugin-routing seam, but immersive object
rendering and room-policy truth still sits below the shared runtime surface.
Without a runtime-owned contract here, richer immersive execution will drift
back into renderer-private policy, host-local room rules, or product-local
monitoring interpretation.

## Goals

- [ ] freeze one runtime-owned authority line for immersive object rendering and room policy
- [ ] keep immersive routing and room-policy meaning composable with the closed plugin-routing substrate
- [ ] avoid renderer-private or host-local room policy becoming shared truth

## Non-Goals

- [ ] no product-local immersive mixer, panner, or room editor UX
- [ ] no final renderer-capability negotiation or export packaging in this milestone

## Execution Plan

### Batch 6.1 - Immersive Object And Room-Policy Contract

- [x] freeze runtime-owned immersive object rendering and room-policy meaning
- [x] define shared runtime versus renderer-private authority explicitly

### Batch 6.2 - Runtime Immersive Baseline

- [x] materialize the first runtime-owned immersive object and room-policy receipts
- [x] align stable host-edge export with the same bounded model

### Batch 6.3 - Consumer Proof

- [x] prove the widened immersive seam through shared runtime,
      supervisor, and stable host-edge surfaces

## Acceptance Criteria

- [x] immersive object rendering and room-policy posture is runtime-owned and inspectable
- [x] renderer-private and host-local room detail stays bounded and typed
- [x] later immersive export and monitoring work can build on one explicit room-policy authority line

## Risks And Mitigations

- Risk: immersive execution depth drifts into renderer-private or host-local room policy.
- Mitigation: freeze one runtime-owned contract before widening runtime realization.

## Evidence Requirements

- [x] log each meaningful tranche
- [x] run focused validation after the runtime baseline lands
- [x] record the next milestone step explicitly

## Batch 6.1 Outcome

Batch 6.1 freezes the first reusable immersive object and room-policy contract
in
`docs/contracts/057-immersive-object-rendering-and-room-policy-substrate-contract.md`.

That contract layers immersive object-rendering posture, room-policy class,
room-policy authority, and immersive room outcome on top of the closed
baseline spatial, richer-spatial, plugin-routing, LV2, and Linux live-ownership
seams instead of letting immersive meaning drift into renderer-private room
rules or host-local deployment heuristics.

It now makes the authority line explicit:

- `036` remains the baseline spatial adapter contract instead of being reopened
  as a generic immersive renderer policy surface
- `037` remains the authority for bed, object, mix-policy, and expanded
  fallback meaning, so immersive room policy composes with that richer-spatial
  substrate instead of replacing it
- `056`, `055`, `052`, and `054` remain the plugin-routing, LV2, and Linux
  live-ownership authorities, so immersive room policy cannot silently invent
  a second backend or plugin-routing truth model
- Batch 6.2 now has one bounded contract target for runtime-owned immersive
  object and room-policy receipts before consumer proof widens in Batch 6.3

## Batch 6.2 Outcome

Batch 6.2 turns the frozen immersive object and room-policy contract into a
reusable runtime-owned receipt layer.

- `signal-runtime` now carries one bounded `immersive_room_policy` summary on
  top of the existing richer-spatial execution receipt instead of opening a
  second immersive report family
- execution topology, plugin-chain stages, and offline-render dependency
  preview now all expose the same immersive room-policy truth, including
  aggregate counts for room-policy-bearing and fallback room-policy paths
- stable host-edge supervisor export now forwards the same runtime-owned
  immersive room-policy receipt family instead of leaving room-policy meaning
  to host-local interpretation
- the current baseline stays explicit rather than aspirational: canonical
  surround fallback paths surface `FallbackRoom` plus `BypassRoomPolicy`, while
  stereo-only paths stay outside the immersive room-policy seam until later
  renderer-backed work exists

Batch 6.3 can now close the consumer seam by widening the shared runtime,
supervisor, and stable host-edge proof boundary to this new immersive receipt
layer.

## Batch 6.3 Outcome

Batch 6.3 closes `g08.006` by widening the existing `spatial-boundary`
consumer seam to the new immersive room-policy substrate instead of creating a
second overlapping acceptance descriptor.

- the repo-owned `spatial-boundary` descriptor now points at the immersive
  room-policy contract instead of the earlier richer-spatial-only contract
- the machine-readable supervisor boundary now describes immersive room-policy
  topology, plugin-chain, and offline-render preview anchors as one bounded
  shared proof seam
- the existing acceptance lane remains reusable, but now proves the widened
  runtime, supervisor, and stable host-edge immersive boundary without a
  renderer-private room-policy shell
- `g08.006` is complete, and later speaker deployment, fold-down, and
  monitoring-scene work can build on one explicit immersive room-policy
  authority line instead of reopening ownership in host or renderer code

## Next Task

Continue `g08.007` with Batch 7.1 by freezing the first runtime-owned speaker
deployment, fold-down, and monitoring-scene contract on top of the closed
immersive room-policy seam.
