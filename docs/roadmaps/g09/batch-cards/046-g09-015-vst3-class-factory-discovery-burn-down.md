# 046 - g09.015 VST3 Class-Factory Discovery Burn-Down

Status: complete
Owner: core-product
Updated: 2026-04-10
Parent roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`
Governing contracts: `docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md`, `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`, `docs/contracts/081-operator-visible-interactive-demo-and-low-dependency-ui-contract.md`

## Objective

Replace the remaining Signal-specific VST3 metadata-file discovery shims with
real bundle and class-factory introspection so installed VST3 browsing can be
claimed honestly before the plugin browser batch resumes.

## Scope

- remove `signal-vst3-module.txt` and `signal-vst3-factory.txt` dependency from
  the VST3 production discovery path
- derive module, factory, and plugin-class identity from real `.vst3` bundle
  and binary introspection
- keep test fixtures and public proof roots aligned to the real VST3 discovery
  shape
- leave the browser deferred until VST3 installed-plugin discovery is honest

## Out Of Scope

- building the browser UI itself
- AU work already closed in `045`
- widening into unrelated host/runtime/plugin execution work

## Acceptance Criteria

- VST3 production discovery no longer depends on Signal-specific `.txt`
  metadata files
- VST3 discovery produces enough real module and class identity to support an
  honest installed-plugin browser claim
- the next browser batch is honest about supported installed-plugin browsing
  posture across CLAP, VST3, and AU

## Validation

- `cargo check -p signal-plugin-vst3`
- focused VST3 discovery and host parity proof commands
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- updated roadmap/currentness surfaces
- code path changes showing VST3 `.txt` metadata discovery removal
- batch log with validation actually run

## Stop Conditions

- real VST3 class-factory introspection needs deeper contract or architecture
  decisions than are currently frozen in the plugin-hosting surfaces
- platform-specific binary/factory inspection proves broad enough to require a
  narrower sub-split before honest closeout

## Next Task

Continue the reopened strict `g09` lane from
`docs/roadmaps/g09/batch-cards/043-g09-015-plugin-capability-browser-bootstrap.md`.
