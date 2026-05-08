# 09-380500 - g09.012 Hardware Diagnostics Bootstrap Closeout

Status: complete
Owner: core-product
Updated: 2026-04-09
Roadmap refs: docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Spec refs: docs/roadmaps/g09/batch-cards/027-g09-012-hardware-topology-diagnostics-bootstrap.md

## Summary

Closed the bounded hardware topology and diagnostics bootstrap batch by wrapping
the existing local and server host binaries in one repo-owned hardware receipt
surface, then returned the strict lane to planning because plugin capability
browsing still wants fresh scan-root judgment before another honest card exists.

## Implementation

- added the official live hardware diagnostics manifest in
  `demos/manifests/hardware-topology-diagnostics.demo.json`
- added the operator notes in
  `demos/scenarios/hardware-topology-diagnostics.default.md`
- added the receipt generator in
  `demos/scripts/run_hardware_topology_diagnostics_demo.py`
- added the repo-owned launch task `effigy demo:hardware-topology-diagnostics`
- generated the receipt in
  `demos/receipts/hardware-topology-diagnostics.receipt.json`
- promoted `signal-hardware` and `signal-hardware-coreaudio` from deferred to
  live demo coverage in the matrix
- kept plugin capability browsing explicitly deferred instead of blending scan
  browsing into this hardware bootstrap surface

## Validation

- `effigy demo:hardware-topology-diagnostics`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Planning Result

- plugin capability browsing remains deferred because demo-owned scan-root and
  browse-posture decisions still want fresh planning judgment
- no new `g09.012` ready card was promoted automatically after this batch

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.012` seam is plugin capability browsing, another bounded
host/runtime/hardware live-demo batch, or a continued planning pause.
