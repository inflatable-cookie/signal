# 047 - g09.015 Honest Local Launch Targets

Status: complete
Owner: core-product
Updated: 2026-04-10
Parent roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`
Governing contracts: `docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md`, `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`, `docs/contracts/081-operator-visible-interactive-demo-and-low-dependency-ui-contract.md`

## Objective

Make the plugin capability browser honest about local launchability by removing
derived local launch targets and switching plugin launch roots to exact
per-plugin bundle/library roots.

## Why This Batch Exists

`043` made the browser visible and repo-owned, but real local launches still
used inferred targets:

- local launch buttons were being derived from server-discovered inventory
- VST3 and AU launch roots were broad directory scan roots instead of exact
  bundle roots
- some local VST3 launches therefore scanned unrelated installed bundles,
  emitted Objective-C duplicate-class warnings, and timed out before returning
  a bounded result

The browser needs one follow-on truth pass before broader live interaction
planning is honest.

## Scope

- let VST3 and AU discovery accept exact bundle roots as scan inputs
- update browser inventory examples to emit exact per-plugin launch roots
- stop deriving local launch targets from server inventory
- show local launch buttons only for plugins returned by the local scan surface
- keep bounded timeout/failure output explicit when a local launch still hangs

## Out Of Scope

- plugin editor embedding
- richer persistent transport or plugin-session control
- solving arbitrary installed-plugin launch hangs in one broad pass

## Acceptance Criteria

- the official plugin browser no longer launches local VST3/AU plugins through
  whole-directory roots
- the browser only shows local launch buttons for plugins returned by the local
  scan inventory
- fixture-backed proof still passes through `effigy demo:plugin-capability-browser`

## Validation

- `cargo check -p signal-plugin-vst3 -p signal-plugin-au -p signal-host-local --example plugin_capability_scan -p signal-host-server --example plugin_capability_scan`
- `python3 demos/scripts/run_plugin_capability_browser_demo.py --scan-mode fixture`
- `effigy demo:plugin-capability-browser`
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- exact-root browser inventory and receipt output
- updated plugin browser manifest/scenario wording
- roadmap and log closeout reflecting the narrowed browser truth

## Stop Conditions

- exact per-plugin roots are still not sufficient for bounded local launch
- honest local launchability requires a deeper host-interaction design pass not
  already covered by the current contracts

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.015` seam is deeper live plugin interaction, broader plugin-browser
operator posture, or a planning pause.
