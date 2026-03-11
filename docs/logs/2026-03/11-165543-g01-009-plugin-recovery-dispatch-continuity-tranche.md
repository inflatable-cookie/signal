# 2026-03-11 16:55:43 GMT - g01.009 plugin recovery dispatch continuity tranche

## Summary

Advanced `g01.009 / 009.3` by pinning that runtime-owned plugin dispatch truth
survives timeout fallback, watchdog restart, safe-mode escalation, and overlap
recovery while the plugin-backed node remains attached to the same engine graph
path as native nodes.

This closes the `009.3` recovery/fallback validation slice. The remaining work
for the milestone is now the broader topology proof: plugin-backed nodes need
to be exercised more explicitly as first-class participants in the emerging
track-lane, console-node, and bus-oriented graph semantics.

## What changed

- extended `crates/signal-host-local/src/host.rs` with
  `LocalPluginDispatchSummary` plus internal capture of:
  - the actual `PluginRenderContext` sent to the sandbox for the last block
  - the runtime-owned automation value injected into the payload
  - bound-node render override continuity, including bypass count plus the last
    latency/tail/bypass state applied back into runtime
- updated the host-local realtime block path so each block records the plugin
  dispatch truth it used before sending the request and the render-override
  truth it applied after reading the completion slot outcome
- expanded host-local coverage to pin:
  - single-block timeout fallback still bypasses through the bound plugin node
    without detaching graph binding
  - timeout recovery summaries keep runtime dispatch truth aligned with engine
    execution context after lease rollover
  - watchdog restart, safe-mode escalation, and mixed watchdog soak retain the
    same dispatch truth on the bound plugin seam
  - overlap recovery replacement sessions continue dispatching through the
    bound plugin node with runtime-owned transport/parameter state

## Validation

- `cargo fmt`
- `cargo test -p signal-host-local`

## Follow-on

The next `009.3` batch should prove plugin-backed nodes remain first-class in
the track-lane, console-node, and bus-oriented topology/reporting path rather
than only as a single demo insert with recovery coverage.
