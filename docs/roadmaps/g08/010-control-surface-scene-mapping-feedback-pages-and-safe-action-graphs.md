# 010 - Control-Surface Scene Mapping, Feedback Pages, And Safe Action Graphs

Status: complete
Owner: core-product
Created: 2026-03-21
Depends on: g08.009
Vision tags: `CONTROL`, `WORKFLOW`, `SAFETY`

## Problem

`g08.009` closes the bounded advanced display, motor, and haptic seam, but
scene mapping, feedback-page meaning, and safe action graph coordination are
still deferred to product-local controller workflow or host-side device
scripts. Without a runtime-owned contract here, richer control-surface
workflow depth will drift back into controller-page assumptions, unsafe device
action glue, or app-local scene ledgers.

## Goals

- [ ] freeze one runtime-owned authority line for scene mapping, feedback pages, and safe action graphs
- [ ] keep richer control-surface workflow composable with the closed controller-expression, control-surface, advanced feedback, and advanced-hardware seams
- [ ] avoid device-private page shells or product-local workflow graphs becoming shared truth

## Non-Goals

- [ ] no product-local controller UI design or end-user page editing workflow
- [ ] no arbitrary executable device scripting or unsafe action dispatch

## Execution Plan

### Batch 10.1 - Scene Mapping And Safe Action Contract

- [x] freeze runtime-owned scene mapping, feedback-page, and safe action graph meaning
- [x] define shared runtime versus device-private authority explicitly

### Batch 10.2 - Runtime Workflow Baseline

- [x] materialize the first runtime-owned scene mapping, feedback-page, and safe action graph receipts
- [x] align stable host-edge export with the same bounded model

### Batch 10.3 - Consumer Proof

- [x] prove the widened control-surface workflow seam through shared runtime,
      supervisor, and stable host-edge surfaces

## Acceptance Criteria

- [x] scene mapping, feedback-page, and safe action graph posture is runtime-owned and inspectable
- [ ] device-private workflow detail stays bounded and typed
- [ ] later control-surface acceptance and workflow work can build on one explicit action authority line

## Risks And Mitigations

- Risk: richer control-surface workflow drifts into controller-page assumptions, unsafe device scripts, or host-local scene orchestration.
- Mitigation: freeze one runtime-owned contract before widening runtime realization.

## Evidence Requirements

- [ ] log each meaningful tranche
- [ ] run focused validation after the runtime baseline lands
- [ ] record the next milestone step explicitly

## Batch 10.1 Outcome

Batch 10.1 freezes the first reusable control-surface workflow contract in
`docs/contracts/061-control-surface-scene-mapping-feedback-pages-and-safe-action-graph-contract.md`.

That contract widens the closed controller-expression, control-surface,
advanced-hardware, and advanced-feedback seams without reopening
controller-page assumptions, host-local scene ledgers, or unsafe device
scripts as the shared authority.

It now makes the authority line explicit:

- `043`, `044`, `045`, and `060` remain the widened controller-expression,
  control-surface, advanced-hardware, and advanced-feedback authorities, so
  richer scene, page, and safe-action semantics must compose with those seams
  instead of replacing them
- Batch 10.2 now has one bounded contract target for runtime-owned scene
  mapping, feedback-page, and safe action graph receipts before consumer proof
  widens in Batch 10.3
- vendor page-bank schemas, unsafe macro payloads, and product-local
  controller workflow remain explicitly deferred until later runtime
  realization

## Next Task

Open `g08.011` with Batch 11.1 by freezing the first runtime-owned preview-
output routing, audition-sink ownership, and low-latency device-policy
contract on top of the closed controller and workflow seams.

## Batch 10.2 Outcome

Batch 10.2 widens the existing advanced-hardware seam instead of opening a
parallel controller-workflow shell.

`signal-runtime` now owns bounded scene-mapping, feedback-page, and safe
action graph posture directly on `RuntimeAdvancedHardwareSnapshot`, including
shared authority and safe-action outcome answers plus aggregate workflow
device counts.

That same runtime-owned receipt family now flows through:

- public runtime re-exports and focused downstream-style proofs
- stable local and server host-edge export without host-local workflow
  reclassification
- existing advanced-hardware JSON and multiline renderers so later supervisor
  proof can widen the current seam instead of inventing a controller-specific
  descriptor

The batch keeps vendor page-bank schemas, unsafe macros, and product-local
controller workflow out of scope while making the bounded Signal-owned
workflow truth inspectable for the first time.

## Batch 10.3 Outcome

Batch 10.3 closes `g08.010` by widening the existing
`signal.runtime.advanced-hardware-boundary` instead of introducing a second
controller-workflow-only acceptance lane.

The shared supervisor boundary now points at the control-surface workflow
contract and explicitly describes:

- runtime-owned scene-mapping, feedback-page, and safe action graph counts on
  `RuntimeAdvancedHardwareSnapshot`
- bounded per-device workflow posture, authority, and safe-action outcome on
  `RuntimeAdvancedHardwareDeviceDescriptor`
- the same focused runtime and stable host-edge proof spine already used for
  the broader advanced-hardware seam

This closes the bounded consumer seam for control-surface workflow without
promoting vendor-private page shells, unsafe scripting, or host-local scene
orchestration into shared Signal policy.
