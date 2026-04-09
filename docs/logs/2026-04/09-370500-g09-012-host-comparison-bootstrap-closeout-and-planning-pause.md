# 09-370500 - g09.012 Host Comparison Bootstrap Closeout And Planning Pause

Status: complete
Owner: core-product
Updated: 2026-04-09
Roadmap refs: docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Spec refs: docs/specs/batch-cards/025-g09-012-local-server-host-comparison-bootstrap.md

## Summary

Closed the bounded local-versus-server host comparison bootstrap batch, then
paused the strict lane at planning because the next `g09.012` plugin and
hardware demo seams still need fresh judgment before another honest ready card
exists.

## Implementation

- added the official live host comparison manifest in
  `demos/manifests/local-server-host-comparison.demo.json`
- added the operator notes in
  `demos/scenarios/local-server-host-comparison.default.md`
- added the receipt generator in
  `demos/scripts/run_local_server_host_comparison_demo.py`
- added the repo-owned launch task `effigy demo:local-server-host-comparison`
- generated the receipt in
  `demos/receipts/local-server-host-comparison.receipt.json`
- promoted `signal-host-local` and `signal-host-server` from deferred to live
  demo coverage in the matrix
- kept plugin capability browsing and hardware topology demos explicitly
  deferred instead of folding them into this host wrapper

## Validation

- `effigy demo:local-server-host-comparison`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Planning Result

- did not promote plugin capability browsing as the next ready card because it
  still wants fresh planning judgment about demo-owned scan roots and how much
  of the scan surface should be machine-browsable versus receipt-only
- did not promote hardware topology and diagnostics as the next ready card
  because the live simulated/native backend split still wants bounded planning
  rather than another ad hoc bootstrap batch

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.012` seam is plugin capability browsing, hardware diagnostics, or a
continued planning pause.
