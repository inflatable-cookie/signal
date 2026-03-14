# g06.006 - Block Timing Boundary Proof Closure And g06.007 Handoff

Date: 2026-03-14
Milestone: `g06.006`
Batch: `6.3`
Status: complete

## Summary

Closed the bounded per-block timing milestone as a consumer-facing boundary.
The new timing and deadline-pressure fields are now proven consumable through
public `signal-runtime` reports, both stable host-edge `supervisor_report()`
surfaces, and a repo-owned `signal-supervisor-tools` descriptor plus Effigy
acceptance task.

## What changed

- added a downstream-style runtime proof:
  - `public_runtime_block_timing_boundary_reports_bounded_runtime_measurements`
- added stable host-edge timing proofs:
  - `local_shared_host_edge_exports_runtime_block_timing_truth`
  - `server_shared_host_edge_exports_runtime_block_timing_truth`
- added `signal-supervisor-tools --describe-block-timing-boundary`
- added `effigy acceptance:block-timing-boundary --repo .`
- updated the timing contract, roadmap, and reference docs to mark `g06.006`
  complete and move the active queue to `g06.007`

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime public_runtime_block_timing_boundary_reports_bounded_runtime_measurements -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_runtime_block_timing_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_block_timing_truth -- --nocapture`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_block_timing_boundary_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools block_timing_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-block-timing-boundary --format=json`
- `effigy acceptance:block-timing-boundary --repo .`

## Deferred

- critical-path, hot-node, and worker-lane attribution remain deferred to
  `g06.007`
- host callback cadence remains advisory evidence rather than a canonical
  runtime timing authority
- broader tracing, history buffers, and long-running timing analytics remain
  later profiling and acceptance work

## Next

Continue `g06.007` with Batch 7.1 by freezing graph critical-path, hot-node,
and worker-lane instrumentation semantics on top of the closed per-block timing
boundary before deeper scheduler attribution work begins.
