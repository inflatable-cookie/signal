# g06.007 - Critical-Path Boundary Proof Closure And g06.008 Handoff

Date: 2026-03-14
Milestone: `g06.007`
Batch: `7.3`
Status: complete

## Summary

Closed the bounded hotspot and lane boundary with downstream-style consumer
proof. The widened critical-path, hot-node, hot-group, and worker-lane receipt
family is now proven through public runtime surfaces, both stable host edges,
and a machine-readable supervisor-tools descriptor with a repo-owned acceptance
task.

## What changed

- added the downstream-style runtime proof:
  - `public_runtime_critical_path_boundary_reports_bounded_hotspot_receipts`
- added stable host-edge proofs:
  - `local_shared_host_edge_exports_runtime_critical_path_truth`
  - `server_shared_host_edge_exports_runtime_critical_path_truth`
- added the machine-readable critical-path boundary descriptor to
  `signal-supervisor-tools`:
  - `--describe-critical-path-boundary`
- added the repo-owned acceptance task:
  - `effigy acceptance:critical-path-boundary`
- closed `g06.007` and activated `g06.008`

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime public_runtime_critical_path_boundary_reports_bounded_hotspot_receipts -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_runtime_critical_path_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_critical_path_truth -- --nocapture`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_critical_path_boundary_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools critical_path_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-critical-path-boundary --format=json`
- `effigy acceptance:critical-path-boundary --repo .`

## Deferred

- deeper scheduler attribution beyond the current bounded hot-node, hot-group,
  critical-path lane, and worker-lane summaries
- node-by-node elapsed-time traces, flamegraph exports, and host thread
  telemetry
- deferred-work scheduler priority, starvation, backpressure, and cancellation
  policy, which now moves to `g06.008`

## Next

Continue `g06.008` with Batch 8.1 by defining deferred-work scheduler
priority, backpressure, starvation, and cancellation semantics on top of the
closed timing, hotspot, and orchestration receipt families.
