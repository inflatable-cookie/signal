# g09.015 - Plugin Browser Live-Scan Resilience Closeout

Status: complete
Date: 2026-04-10
Batch card: `docs/roadmaps/g09/batch-cards/051-g09-015-plugin-browser-live-scan-resilience.md`
Roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`

## Summary

Closed the plugin browser live-scan resilience batch.

- browser scans now run in isolated process groups and are torn down explicitly
  on timeout or interrupt
- interactive system scans now use bounded exact-root batches instead of broad
  all-or-nothing directory scans
- macOS interactive runs now prefer bounded local inventory first, then add
  bounded server enrichment over locally confirmed roots
- browser serve startup now auto-selects a free localhost port instead of
  failing when `8765` is already in use

## Important Reality Notes

- this is a resilience and usability batch, not a claim of exhaustive
  machine-wide plugin enumeration
- the proof task remains the stable automatable validation path
- the interactive path now degrades in bounded ways instead of crashing,
  blanking the inventory, or leaving child scans behind

## Validation Run

- `effigy demo:plugin-capability-browser:proof`
- `python3 demos/scripts/run_plugin_capability_browser_demo.py --scan-mode system`
- `effigy qa:docs`
- `effigy qa:northstar`

## Outcome

- `051-g09-015-plugin-browser-live-scan-resilience.md` is complete
- the browser produced a real system-mode receipt on this machine with:
  - `plugin_count=18`
  - `launch_target_count=23`
  - one passed bounded local VST3 launch
- `g09.015` remains active, but there is no current ready card after this
  closeout

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.015` seam is another crate-family operator-view uplift, deeper live
plugin interaction, or a planning pause.
