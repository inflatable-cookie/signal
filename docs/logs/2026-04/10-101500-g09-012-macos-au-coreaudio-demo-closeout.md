# 10-101500 - g09.012 macOS AU CoreAudio Demo Closeout

Status: complete
Owner: core-product
Updated: 2026-04-10
Roadmap refs: docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Spec refs: docs/specs/batch-cards/029-g09-012-macos-au-coreaudio-demo-bootstrap.md

## Summary

Closed the bounded macOS AU/CoreAudio demo bootstrap batch by wrapping the
existing descriptor and acceptance surfaces in one repo-owned live demo, then
returned the strict lane to planning because plugin browsing and Linux-native
coverage still need fresh judgment before another honest card exists.

## Implementation

- added the official live macOS AU/CoreAudio manifest in
  `demos/manifests/macos-au-coreaudio-boundary.demo.json`
- added the operator notes in
  `demos/scenarios/macos-au-coreaudio-boundary.default.md`
- added the receipt generator in
  `demos/scripts/run_macos_au_coreaudio_boundary_demo.py`
- added the repo-owned launch task
  `effigy demo:macos-au-coreaudio-boundary`
- generated the receipt in
  `demos/receipts/macos-au-coreaudio-boundary.receipt.json`
- promoted `signal-plugin-au` from deferred to live demo coverage in the matrix
- kept plugin capability browsing explicitly deferred instead of folding scan
  root design into this macOS-specific demo

## Validation

- `cargo run -q -p signal-supervisor-tools -- --describe-macos-au-coreaudio-boundary --format=json`
- `effigy acceptance:macos-au-coreaudio-boundary`
- `effigy demo:macos-au-coreaudio-boundary`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Planning Result

- plugin capability browsing remains deferred because demo-owned scan-root and
  browse-posture decisions are still not frozen tightly enough for a ready
  execution card
- Linux-native backend and LV2 demo coverage also still wants fresh planning
  judgment rather than another automatic ready card

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.012` seam is plugin capability browsing, Linux-native backend/LV2
demo coverage, or a continued planning pause.
