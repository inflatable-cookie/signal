# 006 - Surround Bed, Object, And Mix-Policy Expansion

Status: complete
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

- [x] define bed, object, and mix-policy semantics on top of the baseline adapter model
- [x] keep fallback and unsupported-state behavior explicit

### Batch 6.2 - Runtime Expansion

- [x] implement the first bounded richer spatial path
- [x] keep multichannel and routing receipts aligned with the widened model

### Batch 6.3 - Focused Proof

- [x] add focused proofs for richer spatial and mix-policy behavior

## Acceptance Criteria

- [x] Signal has an explicit richer spatial expansion path
- [x] later product workflows can build on runtime-owned spatial truth
- [x] immersive or surround behavior remains inspectable and bounded

## Risks And Mitigations

- Risk: richer spatial work drifts into product UX or room-design scope.
- Mitigation: keep the queue on execution, policy, and receipts only.

## Evidence Requirements

- [x] log each meaningful richer-spatial tranche
- [x] run focused spatial and mix-policy validation
- [ ] record deferred immersive breadth explicitly

## Batch 6.1 Outcome

Batch 6.1 freezes the bounded richer-spatial expansion contract in
`docs/contracts/037-surround-bed-object-and-mix-policy-expansion-contract.md`.

Signal now has one explicit shared vocabulary for:

- surround-bed class
- object role
- mix policy
- render scope
- expanded fallback outcome

That gives Batch 6.2 one fixed runtime-owned target for widening beyond the
current `StereoBalance` baseline without drifting into product-local immersive
console policy, room-design UX, or renderer-private object semantics.

## Batch 6.2 Outcome

Batch 6.2 materializes the first bounded richer-spatial runtime path on top of
that contract.

`signal-runtime` now carries surround-bed, mix-policy, render-scope, and
expanded-fallback meaning directly on the existing spatial execution receipts
instead of leaving richer spatial depth implicit in layout and fallback alone.

In this bounded baseline:

- stereo `StereoBalance` stages surface explicit `StereoBed`, `BedOnly`, and
  `BedRender` meaning
- canonical surround stages surface explicit `CanonicalSurroundBed` plus
  `CollapseToBaselineSpatial` expanded fallback instead of silent non-stereo
  bypass
- object-aware depth stays explicit rather than implied: `object_role` and
  `object_count` are runtime-owned receipts even though the current bounded
  path still realizes zero objects
- execution-topology, plugin-chain, offline-render dependency preview, and
  shared host report JSON now stay aligned to one richer spatial model

Deferred scope is still explicit:

- true object rendering is not implemented yet
- richer fold-down policy and immersive renderer breadth are still deferred
- the public runtime, supervisor, and stable host-edge proof seam is still the
  next batch

## Batch 6.3 Outcome

Batch 6.3 closes the public consumer seam for the widened richer-spatial
receipt family.

Public runtime proofs, both stable host edges, and the machine-readable
`signal.runtime.spatial-boundary` descriptor now all verify the same bounded
surround-bed, mix-policy, render-scope, and expanded-fallback truth.

This closes `g07.006` as a bounded reusable substrate milestone:

- downstream runtime consumers can inspect richer spatial receipts without
  adapter-local renderer reconstruction
- stable host-edge consumers now see the same richer spatial model on
  supervisor export
- the descriptor and repo-owned acceptance lane now point at the richer
  `g07.006` contract instead of the earlier baseline-only spatial contract

Deferred scope stays explicit rather than hidden:

- true object rendering is still not implemented
- immersive renderer breadth and room policy remain later `g07` work
- this closes the bounded richer-spatial seam, not a production surround
  deployment story

## Next Task

Continue `g07.007` with Batch 7.1 by mapping LV2-specific discovery,
lifecycle, and Linux-native capability details onto the existing backend-neutral
plugin contract before runtime-owned LV2 realization widens.
