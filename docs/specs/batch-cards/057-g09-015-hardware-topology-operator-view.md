# 057 - g09.015 Hardware Topology Operator View

Status: complete
Owner: core-product
Updated: 2026-04-11
Parent roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`
Governing contracts: `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`, `docs/contracts/081-operator-visible-interactive-demo-and-low-dependency-ui-contract.md`

## Objective

Promote the hardware topology diagnostics demo from receipt-and-notes only into
a rendered low-dependency operator view that makes native-versus-simulated
hardware posture visually inspectable.

## Why This Batch Exists

The plugin, analysis, graph, DSP, and runtime families now have rendered
operator companions, but the hardware topology diagnostics surface is still
receipt-shaped.

This is the next honest seam because:

- the hardware diagnostics demo already captures bounded local and server host
  summary truth
- the remaining gap is presentation, not new backend or device behavior
- a rendered operator view can stay presentation-only over existing proof data
  without widening into a hardware control shell or device inspector product UI

## Scope

- add a rendered companion view for the hardware topology diagnostics demo
- surface native CoreAudio posture and simulated Linux backend posture as
  visual operator cards rather than raw receipt fields alone
- keep native-versus-simulated differences explicit in the rendered surface
- align manifest, operator notes, and receipt evidence to the rendered view

## Out Of Scope

- interactive device control
- new hardware, backend, or host behavior
- native Linux device ownership proof
- replacing the underlying receipt surface

## Acceptance Criteria

- `effigy demo:hardware-topology-diagnostics` emits a rendered companion view
- the rendered surface makes native and simulated hardware posture visually
  inspectable without reading raw JSON first
- the surface stays browser-native and low-dependency

## Validation

- `effigy demo:hardware-topology-diagnostics`
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- rendered hardware topology companion view
- receipt, manifest, and operator notes aligned to the rendered view
- batch log with validation actually run

## Stop Conditions

- the uplift would require inventing new hardware or backend behavior instead
  of presenting existing proof data
- the current hardware receipt is too thin to support an honest rendered view
  without another planning step

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.015` seam is another crate-family operator-view uplift, deeper live
plugin interaction, or a planning pause.
