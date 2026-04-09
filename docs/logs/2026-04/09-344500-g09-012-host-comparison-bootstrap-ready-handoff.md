# 09-344500 - g09.012 Host Comparison Bootstrap Ready Handoff

Status: complete
Owner: core-product
Updated: 2026-04-09
Roadmap refs: docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Spec refs: docs/specs/batch-cards/025-g09-012-local-server-host-comparison-bootstrap.md

## Summary

Re-entered planning after the host bootstrap-fix closeout and promoted the
next honest `g09.012` ready card around a bounded local-versus-server host
comparison demo surface.

## Decision

- chose host comparison next because both host binaries now boot cleanly
  through the bounded supported demo path from `024`
- kept the seam narrow around manifest, launch, receipt, and operator-notes
  wrapping of the existing host binaries
- did not choose plugin capability browsing yet because it still wants fresh
  planning judgment around demo-owned scan roots and browse posture
- did not keep the lane paused because the comparison seam is now executable
  without new substrate design

## Ready Surface

- new ready card:
  `docs/specs/batch-cards/025-g09-012-local-server-host-comparison-bootstrap.md`
- governing executable surfaces:
  `crates/signal-host-local/src/main.rs`
  `crates/signal-host-server/src/main.rs`
  `demos/`

## Validation

- `effigy qa:docs`
- `effigy qa:northstar`

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/025-g09-012-local-server-host-comparison-bootstrap.md`.
