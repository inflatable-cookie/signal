# 050 - g09.015 Analysis Feature Inspector Operator View

Status: complete
Owner: core-product
Updated: 2026-04-10
Parent roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`
Governing contracts: `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`, `docs/contracts/081-operator-visible-interactive-demo-and-low-dependency-ui-contract.md`

## Objective

Turn the existing offline analysis feature inspector into a visually legible
operator surface so rhythm, tonal, loudness, character, and semantic posture
can be verified at a glance instead of through receipt JSON alone.

## Why This Batch Exists

The plugin browser is now operator-facing enough to pause. The next honest
interactive-demo seam is not deeper plugin work, it is lifting another
crate-family proof surface out of receipt-heavy posture.

`signal.demo.analysis.feature-inspector` is the cleanest next target because:

- it already runs through bounded offline examples
- it does not depend on system plugin state, device ownership, or browser-host
  launch containment
- its output is already structured enough to support a lightweight HTML or
  browser-native operator view without introducing a product shell

## Scope

- add one low-dependency visual operator surface for the analysis inspector
- surface rhythm, tonal, loudness, and character/semantic outputs as readable
  cards or sections rather than receipt-only JSON
- keep the underlying receipt and offline example commands as the source of
  truth
- keep the surface synthetic-input and offline-focused

## Out Of Scope

- asset-library browsing
- user-audio upload workflows
- recommendation or tagging UX
- plugin-host or device interaction

## Acceptance Criteria

- an operator can visually inspect the analysis demo without reading the raw
  receipt first
- rhythm, tonal, loudness, and character/semantic posture are each visible in
  the rendered surface
- the surface stays low-dependency and browser-native, not a new product UI

## Validation

- `effigy demo:analysis-feature-inspector`
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- updated analysis feature inspector demo surface and operator notes
- generated visual artifact or rendered companion view
- batch log with validation actually run

## Stop Conditions

- the visual uplift would require new runtime/plugin/device behavior rather
  than presentation of existing bounded outputs
- the receipt/output shape proves too unstable for one meaningful rendered
  operator surface

## Outcome

- the analysis feature inspector now emits a rendered companion view alongside
  the receipt
- rhythm, tonal, loudness, and character-semantic posture are visually
  inspectable without reading raw JSON first
- the uplift stayed inside presentation of existing bounded outputs; no new
  analysis/runtime behavior was required
- there is no current ready card after this closeout

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.015` seam is another crate-family operator-view uplift or a
planning pause.
