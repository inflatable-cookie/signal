# 058 - g09.015 Local Server Host Comparison Operator View

Status: complete
Owner: core-product
Updated: 2026-04-11
Parent roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`
Governing contracts: `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`, `docs/contracts/081-operator-visible-interactive-demo-and-low-dependency-ui-contract.md`

## Objective

Promote the local-versus-server host comparison demo from receipt-and-notes
only into a rendered low-dependency operator view that makes shared and
differing host posture visually inspectable.

## Why This Batch Exists

The plugin, analysis, graph, DSP, runtime, and hardware demo families now have
rendered operator companions, but the local/server host comparison surface is
still receipt-shaped.

This is the next honest seam because:

- the host comparison demo already captures bounded local and server bootstrap
  truth from the existing binaries
- the remaining gap is presentation, not new host or plugin behavior
- a rendered operator view can stay presentation-only over existing proof data
  without widening into a host UI shell or interactive control surface

## Scope

- add a rendered companion view for the local-versus-server host comparison
  demo
- surface shared readiness and the key local-versus-server differences as
  visual operator cards rather than raw receipt fields alone
- keep the comparison bounded to the existing host summary lines
- align manifest, operator notes, and receipt evidence to the rendered view

## Out Of Scope

- interactive host control
- new host runtime behavior
- plugin browsing redesign
- replacing the underlying receipt surface

## Acceptance Criteria

- `effigy demo:local-server-host-comparison` emits a rendered companion view
- the rendered surface makes local-versus-server host posture visually
  inspectable without reading raw JSON first
- the surface stays browser-native and low-dependency

## Validation

- `effigy demo:local-server-host-comparison`
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- rendered local/server host comparison companion view
- receipt, manifest, and operator notes aligned to the rendered view
- batch log with validation actually run

## Stop Conditions

- the uplift would require inventing new host behavior instead of presenting
  existing proof data
- the current comparison receipt is too thin to support an honest rendered view
  without another planning step

## Next Task

Re-enter planning for the active strict `g09` lane and leave a bounded runway
for the next `g09.015` operator-visible batches, including the next planning
checkpoint.
