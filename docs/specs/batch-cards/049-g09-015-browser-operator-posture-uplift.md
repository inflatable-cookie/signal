# 049 - g09.015 Browser Operator Posture Uplift

Status: complete
Owner: core-product
Updated: 2026-04-10
Parent roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`
Governing contracts: `docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md`, `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`, `docs/contracts/081-operator-visible-interactive-demo-and-low-dependency-ui-contract.md`

## Objective

Make the plugin capability browser easier to read and trust as an operator
surface by surfacing local/server availability, probe posture, and bounded
launch status directly in the UI instead of burying that truth in exclusions and
raw JSON only.

## Why This Batch Exists

`048` made local visibility more robust, but the browser still behaves more like
an engineer-facing tool than an operator-facing proof surface:

- local versus server availability is not obvious at a glance
- probe containment and degraded local posture are mostly implicit
- the launch area still reads like raw host output rather than an intentional
  verification surface

Before planning deeper live plugin interaction, the current browser needs a
clearer operator posture.

## Scope

- add explicit local/server availability indicators per plugin
- summarize local probe results and bounded launch posture in the browser UI
- make launch result status clearer than raw JSON alone while preserving the
  underlying detail
- keep the browser low-dependency and browser-native

## Out Of Scope

- embedded plugin editor UI
- persistent live plugin transport or stateful plugin sessions
- new host/plugin execution capabilities

## Acceptance Criteria

- an operator can tell at a glance which plugins have local launch, server
  launch, or both
- degraded local probe posture is visible without reading the known-exclusions
  list closely
- the launch result area communicates pass/fail/timeout clearly while still
  exposing bounded host detail

## Validation

- `effigy demo:plugin-capability-browser:proof`
- `python3 demos/scripts/run_plugin_capability_browser_demo.py --scan-mode system`
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- updated browser HTML/operator surface
- receipt and scenario wording aligned to the new operator posture
- batch log with interactive and proof validation actually run

## Stop Conditions

- the posture uplift still depends on deeper persistent session behavior that is
  not already present
- the browser would need a heavier UI/runtime stack to present the added
  clarity

## Outcome

- the browser now shows local/server availability per plugin at a glance
- bounded launch status is summarized as passed, failed, or timed out before
  the raw JSON detail
- the operator notes, manifest, and coverage surfaces now reflect the uplifted
  posture
- there is no current ready card after this closeout

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.015` seam is deeper live plugin interaction, a broader
plugin-browser operator surface, or a planning pause.
