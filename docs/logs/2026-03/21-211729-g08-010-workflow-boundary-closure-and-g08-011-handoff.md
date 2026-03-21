# 2026-03-21 21:17:29 - g08.010 workflow boundary closure and g08.011 handoff

## Summary

Batch 10.3 closed `g08.010` by widening the existing advanced-hardware
consumer seam to the new control-surface workflow contract and opening
`g08.011` as the next active milestone.

## Delivered

- updated `signal-supervisor-tools` so
  `signal.runtime.advanced-hardware-boundary` now points at the
  control-surface workflow contract and explicitly describes scene-mapping,
  feedback-page, and safe-action workflow anchors
- widened the advanced-hardware boundary tests so the descriptor proves the new
  workflow counts and per-device posture or authority fields directly
- closed `g08.010` across the roadmap, contract, and feature-reference trail
- opened `g08.011` with a new roadmap file for preview-output routing,
  audition-sink ownership, and low-latency device policy

## Validation

- `cargo fmt --all`
- `cargo test -p signal-supervisor-tools advanced_hardware_boundary_text_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo test -p signal-supervisor-tools advanced_hardware_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-advanced-hardware-boundary --format=json`
- `effigy acceptance:advanced-hardware-boundary`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Next Task

Continue `g08.011` with Batch 11.1 by freezing the first runtime-owned
preview-output routing, audition-sink ownership, and low-latency device-policy
contract on top of the closed controller and workflow seams.
