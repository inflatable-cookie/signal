# 054 - g09.015 DSP Processing Operator View

Status: complete
Owner: core-product
Updated: 2026-04-10
Parent roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`
Governing contracts: `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`, `docs/contracts/081-operator-visible-interactive-demo-and-low-dependency-ui-contract.md`

## Objective

Promote the DSP processing lab from receipt-and-notes only into a rendered
low-dependency operator view that makes stretch, marker-analysis, and
transform-artifact posture visually inspectable.

## Why This Batch Exists

Plugin, analysis, and graph now have genuinely visual operator surfaces. The
DSP processing lab is still live, but it remains too receipt-shaped for direct
operator verification.

This is the next honest seam because:

- the DSP lab already captures bounded machine-readable descriptor payloads
  plus focused acceptance-lane results
- the surface is presentation-thin rather than behavior-thin
- a rendered operator view can stay presentation-only over existing proof data
  without widening into an editor, waveform browser, or product shell

## Scope

- add a rendered companion view for the DSP processing lab
- surface stretch, marker-analysis, and transform-artifact posture as visual
  operator cards rather than raw receipt fields alone
- keep acceptance-lane posture and deferred scope explicit in the rendered
  surface
- align manifest, operator notes, and receipt evidence to the rendered view

## Out Of Scope

- interactive sample editing
- waveform scrubbing, asset browsing, or tutorial UI
- new DSP, runtime, or analysis behavior
- replacing the underlying descriptor or acceptance proof surfaces

## Acceptance Criteria

- `effigy demo:dsp-processing-lab` emits a rendered companion view
- the rendered surface makes the three DSP boundary families visually
  inspectable without reading raw JSON first
- the surface stays browser-native and low-dependency

## Validation

- `effigy demo:dsp-processing-lab`
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- rendered DSP processing companion view
- receipt, manifest, and operator notes aligned to the rendered view
- batch log with validation actually run

## Stop Conditions

- the uplift would require inventing new DSP/runtime behavior instead of
  presenting existing proof data
- the current DSP lab data is too thin to support an honest rendered view
  without another planning step

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.015` seam is another crate-family operator-view uplift, deeper
live plugin interaction, or a planning pause.
