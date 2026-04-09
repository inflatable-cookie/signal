# 09-320500 - g09.012 Sandbox Lifecycle Demo Bootstrap Closeout

Status: complete
Owner: core-product
Updated: 2026-04-09
Roadmap refs: docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Spec refs: docs/specs/batch-cards/022-g09-012-sandbox-lifecycle-demo-bootstrap.md

## Summary

Closed the first `g09.012` live-demo batch by turning the existing
`signal-plugin-sandbox` broker binary into one official repo-owned demo surface
with a manifest, Effigy launch task, operator notes, and a generated receipt.

## Implementation

- fixed the unrelated `signal-analysis-tonal` root-import test failure by
  restoring test-local access to the tuning helpers without widening the public
  crate boundary
- added the official live demo manifest in
  `demos/manifests/plugin-sandbox-lifecycle.demo.json`
- added the operator notes in
  `demos/scenarios/plugin-sandbox-lifecycle.default.md`
- added the demo runner script in
  `demos/scripts/run_sandbox_lifecycle_demo.py`
- added the repo-owned launch task `effigy demo:sandbox-lifecycle`
- generated the receipt in
  `demos/receipts/plugin-sandbox-lifecycle.receipt.json`
- promoted `signal-plugin-sandbox` and `signal-ipc` from deferred to live
  coverage in the demo coverage matrix

## Validation

- `cargo test -p signal-analysis-tonal --lib --no-run`
- `effigy demo:sandbox-lifecycle`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`
- attempted `effigy validate`
  - still fails outside this batch in the pre-existing split-test module trees
    for `crates/signal-host-local/tests/public_host_edge_boundary.rs` and
    `crates/signal-host-server/tests/public_host_edge_boundary.rs`

## Remaining Deferred Truth

- plugin capability browsing is still deferred
- runtime and host live demo surfaces are still deferred
- hardware live demo surfaces are still deferred
- DSP, graph, and analysis live demo surfaces remain deferred to `g09.013`

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.012` seam is plugin capability browsing, runtime/host demo
bootstrap, or a planning pause before creating another ready card.
