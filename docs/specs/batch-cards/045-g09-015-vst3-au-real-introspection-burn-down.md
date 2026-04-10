# 045 - g09.015 AU Real Component Introspection And VST3 Split

Status: complete
Owner: core-product
Updated: 2026-04-10
Parent roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`
Governing contracts: `docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md`, `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`, `docs/contracts/081-operator-visible-interactive-demo-and-low-dependency-ui-contract.md`

## Objective

Replace the remaining AU metadata-file discovery shim with real component
bundle introspection, and split the still-deeper VST3 work into its own batch
so the plugin browser remains honest about installed-plugin discovery posture.

## Scope

- remove `signal-au-component.txt` dependency from the AU production discovery
  path
- keep test fixtures and public proof roots aligned to the real discovery shape
- split the remaining VST3 class-factory discovery seam explicitly if the AU
  path proves narrower and landable first

## Out Of Scope

- building the browser UI itself
- plugin editor or custom GUI embedding
- widening into LV2 execution or unrelated host/runtime work

## Acceptance Criteria

- AU production discovery no longer depends on Signal-specific `.txt`
  metadata files
- the remaining VST3 discovery blocker is narrowed into its own ready card
  instead of hidden inside a falsely complete combined batch

## Validation

- `cargo check -p signal-plugin-au`
- `cargo test -p signal-plugin-au --lib`
- direct local and server host runs against temporary `.component` bundles
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- updated roadmap/currentness surfaces reflecting the AU closeout and VST3
  split
- code path changes showing AU `.txt` metadata discovery removal
- batch log with validation actually run

## Stop Conditions

- installed VST3 bundles on the active machine still do not provide a cheap
  metadata-only replacement for Signal's current factory/controller truth, so
  class-factory discovery must be split into a deeper follow-on batch

## Next Task

Continue the reopened strict `g09` lane from
`docs/specs/batch-cards/046-g09-015-vst3-class-factory-discovery-burn-down.md`.
