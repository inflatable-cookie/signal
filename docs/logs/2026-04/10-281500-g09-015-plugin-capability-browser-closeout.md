# g09.015 - Plugin Capability Browser Closeout

Status: complete
Date: 2026-04-10
Batch card: `docs/roadmaps/g09/batch-cards/043-g09-015-plugin-capability-browser-bootstrap.md`
Roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`

## Summary

Closed the first official operator-visible plugin browser batch.

- added `signal.demo.plugin.capability-browser` with:
  - manifest in `demos/manifests/plugin-capability-browser.demo.json`
  - operator notes in `demos/scenarios/plugin-capability-browser.default.md`
  - repo-owned wrapper in `demos/scripts/run_plugin_capability_browser_demo.py`
  - generated HTML artifact in `demos/receipts/plugin-capability-browser.view.html`
  - generated receipt in `demos/receipts/plugin-capability-browser.receipt.json`
- added dedicated local/server scan examples:
  - `crates/signal-host-local/examples/plugin_capability_scan.rs`
  - `crates/signal-host-server/examples/plugin_capability_scan.rs`
- promoted the plugin browser into the live coverage matrix and Effigy demo
  task surface

## Important Reality Notes

- interactive terminal runs can browse system plugin roots
- the official non-interactive proof task uses a bounded fixture-backed VST3
  scan by default because arbitrary installed plugins can still hang during
  discovery on this machine
- the browser remains intentionally bounded:
  - inventory plus host launch posture
  - no embedded vendor editor UI
  - no long-lived plugin session shell

## Validation Run

- `cargo check -p signal-plugin-clap`
- `cargo check -p signal-host-local --example plugin_capability_scan`
- `cargo check -p signal-host-server --example plugin_capability_scan`
- `python3 demos/scripts/run_plugin_capability_browser_demo.py`
- `python3 -m json.tool demos/manifests/plugin-capability-browser.demo.json > /dev/null`
- `python3 -m json.tool demos/receipts/plugin-capability-browser.receipt.json > /dev/null`

## Outcome

- `043-g09-015-plugin-capability-browser-bootstrap.md` is complete
- `signal.demo.plugin.capability-browser` is no longer deferred in
  `demos/coverage-matrix.*`
- `g09.015` remains active, but there is no current ready card after this
  closeout

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.015` seam is deeper live plugin interaction, broader plugin-browser
operator posture, or a planning pause.
