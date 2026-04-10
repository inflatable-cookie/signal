# 043 - g09.015 Plugin Capability Browser Bootstrap

Status: pending
Owner: core-product
Updated: 2026-04-10
Parent roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`
Governing contracts: `docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md`, `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`, `docs/contracts/081-operator-visible-interactive-demo-and-low-dependency-ui-contract.md`

## Objective

Create the first official operator-visible plugin capability browser so an
operator can browse installed plugins and launch bounded supported interaction
paths through repo-owned commands.

## Status Note

This card remains valid follow-on work, but it is not executable yet. Current
adapter reality still blocks an honest installed-plugin browser:

- CLAP discovery is still harness-backed rather than filesystem-backed
- VST3 and AU discovery still rely on Signal-specific metadata files
- LV2 still keeps scaffold-backed direct `discover_plugin_type(...)` lookup

The browser should resume only after those discovery seams are burned down.

## Scope

- add one official `signal.demo.plugin.capability-browser` surface under
  `demos/`
- keep the UI posture lightweight:
  - browser-native/static asset or terminal/TUI surface
  - no heavyweight product-style UI framework
- browse installed plugin inventory across the supported discovery paths
- expose explicit per-plugin launch affordances for supported live interaction
  paths
- keep unsupported formats, unsupported host paths, and platform exclusions
  explicit in the surface

## Out Of Scope

- full custom plugin editor embedding
- inventing new host support for formats the repo still does not support live
- widening into a general product browser

## Acceptance Criteria

- an operator can launch the official browser through a repo-owned command
- the browser shows discovered plugins and their supported interaction posture
- at least one supported CLAP or VST3 live interaction path is launchable from
  the official surface
- the coverage matrix no longer leaves `signal.demo.plugin.capability-browser`
  in deferred scope

## Validation

- `effigy health`
- focused plugin/host discovery and parity proof commands
- the new official demo launch task
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- manifest, scenario, launch script, and receipt for the browser demo
- updated coverage matrix and roadmap/currentness surfaces
- batch log with validation actually run

## Stop Conditions

- live interaction still requires fresh host-support planning not already
  covered by the current plugin-hosting contracts
- the proposed UI surface cannot stay low-dependency without inventing a new
  shared UI runtime

## Next Task

Resume this card after
`docs/specs/batch-cards/044-g09-015-real-plugin-discovery-gap-burn-down.md`
is complete.
