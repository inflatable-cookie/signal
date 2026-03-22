# 2026-03-22 - g08.018 control and preview workflow acceptance closure

## Summary

- closed the grouped control-surface and preview workflow acceptance seam with
  one explicit supervisor export proof
- widened the repo-owned acceptance lane so it now proves grouped consumer
  evidence rather than only descriptor and task composition
- moved the active roadmap queue forward from `g08.018` to `g08.019`

## Evidence

- `cargo fmt --all`
- `cargo test -p signal-supervisor-tools export_json_carries_cross_family_control_preview_workflow_acceptance_evidence -- --nocapture`
- `cargo test -p signal-supervisor-tools control_preview_workflow_acceptance_lane_json_reports_required_and_deferred_policy -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-control-preview-workflow-acceptance-lane --format=json`
- `effigy acceptance:control-preview-workflow-acceptance-lane`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Next Task

Continue `g08.019` with Batch 19.1 by freezing the shared integrated live-
ownership and workflow acceptance contract on top of the closed Linux live,
device workflow, immersive, and control-preview workflow acceptance seams.
