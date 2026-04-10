# 09-390500 - g09.012 Supervisor Runtime Companion Ready Handoff

Status: complete
Owner: core-product
Updated: 2026-04-09
Roadmap refs: docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Spec refs: docs/specs/batch-cards/028-g09-012-supervisor-runtime-boundary-companion.md

## Summary

Re-entered planning after the hardware diagnostics closeout and promoted the
next honest `g09.012` seam: a `signal-supervisor-tools` companion surface for
the existing runtime recovery inspector family.

## Planning Basis

- plugin capability browsing remains underplanned because demo-owned scan-root
  and browse-posture decisions are still not frozen tightly enough for a ready
  execution card
- `signal-supervisor-tools` is the one remaining deferred crate in the runtime
  demo family and it already exposes stable machine-readable runtime boundary
  descriptors through its current CLI
- that makes a supervisor companion bootstrap card cleaner and more bounded
  than inventing a plugin browse surface next

## Validation

- `cargo run -q -p signal-supervisor-tools -- --describe-interruption-boundary --format=json`
- `cargo run -q -p signal-supervisor-tools -- --describe-fault-diagnostic-boundary --format=json`
- `effigy qa:docs`
- `effigy qa:northstar`

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/028-g09-012-supervisor-runtime-boundary-companion.md`.
