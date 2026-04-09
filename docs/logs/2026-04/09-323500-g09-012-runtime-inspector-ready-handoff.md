# 09-323500 - g09.012 Runtime Inspector Ready Handoff

Status: complete
Owner: core-product
Updated: 2026-04-09
Roadmap refs: docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Spec refs: docs/specs/batch-cards/023-g09-012-runtime-recovery-inspector-bootstrap.md

## Summary

Re-entered planning after the sandbox lifecycle bootstrap closeout and promoted
the next honest `g09.012` ready card around the existing runtime supervisor
report example.

## Decision

- chose runtime recovery inspector bootstrap as the next seam
- did not choose plugin capability browsing yet because the live demo posture
  for plugin scan roots still wants fresh judgment about how to avoid leaning
  on test-only setup helpers
- did not widen into host comparison because the existing runtime example is a
  narrower, cleaner executable starting point

## Ready Surface

- new ready card:
  `docs/specs/batch-cards/023-g09-012-runtime-recovery-inspector-bootstrap.md`
- governing executable surface:
  `crates/signal-runtime/examples/supervisor_report_demo.rs`

## Validation

- `effigy qa:docs`
- `effigy qa:northstar`

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/023-g09-012-runtime-recovery-inspector-bootstrap.md`.
