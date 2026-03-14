# g06.007 - Runtime Hotspot And Worker-Lane Depth Tranche

Date: 2026-03-14
Milestone: `g06.007`
Batch: `7.2`
Status: complete

## Summary

Turned the bounded critical-path contract into a real runtime-owned
instrumentation seam. `signal-runtime` now derives hot-group membership,
critical-path lane attribution, and typed worker-lane summaries directly from
the existing engine-block planning and lane-order snapshot instead of leaving
that explanation implicit in host-side tooling.

## What changed

- extended `RuntimePerformanceSnapshot` with:
  - `hot_latency_group_node_count`
  - `critical_path_lane`
  - `critical_path_lane_node_count`
  - `critical_path_lane_plugin_backed_node_count`
  - `critical_path_lane_planning_group_count`
  - `critical_path_lane_total_latency_samples`
  - `worker_lane_summaries`
- added `RuntimeWorkerLaneInstrumentationSummary` as the typed per-lane digest
  for node count, plugin-backed node count, planning-group count, total lane
  latency, and maximum node latency
- extended `RuntimePerformanceTraceReceipt` with peak hot-group membership and
  peak critical-path lane fields so bounded hotspot evidence survives the
  observation window
- kept all widened hotspot and lane attribution derived from
  `RuntimeEngineBlockSnapshot` planning and lane-order truth instead of
  introducing host-local scheduler reconstruction
- strengthened the focused runtime proofs for performance snapshot and trace
  rollup behavior to cover the new critical-path lane and worker-lane fields

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime --lib --no-run`
- `cargo test -p signal-runtime runtime_performance_snapshot_captures_scheduler_pressure_and_background_policy -- --nocapture`
- `cargo test -p signal-runtime runtime_performance_trace_receipt_summarizes_playback_recording_and_deferred_work_window -- --nocapture`
- `cargo test -p signal-runtime runtime_performance_ -- --nocapture`

## Deferred

- public and stable host-edge proof for the widened hotspot and lane receipts
- a machine-readable consumer boundary for the new critical-path surface
- deeper scheduler attribution beyond the current bounded node, group, and lane
  summaries

## Next

Continue `g06.007` with Batch 7.3 by proving the widened critical-path,
hot-node, and worker-lane receipts remain consumable through shared runtime,
supervisor, and stable host-edge surfaces without private runtime hooks.
