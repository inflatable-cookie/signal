# 009 - Advanced Control-Surface Display, Motor, And Haptic Transport

Status: complete
Owner: core-product
Created: 2026-03-21
Depends on: g08.008
Vision tags: `CONTROL`, `DEVICE`, `FEEDBACK`

## Problem

`g08.008` closes the bounded immersive renderer and export seam, but richer
control-surface feedback is still deferred to backend-private device glue or
product-local controller UX. Without a runtime-owned contract here, display
payloads, motor state, and haptic transport meaning will drift back into
device-specific scripts, controller-page assumptions, or host-local feedback
bridges.

## Goals

- [ ] freeze one runtime-owned authority line for advanced display, motor, and haptic transport meaning
- [ ] keep richer control-surface feedback composable with the closed controller-expression, control-surface, and advanced-hardware seams
- [ ] avoid device-private protocol payloads or product-local feedback shells becoming shared truth

## Non-Goals

- [ ] no product-local controller page design or workflow choreography
- [ ] no vendor-exclusive device editor UX or preset librarian work

## Execution Plan

### Batch 9.1 - Display, Motor, And Haptic Contract

- [x] freeze runtime-owned display, motor, and haptic transport meaning
- [x] define shared runtime versus device-private authority explicitly

### Batch 9.2 - Runtime Feedback Baseline

- [x] materialize the first runtime-owned display, motor, and haptic transport receipts
- [x] align stable host-edge export with the same bounded model

### Batch 9.3 - Consumer Proof

- [x] prove the widened advanced control-surface feedback seam through shared runtime,
      supervisor, and stable host-edge surfaces

## Acceptance Criteria

- [x] advanced display, motor, and haptic transport posture is runtime-owned and inspectable
- [x] device-private feedback detail stays bounded and typed
- [x] later control-surface workflow and acceptance work can build on one explicit feedback authority line

## Risks And Mitigations

- Risk: richer control-surface feedback drifts into vendor SDK detail, host-local protocol shells, or product-local controller UX.
- Mitigation: freeze one runtime-owned contract before widening runtime realization.

## Evidence Requirements

- [ ] log each meaningful tranche
- [ ] run focused validation after the runtime baseline lands
- [ ] record the next milestone step explicitly

## Batch 9.1 Outcome

Batch 9.1 freezes the first reusable advanced control-surface display, motor,
and haptic transport contract in
`docs/contracts/060-advanced-control-surface-display-motor-and-haptic-transport-contract.md`.

That contract widens the closed controller-expression, control-surface, and
advanced-hardware seams without reopening device-private feedback payloads,
host-local feedback bridges, or product-local controller UX as the shared
authority.

It now makes the authority line explicit:

- `043`, `044`, and `045` remain the widened controller-expression,
  control-surface feedback, and advanced-hardware authorities, so richer
  display, motor, and haptic semantics must compose with those seams instead
  of replacing them
- Batch 9.2 now has one bounded contract target for runtime-owned display,
  motor, and haptic receipts before consumer proof widens in Batch 9.3
- vendor page schemas, servo detail, haptic waveform payloads, and
  product-local controller workflow remain explicitly deferred until later
  runtime realization

## Batch 9.2 Outcome

Batch 9.2 lands the first runtime-owned advanced control-surface display,
motor, and haptic receipt seam inside `signal-runtime` instead of leaving that
meaning at the contract layer only.

What now exists on the shared runtime surface:

- `RuntimeAdvancedHardwareSnapshot` now carries typed display posture, display
  content class, motor posture, haptic posture, feedback authority, and
  feedback outcome per device on the existing advanced-hardware seam
- the same snapshot now exposes aggregate display, motor, and haptic transport
  device counts so consumers do not need to reconstruct richer feedback depth
  from action-class flags alone
- the focused public runtime and stable host-edge proofs now assert the same
  bounded display, motor, and haptic answers, so consumers do not need
  vendor-private payload schemas or host-local feedback bridges

This keeps Batch 9.2 meaningful but bounded:

- the shared runtime seam now exposes richer advanced-feedback posture for the
  current guarded display baseline
- stable host-edge export is aligned to that same bounded model
- consumer-facing supervisor proof still belongs to Batch 9.3

## Batch 9.3 Outcome

Batch 9.3 closes the widened advanced control-feedback seam through the
existing shared `signal.runtime.advanced-hardware-boundary` consumer surface
instead of opening a separate display-only or haptics-only acceptance lane.

What now exists on the shared consumer boundary:

- `signal-supervisor-tools` points the advanced-hardware boundary at contract
  `060` and explicitly describes display, motor, and haptic transport counts
  plus device-level posture and bounded feedback outcome anchors
- the machine-readable supervisor descriptor now carries the widened advanced
  feedback seam on the same runtime-owned snapshot family that already covered
  scripting-safe policy and guarded feedback posture
- the existing `effigy acceptance:advanced-hardware-boundary` lane now closes
  the bounded display, motor, and haptic consumer seam through public runtime
  proof, stable local host-edge proof, stable server host-edge proof, and the
  shared supervisor descriptor

This keeps the closure meaningful but still bounded:

- the closed seam now proves one explicit runtime-owned advanced control-
  feedback authority line for the current guarded display baseline
- consumers do not need vendor-private payload schemas or host-local feedback
  bridges to inspect advanced control-feedback posture
- page-aware display depth, real motor transport, real haptic transport, and
  fuller control-surface workflow remain later `g08` work

## Completion

`g08.009` is complete. The bounded advanced control-surface display, motor,
and haptic transport seam is now frozen, runtime-owned, proved through the
shared consumer boundary, and ready for later control-surface workflow and
acceptance milestones to build on.

## Next Task

Continue `g08.010` with Batch 10.1 by freezing the first runtime-owned
control-surface scene mapping, feedback pages, and safe action graph contract
on top of the closed controller-expression, control-surface, advanced
feedback, and advanced-hardware seams.
