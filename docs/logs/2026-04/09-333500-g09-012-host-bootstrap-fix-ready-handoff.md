# 09-333500 - g09.012 Host Bootstrap Fix Ready Handoff

Status: complete
Owner: core-product
Updated: 2026-04-09
Roadmap refs: docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Spec refs: docs/roadmaps/g09/batch-cards/024-g09-012-host-demo-bootstrap-fix.md

## Summary

Re-entered planning after the runtime inspector closeout and promoted the next
honest `g09.012` ready card around the existing host demo bring-up failure.

## Decision

- chose host demo bootstrap fix as the next seam
- did not keep the lane paused because the failure is narrow, reproducible, and
  batch-cardable
- did not choose plugin capability browsing yet because it still wants fresh
  planning judgment around demo-owned scan roots

## Ready Surface

- new ready card:
  `docs/roadmaps/g09/batch-cards/024-g09-012-host-demo-bootstrap-fix.md`
- governing executable surfaces:
  `crates/signal-host-local/src/main.rs`
  `crates/signal-host-server/src/main.rs`

## Validation

- `effigy qa:docs`
- `effigy qa:northstar`

## Next Task

Continue the active strict `g09` lane from
`docs/roadmaps/g09/batch-cards/024-g09-012-host-demo-bootstrap-fix.md`.
