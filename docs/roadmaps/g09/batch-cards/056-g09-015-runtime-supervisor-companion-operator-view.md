# 056 - g09.015 Runtime Supervisor Companion Operator View

Status: complete
Owner: core-product
Updated: 2026-04-11
Parent roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`
Governing contracts: `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`, `docs/contracts/081-operator-visible-interactive-demo-and-low-dependency-ui-contract.md`

## Objective

Promote the runtime supervisor boundary companion from receipt-and-notes only
into a rendered low-dependency operator view that makes interruption and
fault-diagnostic boundary posture visually inspectable.

## Why This Batch Exists

The runtime recovery inspector now has a rendered operator view, but its
machine-readable companion surface in `signal-supervisor-tools` is still
receipt-shaped. That leaves the runtime family partially visible at the
operator layer.

This is the next honest seam because:

- the supervisor companion already captures bounded machine-readable boundary
  descriptor payloads
- the surface is presentation-thin rather than behavior-thin
- a rendered operator view can stay presentation-only over existing proof data
  without widening into a runtime console, dashboard, or host shell

## Scope

- add a rendered companion view for the runtime supervisor boundary companion
- surface interruption and fault-diagnostic boundary posture as visual operator
  cards rather than raw receipt fields alone
- keep descriptor-backed runtime-boundary semantics explicit in the rendered
  surface
- align manifest, operator notes, and receipt evidence to the rendered view

## Out Of Scope

- interactive runtime control
- new supervisor descriptor behavior
- new runtime, host, or device behavior
- replacing the underlying descriptor or receipt surface

## Acceptance Criteria

- `effigy demo:supervisor-runtime-boundary-companion` emits a rendered
  companion view
- the rendered surface makes the interruption and fault-diagnostic boundary
  posture visually inspectable without reading raw JSON first
- the surface stays browser-native and low-dependency

## Validation

- `effigy demo:supervisor-runtime-boundary-companion`
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- rendered runtime supervisor companion view
- receipt, manifest, and operator notes aligned to the rendered view
- batch log with validation actually run

## Stop Conditions

- the uplift would require inventing new supervisor or runtime behavior instead
  of presenting existing proof data
- the current companion output is too thin to support an honest rendered view
  without another planning step

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.015` seam is another crate-family operator-view uplift, deeper live
plugin interaction, or a planning pause.
