# 2026-03-18 - g07.012 controller-expression boundary closure and g07.013 handoff

## Summary

Closed `g07.012` by proving the widened MIDI 2.0, MPE, and richer
controller-expression receipt family through public runtime, both stable host
edges, and a machine-readable supervisor-tools descriptor.

This tranche turns the Batch 12.2 runtime baseline into a real consumer seam
instead of leaving widened controller-expression meaning dependent on
adapter-private packet models or host-local capability reconstruction.

## Key changes

- added the focused public runtime proof for widened controller-expression
  posture and bounded external-device capability posture
- added stable local-host and server-host proofs that widened
  controller-expression truth survives `supervisor_report()` without packet
  reconstruction
- exposed the machine-readable
  `signal.runtime.controller-expression-boundary` descriptor in
  `signal-supervisor-tools`
- added the repo-owned Effigy rerun lane
  `acceptance:controller-expression-boundary`
- closed `g07.012` and promoted `g07.013` to active so control-surface work can
  widen from the now-closed external MIDI and controller-expression substrate

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime public_runtime_controller_expression_boundary_reports_runtime_owned_expression_truth -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_runtime_controller_expression_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_controller_expression_truth -- --nocapture`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_controller_expression_boundary_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools controller_expression_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-controller-expression-boundary --format=json`
- `effigy acceptance:controller-expression-boundary --repo .`

## Residual risk

This closes the bounded controller-expression proof seam, not full MIDI 2.0
UMP transport, negotiation, profile exchange, or control-surface mapping and
feedback depth. Those remain the next queue by design.

## Next Task

Continue `g07.013` with Batch 13.1 by freezing the runtime-owned
control-surface transport, mapping, feedback, and capability contract on top
of the now-closed external MIDI endpoint and widened controller-expression
boundaries.
