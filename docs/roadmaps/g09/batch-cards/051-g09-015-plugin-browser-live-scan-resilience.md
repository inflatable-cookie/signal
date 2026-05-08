# 051 - g09.015 Plugin Browser Live-Scan Resilience

Status: complete
Owner: core-product
Updated: 2026-04-10
Parent roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`
Governing contracts: `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`, `docs/contracts/081-operator-visible-interactive-demo-and-low-dependency-ui-contract.md`

## Objective

Make the interactive plugin browser survive real machine plugin roots without
blanking the inventory, hanging indefinitely, or leaving scan subprocesses
behind.

## Why This Batch Existed

The browser surface was honest enough to expose its failures, but still too
fragile on real plugin roots:

- a killed server scan could blank the inventory
- broad directory scans could take too long for an interactive operator surface
- `Ctrl+C` could leave child scan processes running
- stale browser listeners could make relaunching fail on the default port

## Scope

- contain host scan subprocesses so timeout and interrupt tear down the whole
  scan tree
- break broad system-root scans into bounded exact-root batches
- prefer bounded local inventory first on macOS, then add bounded server
  enrichment where possible
- auto-bind the browser server to a free localhost port when the default is in
  use

## Out Of Scope

- embedded plugin editor windows
- persistent plugin sessions
- exhaustive machine-wide scan guarantees across every installed plugin

## Acceptance Criteria

- the browser no longer dies just because one host scan process is killed
- interactive system-mode runs return some bounded inventory on a real machine
  instead of blanking the browser wholesale
- `Ctrl+C` does not leave scan subprocesses behind
- a stale listener on `8765` does not make relaunch fail

## Validation

- `effigy demo:plugin-capability-browser:proof`
- `python3 demos/scripts/run_plugin_capability_browser_demo.py --scan-mode system`
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- updated plugin browser scan containment and serve startup logic
- generated browser receipt showing bounded inventory and launch truth from a
  real system-mode run
- batch log with validation actually run

## Outcome

- interactive browser scans now use bounded exact-root batches with smaller
  time budgets
- macOS system-mode runs now prefer bounded local inventory first and then add
  bounded server enrichment across locally confirmed roots
- scan subprocesses are torn down as process groups on timeout or interrupt
- browser serve startup now auto-selects a free localhost port instead of
  dying on `8765`
- there is no current ready card after this closeout

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.015` seam is another crate-family operator-view uplift, deeper live
plugin interaction, or a planning pause.
