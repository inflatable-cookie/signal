# 2026-03-19 - g08.006 immersive boundary closure and g08.007 handoff

## Summary

Closed Batch 6.3 and `g08.006` by widening the existing shared spatial
consumer seam to the new immersive room-policy substrate, then opened
`g08.007`.

## Changes

- updated the existing `signal.runtime.spatial-boundary` descriptor to point at
  the immersive room-policy contract and describe immersive topology,
  plugin-chain, and render-preview anchors
- reused the existing spatial acceptance lane instead of creating a second
  overlapping immersive acceptance shell
- marked `g08.006` complete and opened
  `docs/roadmaps/g08/007-speaker-deployment-fold-down-and-monitoring-scene-depth.md`
  as the next active milestone

## Validation

- `cargo fmt --all`
- `effigy test --plan`
- `cargo test -p signal-supervisor-tools spatial_boundary_text_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo test -p signal-supervisor-tools spatial_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-spatial-boundary --format=json`
- `effigy acceptance:spatial-boundary`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Residual Risk

This closes the bounded immersive room-policy consumer seam, not speaker
deployment, fold-down, monitoring-scene depth, or renderer-capability
negotiation. Those remain the next `g08` milestones.

## Next Task

Continue `g08.007` with Batch 7.1 by freezing the first runtime-owned speaker
deployment, fold-down, and monitoring-scene contract on top of the closed
immersive room-policy seam.
