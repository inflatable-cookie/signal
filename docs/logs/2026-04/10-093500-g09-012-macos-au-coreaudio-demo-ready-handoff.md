# 10-093500 - g09.012 macOS AU CoreAudio Demo Ready Handoff

Status: complete
Owner: core-product
Updated: 2026-04-10
Roadmap refs: docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Spec refs: docs/roadmaps/g09/batch-cards/029-g09-012-macos-au-coreaudio-demo-bootstrap.md

## Summary

Re-entered planning after the supervisor runtime companion closeout and
promoted the next honest `g09.012` seam: a macOS-specific AU/CoreAudio live
demo bootstrap.

## Planning Basis

- plugin capability browsing remains underplanned because demo-owned scan-root
  and browse-posture decisions are still not frozen tightly enough for a ready
  execution card
- the macOS AU/CoreAudio boundary is already frozen by `g09.004` and the repo
  has both a machine-readable descriptor command and a dedicated acceptance lane
  for that surface
- that makes a macOS AU/CoreAudio demo bootstrap cleaner and more bounded than
  inventing the plugin browsing surface next

## Validation

- `cargo run -q -p signal-supervisor-tools -- --describe-macos-au-coreaudio-boundary --format=json`
- `effigy qa:docs`
- `effigy qa:northstar`

## Next Task

Continue the active strict `g09` lane from
`docs/roadmaps/g09/batch-cards/029-g09-012-macos-au-coreaudio-demo-bootstrap.md`.
