# 2026-03-19 - g08.007 spatial boundary closure and g08.008 handoff

## Summary

Closed `g08.007` by widening the existing shared spatial consumer boundary to
the new deployment, fold-down, and monitoring-scene seam, then opened
`g08.008`.

## Changes

- updated the existing `signal.runtime.spatial-boundary` descriptor to point at
  `docs/contracts/058-speaker-deployment-fold-down-and-monitoring-scene-contract.md`
  instead of stopping at the earlier immersive room-policy contract
- widened the machine-readable boundary to describe deployment-aware,
  folded-down, and fallback-monitoring topology, stage, and render-preview
  anchors alongside `deployment_monitoring`
- kept the repo-owned `acceptance:spatial-boundary` lane reusable rather than
  inventing a second overlapping monitoring descriptor
- closed `g08.007` in the roadmap and contract trail
- opened
  `docs/roadmaps/g08/008-renderer-capability-negotiation-and-immersive-export-baseline.md`
  as the next active milestone

## Validation

- `cargo fmt --all`
- `cargo test -p signal-supervisor-tools spatial_boundary_text_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo test -p signal-supervisor-tools spatial_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-spatial-boundary --format=json`
- `effigy acceptance:spatial-boundary`

## Residual Risk

This closes the bounded deployment and monitoring consumer seam, not
renderer-capability negotiation, immersive export packaging, or deeper
renderer-backed monitoring breadth. Those are now the next `g08` milestones.

## Next Task

Continue `g08.008` with Batch 8.1 by freezing the first runtime-owned
renderer-capability negotiation and immersive export contract on top of the
closed deployment, fold-down, and monitoring-scene seam.
