# 2026-03-22 - g08.015 Device Workflow Acceptance Closure And g08.016 Handoff

## Summary

- closed `g08.015` by adding one grouped supervisor export proof for the
  repo-owned `signal.runtime.device-workflow-acceptance-lane`
- widened the runnable Effigy lane so it now composes existing device-boundary
  proofs, the grouped export proof, and the machine-readable grouped
  descriptor
- marked `g08.015` complete and opened `g08.016` as the next active queue for
  Linux live backend acceptance and failure-injection depth

## Validation

- `effigy test --plan`
- `cargo fmt --all`
- `cargo test -p signal-supervisor-tools export_json_carries_cross_family_device_workflow_acceptance_evidence -- --nocapture`
- `cargo test -p signal-supervisor-tools device_workflow_acceptance_lane_json_reports_required_and_deferred_policy -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-device-workflow-acceptance-lane --format=json`
- `effigy acceptance:device-workflow-acceptance-lane`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Next Task

Continue `g08.016` with Batch 16.1 by freezing the shared live Linux backend
acceptance and failure-injection contract on top of the closed live ownership,
JACK coordination, PipeWire/ALSA parity, and clock-topology seams.
