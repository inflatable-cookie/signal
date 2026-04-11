# 060 - g09.015 Platform Boundary Operator Views

Status: complete
Owner: core-product
Updated: 2026-04-11
Parent roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`
Governing contracts: `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`, `docs/contracts/081-operator-visible-interactive-demo-and-low-dependency-ui-contract.md`

## Objective

Promote the remaining receipt-only platform boundary demos into rendered
low-dependency operator views so the macOS AU/CoreAudio and Linux LV2/backend
surfaces are visually inspectable without reading raw receipts first.

## Why This Batch Exists

After `059`, the remaining receipt-only live demo surfaces are the two
platform-boundary demos:

- `signal.demo.macos.au-coreaudio-boundary`
- `signal.demo.linux.lv2-backend-boundary`

This is the next honest seam because:

- both demos already capture bounded descriptor-backed and acceptance-backed
  platform truth
- the remaining gap is presentation, not new host, plugin, or hardware
  behavior
- the two surfaces are structurally similar enough to uplift together without
  becoming another tiny batch sequence

## Scope

- add rendered companion views for the macOS AU/CoreAudio boundary demo and the
  Linux LV2/backend boundary demo
- keep platform-specific truth explicit rather than flattening it into one
  pseudo-cross-platform shell
- align manifests, operator notes, receipts, and coverage notes to the
  rendered views

## Out Of Scope

- new AU, LV2, or backend behavior
- live device control
- generalized plugin browsing redesign
- replacing the underlying receipt surfaces

## Acceptance Criteria

- `effigy demo:macos-au-coreaudio-boundary` emits a rendered companion view
- `effigy demo:linux-lv2-and-backend-boundary` emits a rendered companion view
- both surfaces remain browser-native and low-dependency

## Validation

- `effigy demo:macos-au-coreaudio-boundary`
- `effigy demo:linux-lv2-and-backend-boundary`
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- rendered macOS AU/CoreAudio boundary companion view
- rendered Linux LV2/backend boundary companion view
- manifests, receipts, and operator notes aligned to the rendered views
- batch log with validation actually run

## Stop Conditions

- the uplift would require inventing new platform behavior instead of
  presenting existing proof data
- either platform receipt is too thin to support an honest rendered view
  without another planning step

## Next Task

Re-enter planning at the `g09.015` checkpoint and decide whether the lane can
close on completed operator-visible coverage or whether one final deeper live
plugin-interaction tranche is still honestly required.
