# 10-291500 - g09.015 Honest Local Launch Targets Closeout

## Summary

Closed the follow-on browser-truth batch after `043`.

The plugin capability browser no longer infers local launch buttons from
server-only discovery, and VST3/AU local launch roots are no longer broad
directory scan roots. The browser now uses exact per-plugin bundle/library
roots, and local launch buttons appear only when the local scan surface
actually returned that plugin.

## Meaningful Changes

- widened VST3 discovery in
  `crates/signal-plugin-vst3/src/vst3_host_adapter/discovery.rs` so exact
  `.vst3` bundle paths can be used as scan roots
- widened AU discovery in
  `crates/signal-plugin-au/src/au_host_adapter/discovery.rs` so exact
  `.component` bundle paths can be used as scan roots
- updated the local and server browser inventory examples in
  `crates/signal-host-local/examples/plugin_capability_scan.rs` and
  `crates/signal-host-server/examples/plugin_capability_scan.rs` to emit exact
  per-plugin launch roots instead of parent directory roots
- removed derived local browser targets from
  `demos/scripts/run_plugin_capability_browser_demo.py`
- added a bounded local-scan gate in the browser wrapper so local buttons are
  only shown when the local scan surface succeeds for those plugins
- kept local scan failure explicit in the browser exclusions instead of
  pretending local launchability

## Validation

- `cargo check -p signal-plugin-vst3 -p signal-plugin-au -p signal-host-local --example plugin_capability_scan -p signal-host-server --example plugin_capability_scan`
- `python3 demos/scripts/run_plugin_capability_browser_demo.py --scan-mode fixture`
- `effigy demo:plugin-capability-browser`

## Notes

- this batch does not claim that every installed local plugin now launches
  cleanly
- it narrows the browser claim so local launch buttons are no longer offered
  from unverified inferred targets

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.015` seam is deeper live plugin interaction, broader plugin-browser
operator posture, or a planning pause.
