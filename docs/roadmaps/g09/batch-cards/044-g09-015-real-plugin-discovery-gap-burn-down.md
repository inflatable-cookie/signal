# 044 - g09.015 Real Plugin Discovery Gap Burn-Down

Status: complete
Owner: core-product
Updated: 2026-04-10
Parent roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`
Governing contracts: `docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md`, `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`, `docs/contracts/081-operator-visible-interactive-demo-and-low-dependency-ui-contract.md`

## Objective

Replace the remaining plugin discovery shims that make an installed-plugin
browser dishonest: CLAP harness-only discovery, VST3/AU `.txt` metadata
dependency, and LV2 scaffold-backed direct lookup.

## Scope

- inventory the exact discovery seams that still depend on Signal-specific
  metadata or scaffold lookup
- land the first real CLAP discovery pass for installed plugins
- define and begin the removal path for VST3/AU metadata-file discovery
- remove or explicitly quarantine LV2 scaffold-backed
  `discover_plugin_type(...)` lookup if it still leaks into production paths
- keep the browser work deferred until the discovery claim is honest

## Out Of Scope

- building the browser UI itself
- full plugin editor or custom GUI embedding
- widening into unrelated host/runtime feature work

## Acceptance Criteria

- CLAP no longer depends only on harness-backed `plugin:clap:*` discovery for
  the production discovery path
- the VST3/AU metadata-file dependency is either removed or narrowed into an
  explicit remaining blocker with a concrete next batch
- the next browser batch is honest about what “installed plugin browsing”
  really means

## Outcome

- landed a real CLAP root-scan discovery path using actual `.clap` libraries
  instead of harness-only `plugin:clap:*` ids
- rewired the local and server host CLAP scan/ensure/restart flow to carry the
  scanned CLAP catalog instead of depending on synthetic ids
- moved the CLAP parity proof helpers onto compiled temporary `.clap` fixtures
  so host validation can run through real scan roots
- narrowed the remaining discovery blocker:
  - VST3 and AU still depend on Signal-specific `.txt` metadata shims
  - LV2 still has scaffold-backed direct lookup in the adapter, but that helper
    is no longer part of the active host production discovery path
- promoted the next bounded batch as
  `045-g09-015-vst3-au-real-introspection-burn-down.md`

## Validation

- `effigy health`
- focused plugin discovery and parity proof commands
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- updated roadmap/currentness surfaces
- explicit discovery-gap inventory and the code path changed in this batch
- batch log with validation actually run

## Stop Conditions

- real CLAP discovery needs deeper contract or architecture decisions not yet
  frozen in the plugin-hosting surfaces
- VST3/AU removal of metadata-file discovery proves too broad for one bounded
  batch and must be split explicitly

## Next Task

Continue the reopened strict `g09` lane from
`docs/roadmaps/g09/batch-cards/045-g09-015-vst3-au-real-introspection-burn-down.md`.
