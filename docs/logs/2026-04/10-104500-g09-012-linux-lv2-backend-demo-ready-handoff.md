# 10-104500 - g09.012 Linux LV2 And Backend Demo Ready Handoff

Status: complete
Owner: core-product
Updated: 2026-04-10
Roadmap refs: docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Spec refs: docs/roadmaps/g09/batch-cards/030-g09-012-linux-lv2-and-backend-boundary-demo-bootstrap.md

## Summary

Re-entered planning after the macOS AU/CoreAudio demo closeout and promoted
the next honest `g09.012` seam: a Linux-specific LV2 execution plus
audio-backend boundary demo bootstrap.

## Planning Basis

- plugin capability browsing remains underplanned because demo-owned scan-root
  and browse-posture decisions are still not frozen tightly enough for a ready
  execution card
- the repo already has both machine-readable descriptor commands and dedicated
  acceptance lanes for `linux-lv2-execution-boundary` and
  `linux-audio-backend-boundary`
- that makes a bounded Linux boundary companion cleaner and more honest than
  inventing browse posture next

## Validation

- `cargo run -q -p signal-supervisor-tools -- --describe-linux-lv2-execution-boundary --format=json`
- `cargo run -q -p signal-supervisor-tools -- --describe-linux-audio-backend-boundary --format=json`
- `effigy qa:docs`
- `effigy qa:northstar`

## Next Task

Continue the active strict `g09` lane from
`docs/roadmaps/g09/batch-cards/030-g09-012-linux-lv2-and-backend-boundary-demo-bootstrap.md`.
