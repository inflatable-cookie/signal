# 09-374500 - g09.012 Hardware Diagnostics Ready Handoff

Status: complete
Owner: core-product
Updated: 2026-04-09
Roadmap refs: docs/roadmaps/g09/012-runtime-host-plugin-and-hardware-interactive-demo-suite.md
Spec refs: docs/roadmaps/g09/batch-cards/027-g09-012-hardware-topology-diagnostics-bootstrap.md

## Summary

Re-entered planning after the host comparison closeout and promoted the next
honest `g09.012` seam as a bounded hardware topology and diagnostics bootstrap
batch.

## Planning Result

- did not promote plugin capability browsing because the live-demo shape for
  owned scan roots and machine-browsable scan results still wants fresh design
  judgment
- promoted hardware diagnostics instead because the current host binaries
  already export the needed native-versus-simulated device, backend, and
  endpoint truth without requiring a new executable
- the new ready card is
  `docs/roadmaps/g09/batch-cards/027-g09-012-hardware-topology-diagnostics-bootstrap.md`

## Currentness Updates

- refreshed the strict front doors so `027` is the active ready card
- updated the `g09.012` roadmap with the planning result that hardware
  diagnostics is the next honest seam after host comparison
- left plugin capability browsing explicitly deferred

## Validation

- `effigy qa:docs`
- `effigy qa:northstar`

## Next Task

Continue the active strict `g09` lane from
`docs/roadmaps/g09/batch-cards/027-g09-012-hardware-topology-diagnostics-bootstrap.md`.
