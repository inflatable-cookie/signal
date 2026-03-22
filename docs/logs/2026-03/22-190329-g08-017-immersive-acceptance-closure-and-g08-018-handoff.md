# 2026-03-22 - g08.017 immersive acceptance closure and g08.018 handoff

## Summary

Closed `g08.017` by adding the grouped consumer-facing supervisor export proof
for the shared immersive acceptance lane, then opened `g08.018` for
control-surface and preview workflow acceptance depth.

## Work completed

- widened `signal-supervisor-tools` so the immersive acceptance lane now
  requires one grouped supervisor export proof spanning immersive room-policy,
  deployment-monitoring, and renderer-export truth
- widened `effigy acceptance:immersive-acceptance-lane` to run that grouped
  export proof alongside the existing grouped descriptor and spatial-boundary
  proof
- closed `docs/roadmaps/g08/017-immersive-render-and-monitoring-acceptance-depth.md`
  and marked contract `068` complete
- opened `docs/roadmaps/g08/018-control-surface-and-preview-workflow-acceptance-depth.md`
  as the next active queue
- rolled the shared index and feature-reference trail forward

## Validation

- `cargo fmt --all`
- `cargo test -p signal-supervisor-tools export_json_carries_cross_family_immersive_acceptance_evidence -- --nocapture`
- `cargo test -p signal-supervisor-tools immersive_acceptance_lane_json_reports_required_and_deferred_policy -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-immersive-acceptance-lane --format=json`
- `effigy acceptance:immersive-acceptance-lane`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Next Task

Continue `g08.018` with Batch 18.1 by freezing the shared control-surface and
preview workflow acceptance contract on top of the closed advanced-hardware,
workflow, preview-transform, and preview-device consumer seams.
