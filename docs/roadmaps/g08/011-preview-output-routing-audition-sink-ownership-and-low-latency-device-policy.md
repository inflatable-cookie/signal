# 011 - Preview Output Routing, Audition Sink Ownership, And Low-Latency Device Policy

Status: complete
Owner: core-product
Created: 2026-03-21
Depends on: g08.010
Vision tags: `PREVIEW`, `ROUTING`, `DEVICE`

## Problem

`g08.010` closes the bounded control-surface workflow seam, but preview output
routing, audition sink ownership, and low-latency preview-device policy are
still at risk of drifting into browser-local routing assumptions, host-local
device picks, or app-specific audition shells.

Without a runtime-owned contract here, later preview workflow work will
either reopen device policy outside Signal-owned receipts or tie preview
delivery to product-local output assumptions that cannot survive supervisor or
stable host-edge export.

## Goals

- [ ] freeze one runtime-owned authority line for preview output routing,
      audition sink ownership, and low-latency device policy
- [ ] keep preview routing composable with the closed external I/O,
      control-surface, advanced-hardware, and workflow seams
- [ ] avoid browser-local preview routing or host-local device-pick policy
      becoming shared truth

## Non-Goals

- [ ] no product-local browser UX, editor-specific audition workflows, or end-user device picker design
- [ ] no backend-private output mixer scripting or app-local preview buses as the shared contract

## Execution Plan

### Batch 11.1 - Preview Routing Contract

- [x] freeze runtime-owned preview output routing, audition sink, and low-latency device-policy meaning
- [x] define shared runtime versus host-local authority explicitly

### Batch 11.2 - Runtime Preview Routing Baseline

- [x] materialize the first runtime-owned preview output routing, audition sink, and low-latency device-policy receipts
- [x] align stable host-edge export with the same bounded model

### Batch 11.3 - Consumer Proof

- [x] prove the widened preview-routing seam through shared runtime, supervisor, and stable host-edge surfaces

## Acceptance Criteria

- [ ] preview output routing, audition sink ownership, and low-latency device policy are runtime-owned and inspectable
- [ ] host-local or browser-local preview detail stays bounded and typed
- [ ] later preview and audition workflow work can build on one explicit device-policy authority line

## Risks And Mitigations

- Risk: preview routing drifts into browser-local buses, host-local device picks, or app-specific audition shells.
- Mitigation: freeze one runtime-owned contract before widening runtime realization.

## Evidence Requirements

- [ ] log each meaningful tranche
- [ ] run focused validation after the runtime baseline lands
- [ ] record the next milestone step explicitly

## Next Task

Open `g08.012` with Batch 12.1 by freezing the first runtime-owned preview-
browser queue, media audition orchestration, and transform-scheduling contract
on top of the closed preview-device seam.

## Batch 11.1 Outcome

Batch 11.1 freezes the first reusable preview-device authority line in
`docs/contracts/062-preview-output-routing-audition-sink-and-low-latency-device-policy-contract.md`.

That contract deliberately sits on top of the closed preview-transform,
external-I/O, controller, and advanced-hardware seams instead of inventing a
second preview player, device-picker, or host-private audition route model.

It now makes the ownership split explicit:

- the closed preview-transform contract remains the authority for low-latency
  preview service readiness, degraded state, fallback, and artifact alignment
- the closed external-I/O contract remains the authority for monitor-path and
  device-facing route meaning
- `g08.011` now owns the shared meaning for preview-output routing,
  audition-sink ownership, and low-latency device policy where preview
  delivery meets device-facing routing
- later browser or product-local preview workflow stays explicitly deferred

Batch 11.2 can therefore focus on typed runtime receipts instead of reopening
whether preview-device routing belongs to runtime, hosts, or browser code.

## Batch 11.2 Outcome

Batch 11.2 widens the existing preview-transform seam instead of opening a
parallel preview-device report family.

`signal-runtime` now carries a bounded `preview_device_policy` summary on
`RuntimePreviewTransformServiceSnapshot`, including runtime-owned routing
posture, audition-sink class, sink authority, low-latency device-policy
class, and device-policy outcome.

That same receipt family now flows through:

- public runtime re-exports and focused downstream-style preview-transform
  proofs
- stable local and server host-edge export without host-local preview-route or
  device-pick reconstruction
- the existing preview-transform JSON and summary surfaces so later supervisor
  proof can widen the current seam instead of inventing a preview-only
  boundary

This batch keeps richer live device selection, browser routing UX, and
dedicated preview-output negotiation explicitly deferred while making bounded
preview-device policy inspectable for the first time.

## Batch 11.3 Outcome

Batch 11.3 closes `g08.011` by widening the existing
`signal.runtime.preview-transform-boundary` instead of creating a second
preview-device-only acceptance lane.

The shared supervisor boundary now points at the preview-device contract and
explicitly describes:

- runtime-owned preview-device policy on `RuntimePreviewTransformServiceSnapshot`
- the same preview-device truth on render-preview and offline-preview surfaces
- the focused public runtime and stable host-edge proof spine already used for
  the bounded preview-transform seam

This closes the bounded consumer seam for preview-output routing,
audition-sink ownership, and low-latency device policy without promoting a
host-local route shell, browser-local preview bus, or device-picker model into
shared Signal policy.
