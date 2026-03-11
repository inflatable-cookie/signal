# 2026-03-11 17:03:15 GMT - g01.009 plugin topology and reporting closure

## Summary

Closed `g01.009 / 009.3` by exposing the plugin-backed topology path directly in
the local host summary and pinning that both the local summary and the shared
host/runtime report preserve the same track-lane, bus, and console-node route
for the bound sandboxed plugin across default boot and timeout recovery.

With this tranche, `g01.009` now has evidence for all four `009.3` goals:
plugin-backed node render integration, runtime-owned transport/parameter
dispatch truth, degraded recovery/fallback continuity, and first-class
participation in the emerging node-oriented mixer topology.

## What changed

- extended `crates/signal-host-local/src/host.rs` so
  `LocalRuntimeHostSummary` now carries the runtime-derived execution topology
  summary alongside the existing transport, payload, and plugin dispatch
  details
- added host-local assertions that the local summary preserves the same
  plugin-backed topology route as runtime observation:
  - `track-input` on `track:lead`
  - `plugin-insert` bound to `local-default-sandbox`
  - `bus-main` on `mix:master`
  - `output-main` on `console:main`
- strengthened shared-report coverage so compact/multiline/json rendering is
  pinned to include the plugin-backed node route on both default boot and
  timeout recovery paths
- updated roadmap status and evidence markers so `g01.009` is now recorded as
  complete

## Validation

- `cargo fmt`
- `cargo test -p signal-host-local`

## Completion

`g01.009` is complete.
