# 055 - g09.015 Runtime Recovery Operator View

Status: complete
Owner: core-product
Updated: 2026-04-11
Parent roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`
Governing contracts: `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`, `docs/contracts/081-operator-visible-interactive-demo-and-low-dependency-ui-contract.md`

## Objective

Promote the runtime recovery inspector from receipt-and-notes only into a
rendered low-dependency operator view that makes watchdog, fault, safe-mode,
and degraded external-surface posture visually inspectable.

## Why This Batch Exists

Plugin, analysis, graph, and DSP now have genuinely visual operator surfaces.
The runtime recovery inspector is still live, but it remains too receipt-shaped
for direct operator verification.

This is the next honest seam because:

- the runtime recovery inspector already captures bounded structured report
  output from the existing supervisor report example
- the surface is presentation-thin rather than behavior-thin
- a rendered operator view can stay presentation-only over existing proof data
  without widening into a runtime console, dashboard, or product shell

## Scope

- add a rendered companion view for the runtime recovery inspector
- surface watchdog, plugin-fault, safe-mode, and degraded external/backend
  posture as visual operator cards rather than raw receipt fields alone
- keep the bounded runtime-report semantics explicit in the rendered surface
- align manifest, operator notes, and receipt evidence to the rendered view

## Out Of Scope

- interactive runtime control
- live watchdog injection, restart controls, or product-shell dashboards
- new runtime, host, or device behavior
- replacing the underlying example or receipt surface

## Acceptance Criteria

- `effigy demo:runtime-recovery-inspector` emits a rendered companion view
- the rendered surface makes the runtime recovery posture visually inspectable
  without reading raw JSON first
- the surface stays browser-native and low-dependency

## Validation

- `effigy demo:runtime-recovery-inspector`
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- rendered runtime recovery companion view
- receipt, manifest, and operator notes aligned to the rendered view
- batch log with validation actually run

## Stop Conditions

- the uplift would require inventing new runtime behavior instead of presenting
  existing proof data
- the current runtime recovery output is too thin to support an honest rendered
  view without another planning step

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.015` seam is another crate-family operator-view uplift, deeper
live plugin interaction, or a planning pause.
