# 2026-03-22 - g08.015 Batch 15.2 Device Workflow Acceptance Lane

## Summary

- added the first repo-owned grouped device-workflow acceptance descriptor to
  `signal-supervisor-tools` as
  `signal.runtime.device-workflow-acceptance-lane`
- added the runnable Effigy lane
  `effigy acceptance:device-workflow-acceptance-lane`, composed from the
  already-closed external MIDI, controller-expression, control-surface, and
  advanced-hardware acceptance tasks
- rolled the contract, roadmap, generation index, and feature reference
  forward so the active next step is `g08.015` Batch 15.3 consumer-proof
  closure

## Validation

- `effigy test --plan`
- `cargo fmt --all`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_device_workflow_acceptance_lane_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools device_workflow_acceptance_lane_json_reports_required_and_deferred_policy -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-device-workflow-acceptance-lane --format=json`
- `effigy acceptance:device-workflow-acceptance-lane`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Next Task

Continue `g08.015` with Batch 15.3 by proving the widened device workflow
acceptance seam through shared runtime, supervisor, and stable host-edge
surfaces without introducing a backend-local endpoint or workflow policy shell.
