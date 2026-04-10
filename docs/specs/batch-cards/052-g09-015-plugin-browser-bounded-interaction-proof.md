# 052 - g09.015 Plugin Browser Bounded Interaction Proof

Status: complete
Owner: core-product
Updated: 2026-04-10
Parent roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`
Governing contracts: `docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md`, `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`, `docs/contracts/081-operator-visible-interactive-demo-and-low-dependency-ui-contract.md`

## Objective

Deepen the plugin browser from bounded host bootstrap only into one bounded
live interaction proof, without widening into embedded plugin-editor hosting or
persistent session shells.

## Why This Batch Exists

The browser is now honest enough to browse and launch real installed plugins on
this machine. The next missing value is not more scan polish; it is one visible
proof that launch can become interaction.

The honest next slice is bounded parameter/event interaction because:

- the host demo surfaces already carry demo plugin env overrides
- the runtime and plugin layers already expose parameter/event reporting
- this can stay a proof surface over existing host/runtime behavior instead of
  turning into a full product UI or persistent plugin workstation

## Scope

- add one bounded interaction step to the plugin browser launch path
- surface parameter or plugin-event truth in the browser result instead of
  launch-only bootstrap summaries
- keep the interaction bounded, repeatable, and host-owned
- keep unsupported editor/session breadth explicit

## Out Of Scope

- embedded vendor editor windows
- arbitrary transport playback controls
- persistent session management
- exhaustive plugin-specific interaction support across every installed plugin

## Acceptance Criteria

- the browser can demonstrate one bounded post-launch interaction step for at
  least one supported plugin path
- the launch result area exposes interaction truth more specific than boot
  success alone
- the proof remains low-dependency and browser-native

## Validation

- `effigy demo:plugin-capability-browser:proof`
- `python3 demos/scripts/run_plugin_capability_browser_demo.py --scan-mode system`
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- updated browser interaction result surface
- receipt and operator notes aligned to the bounded interaction proof
- batch log with validation actually run

## Outcome

- added one host-owned `parameter-step` interaction contract to the local and
  server plugin browser launch paths through demo env overrides
- surfaced interaction truth explicitly in host launch summaries:
  `interaction_mode`, `automation_value`, `parameter_events`, and generated
  event bytes
- updated the browser launch result area and receipt checks so the surface now
  proves bounded interaction instead of bootstrap-only launch success
- validated the bounded interaction proof in both fixture mode and real
  system-scan mode without widening into editor embedding or persistent session
  control

## Stop Conditions

- the bounded interaction step would require editor embedding or long-lived
  session control that does not already exist
- current host demo plumbing cannot surface one meaningful parameter/event proof
  without broader runtime/host redesign

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.015` seam is another crate-family operator-view uplift, deeper live
plugin interaction, or a planning pause.
