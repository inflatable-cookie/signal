# 005 - Complex Plugin Pin-Matrix And Dynamic Bus Negotiation Depth

Status: complete
Owner: core-product
Created: 2026-03-19
Depends on: g08.004
Vision tags: `LINUX`, `PLUGIN`, `GRAPH`

## Problem

`g08.004` closes the bounded LV2 extension seam, but complex plugin pin-matrix
and dynamic bus-negotiation truth still sits below the shared runtime surface.
Without a runtime-owned contract here, richer plugin I/O depth will drift back
into adapter-private port graphs, host-local bus rules, or format-specific
negotiation policy.

## Goals

- [x] freeze one runtime-owned authority line for complex plugin pin-matrix and dynamic bus negotiation
- [x] expose bounded pin-matrix and dynamic bus posture through shared runtime and stable host edges
- [x] keep adapter-private bus negotiation detail additive rather than authoritative

## Non-Goals

- [ ] no product-local mixer, patchbay, or bus inspector UX
- [ ] no full format-specific port schema dump or host-private wiring policy

## Execution Plan

### Batch 5.1 - Pin-Matrix And Bus-Negotiation Contract

- [x] freeze runtime-owned complex plugin pin-matrix and dynamic bus-negotiation meaning
- [x] define shared runtime versus adapter-private authority explicitly

### Batch 5.2 - Runtime Pin-Matrix Baseline

- [x] materialize the first runtime-owned pin-matrix and dynamic bus-negotiation receipts
- [x] align stable host-edge export with the same bounded model

### Batch 5.3 - Consumer Proof

- [x] prove the widened pin-matrix and dynamic bus-negotiation seam through shared runtime,
      supervisor, and stable host-edge surfaces

## Acceptance Criteria

- [x] complex plugin pin-matrix and dynamic bus-negotiation posture is runtime-owned and inspectable
- [x] adapter-private bus and port detail stays bounded and typed
- [x] later Linux plugin and workflow work can build on one explicit complex-I/O authority line

## Risks And Mitigations

- Risk: complex plugin I/O depth drifts into adapter-private bus tables or host-local wiring policy.
- Mitigation: freeze one runtime-owned contract before widening runtime realization.

## Evidence Requirements

- [x] log each meaningful tranche
- [x] run focused validation after the runtime baseline lands
- [x] record the next milestone step explicitly

## Batch 5.1 Outcome

Batch 5.1 freezes the bounded pin-matrix and dynamic bus-negotiation seam in
`docs/contracts/056-complex-plugin-pin-matrix-and-dynamic-bus-negotiation-contract.md`.

That contract layers pin-group identity, pin-matrix posture, dynamic
bus-negotiation posture, and negotiation fallback outcome on top of the closed
complex plugin-I/O, multi-bus, Linux parity, and LV2 extension seams instead
of inventing a competing host-local bus-policy shell.

It now makes the authority line explicit:

- `035` remains the bounded complex plugin-I/O baseline instead of being
  reopened as a generic pin-matrix or mixer policy surface
- `034` remains the authority for bus-role, auxiliary-path, and fallback
  meaning, so plugin negotiation still composes through the shared multi-bus
  substrate
- `039` and `055` remain the authority for Linux parity and LV2 extension
  truth, so guarded format depth cannot be reclassified as generic
  pin-matrix truth
- Batch 5.2 now has one bounded contract target for runtime-owned pin-matrix
  and dynamic bus-negotiation receipts before public proof widens in Batch 5.3

## Batch 5.2 Outcome

Batch 5.2 turns the frozen pin-matrix and dynamic bus-negotiation contract into
a reusable runtime-owned receipt family.

- `signal-runtime` now exports typed pin-group identity, pin-matrix posture,
  dynamic bus-negotiation posture, and fallback outcome through one
  `RuntimePluginPinMatrixSnapshot` surface instead of leaving that meaning in
  adapter-private port graphs or host-local bus policy
- the widened receipt composes from runtime-owned complex-I/O discovery,
  sandbox lifecycle, and plugin-chain stage truth, so declared, negotiated,
  guarded, and unavailable outcomes stay visible without inventing a second
  routing or plugin lifecycle model
- stable host-edge surfaces now export the same pin-matrix snapshot instead of
  reconstructing complex bus activation posture from host-local plugin detail
- Batch 5.3 can now close the seam with a bounded consumer proof and, if it is
  warranted, a repo-owned acceptance descriptor on top of the same receipt
  family

## Batch 5.3 Outcome

Batch 5.3 closes `g08.005` by widening the existing complex plugin-I/O
consumer boundary to the new runtime-owned pin-matrix and dynamic
bus-negotiation seam instead of creating a second overlapping routing
descriptor.

- the repo-owned `complex-io-boundary` descriptor now points at the
  pin-matrix and dynamic bus-negotiation contract instead of the older
  baseline-only complex-I/O contract
- the machine-readable supervisor boundary now describes both the prior
  complex-I/O receipts and the new `plugin_pin_matrix_snapshot` surface as one
  bounded shared proof seam
- the existing acceptance lane remains reusable, but now proves the widened
  runtime, supervisor, and stable host-edge routing boundary without
  plugin-format-specific negotiation policy
- `g08.005` is complete, and later immersive or renderer work can build on one
  explicit plugin-routing authority line instead of reopening pin-matrix or
  bus-negotiation policy in host code

## Next Task

Open `g08.006` with Batch 6.1 by freezing the first runtime-owned immersive
object rendering and room-policy contract on top of the closed plugin-routing,
LV2 extension, Linux parity, and live backend seams.
