# 2026-03-22 - g08.019 integrated acceptance closure and g08.020 handoff

## Summary

Closed `g08.019` by proving the integrated live-ownership and workflow
acceptance seam through one grouped supervisor export path, then opened
`g08.020` as the active closeout queue.

## Work completed

- added the grouped export proof for the integrated live-ownership and
  workflow acceptance lane
- widened the repo-owned Effigy lane to run that grouped proof instead of
  stopping at descriptor-only grouping
- marked `g08.019` complete across the contract, roadmap, and feature trail
- opened `g08.020` with the first closeout and downstream workflow readiness
  roadmap surface

## Validation

- `cargo fmt --all`
- `cargo test -p signal-supervisor-tools export_json_carries_cross_family_integrated_live_workflow_acceptance_evidence -- --nocapture`
- `cargo test -p signal-supervisor-tools integrated_live_workflow_acceptance_lane_json_reports_required_and_deferred_policy -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-integrated-live-workflow-acceptance-lane --format=json`
- `effigy acceptance:integrated-live-ownership-and-workflow-acceptance-lane`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Next Task

Continue `g08.020` with Batch 20.1 by freezing the shared generation closeout
and downstream workflow readiness contract on top of the closed `g08.019`
integrated acceptance seam.
