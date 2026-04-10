# Plugin Capability Browser

Status: active
Updated: 2026-04-10

## Operator Goal

Use one lightweight official surface to inspect discovered plugin inventory and
launch one bounded host path for a selected plugin without introducing a
product-style UI shell into Signal.

## Recommended Launch

- interactive operator path: `effigy demo:plugin-capability-browser`
- bounded proof path: `effigy demo:plugin-capability-browser:proof`

## What To Look For

- the browser lists real discovered plugin types rather than a hardcoded demo
  catalog
- each row keeps format, vendor, features, and plugin type identity visible
- each row now shows local/server availability explicitly instead of leaving
  that posture implicit in the launch column
- launch buttons stay explicit about which host surface they drive
- local launch buttons appear only when the plugin was actually returned by the
  bounded local exact-root probe surface
- the launch panel shows clear passed, failed, or timeout posture before the
  raw bounded host detail
- the launch panel now also surfaces bounded interaction proof explicitly:
  interaction mode, applied value, and parameter-event truth rather than boot
  success alone
- interactive system mode now prefers bounded local inventory first on macOS,
  then adds bounded server availability where the confirmed roots allow it

## Known Limits

- editor embedding is intentionally out of scope
- launch paths now prove one bounded host-owned parameter-step interaction, not
  persistent session shells or embedded editor control
- local buttons are intentionally absent when the local scan surface fails or
  times out; the browser does not infer local launchability from server-only
  discovery
- the default interactive path now uses bounded exact-root scan batches, so one
  problematic installed plugin should not suppress the whole browser inventory
- if no suitable installed CLAP or VST3 plugin is available, the official proof
  task may fall back to one bounded temporary VST3 fixture root so the browser
  surface remains testable

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.015` seam is another crate-family operator-view uplift, deeper live
plugin interaction, or a planning pause.
