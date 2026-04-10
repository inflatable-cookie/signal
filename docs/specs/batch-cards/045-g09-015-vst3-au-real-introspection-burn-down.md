# 045 - g09.015 VST3 AU Real Introspection Burn-Down

Status: ready
Owner: core-product
Updated: 2026-04-10
Parent roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`
Governing contracts: `docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md`, `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`, `docs/contracts/081-operator-visible-interactive-demo-and-low-dependency-ui-contract.md`

## Objective

Replace the remaining Signal-specific VST3 and AU metadata-file discovery
shims with real bundle/module/component introspection so the future plugin
browser can claim installed-plugin discovery honestly.

## Scope

- remove `signal-vst3-module.txt` and `signal-vst3-factory.txt` dependency from
  the VST3 production discovery path
- remove `signal-au-component.txt` dependency from the AU production discovery
  path
- keep test fixtures and public proof roots aligned to the real discovery shape
- leave the browser deferred until CLAP, VST3, and AU discovery are all honest

## Out Of Scope

- building the browser UI itself
- plugin editor or custom GUI embedding
- widening into LV2 execution or unrelated host/runtime work

## Acceptance Criteria

- VST3 production discovery no longer depends on Signal-specific `.txt`
  metadata files
- AU production discovery no longer depends on Signal-specific `.txt`
  metadata files
- the next browser batch is honest about supported installed-plugin browsing
  posture across CLAP, VST3, and AU

## Validation

- `effigy health`
- focused VST3 and AU discovery and host parity proof commands
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- updated roadmap/currentness surfaces
- code path changes showing `.txt` metadata discovery removal
- batch log with validation actually run

## Stop Conditions

- real VST3 class-factory or AU component metadata introspection needs deeper
  contract or architecture decisions not yet frozen in the plugin-hosting
  surfaces
- the VST3 and AU removal work proves too broad for one bounded batch and must
  be split explicitly

## Next Task

Implement this card by removing the remaining VST3 and AU metadata-file
discovery shims before the browser resumes.
