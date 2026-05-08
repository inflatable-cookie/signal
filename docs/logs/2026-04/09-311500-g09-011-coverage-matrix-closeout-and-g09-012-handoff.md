# 09-311500 - g09.011 Coverage Matrix Closeout And g09.012 Handoff

Status: complete
Owner: core-product
Updated: 2026-04-09
Roadmap refs: docs/roadmaps/g09/011-interactive-demo-substrate-manifest-and-operator-conventions.md, docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Spec refs: docs/roadmaps/g09/batch-cards/021-g09-011-demo-coverage-matrix.md, docs/roadmaps/g09/batch-cards/022-g09-012-sandbox-lifecycle-demo-bootstrap.md

## Summary

Closed the final `g09.011` substrate batch by freezing the first active-workspace
crate-to-demo coverage matrix, then handed the strict lane into `g09.012`
through a bounded sandbox lifecycle bootstrap card.

## Implementation

- added the machine-readable workspace coverage inventory in
  `demos/coverage-matrix.json`
- added the human-readable grouped matrix in `demos/coverage-matrix.md`
- updated `demos/README.md` so the shared substrate now names the coverage
  files and the boundary no longer claims matrix work is deferred
- added `demo:coverage-matrix` in `effigy.toml`
- marked `g09.011` complete and activated `g09.012`
- promoted the next ready card:
  `docs/roadmaps/g09/batch-cards/022-g09-012-sandbox-lifecycle-demo-bootstrap.md`

## Validation

- `effigy demo:coverage-matrix`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`
- attempted `effigy validate`
  - failed outside this batch in `signal-analysis-tonal` lib-test compilation
    because `crates/signal-analysis-tonal/src/tests.rs` imports
    `cents_offset_from_standard` and `reference_hz_from_cents` from the crate
    root, but those items are no longer exported there

## Remaining Deferred Truth

- there are still no live official demo manifests under `demos/manifests/`
- the first live demo surface is deferred to `g09.012`
- DSP, graph, and analysis live demo surfaces remain deferred to `g09.013`

## Next Task

Continue the active strict `g09` lane from
`docs/roadmaps/g09/batch-cards/022-g09-012-sandbox-lifecycle-demo-bootstrap.md`.
