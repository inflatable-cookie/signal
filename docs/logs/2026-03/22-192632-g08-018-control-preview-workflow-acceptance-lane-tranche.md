# 2026-03-22 - g08.018 Batch 18.2 control and preview workflow acceptance lane

## Summary

- added the first repo-owned grouped descriptor for shared control-surface and
  preview workflow acceptance
- wired the runnable Effigy lane on top of the existing advanced-hardware and
  preview-transform proof spine
- kept device-native and browser-native workflow reruns explicitly advisory or
  deferred instead of folding them into the mandatory lane

## Evidence

- `cargo fmt --all`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_control_preview_workflow_acceptance_lane_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools control_preview_workflow_acceptance_lane_json_reports_required_and_deferred_policy -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-control-preview-workflow-acceptance-lane --format=json`
- `effigy acceptance:control-preview-workflow-acceptance-lane`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Next Task

Continue `g08.018` with Batch 18.3 by proving the widened control-surface and
preview workflow acceptance seam through shared runtime, supervisor, and
stable host-edge surfaces without introducing a device-private or
browser-local workflow shell.
