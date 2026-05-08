# 059 - g09.015 Plugin Sandbox Lifecycle Operator View

Status: complete
Owner: core-product
Updated: 2026-04-11
Parent roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`
Governing contracts: `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`, `docs/contracts/081-operator-visible-interactive-demo-and-low-dependency-ui-contract.md`

## Objective

Promote the plugin sandbox lifecycle demo from receipt-and-notes only into a
rendered low-dependency operator view that makes broker lifecycle and timeout
recovery posture visually inspectable.

## Why This Batch Exists

The operator-visible demo lane has already given rendered companions to the
plugin browser, analysis, graph, DSP, runtime, hardware, and host comparison
families. The most direct remaining receipt-only surface with bounded truth is
the sandbox lifecycle demo.

This is the next honest seam because:

- the sandbox lifecycle demo already captures bounded broker ready, attach,
  status, healthy run, timeout run, teardown, and shutdown truth
- the gap is presentation, not new sandbox or host behavior
- a rendered operator view can stay presentation-only over existing proof data
  without widening into a broker console or host shell

## Scope

- add a rendered companion view for the sandbox lifecycle demo
- surface broker ready, attach/status/teardown, healthy run, timeout run, and
  shutdown posture as visual operator cards rather than raw receipt fields
- keep timeout and cleanup posture explicit in the rendered surface
- align manifest, operator notes, and receipt evidence to the rendered view

## Out Of Scope

- interactive broker control
- new sandbox, runtime, or host behavior
- plugin browsing redesign
- replacing the underlying receipt surface

## Acceptance Criteria

- `effigy demo:sandbox-lifecycle` emits a rendered companion view
- the rendered surface makes broker lifecycle and timeout recovery posture
  visually inspectable without reading raw JSON first
- the surface stays browser-native and low-dependency

## Validation

- `effigy demo:sandbox-lifecycle`
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- rendered sandbox lifecycle companion view
- receipt, manifest, and operator notes aligned to the rendered view
- batch log with validation actually run

## Stop Conditions

- the uplift would require inventing new sandbox or broker behavior instead of
  presenting existing proof data
- the current lifecycle receipt is too thin to support an honest rendered view
  without another planning step

## Next Task

Continue the active strict `g09` lane from
`docs/roadmaps/g09/batch-cards/060-g09-015-platform-boundary-operator-views.md`.
