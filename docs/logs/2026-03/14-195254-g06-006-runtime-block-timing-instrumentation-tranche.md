# g06.006 - Runtime Block Timing Instrumentation Tranche

Date: 2026-03-14
Milestone: `g06.006`
Batch: `6.2`
Status: complete

## Summary

Turned the per-block timing contract into a real runtime-owned measurement seam.
`signal-runtime` now records bounded block execution duration, derives deadline
budget and overrun posture, and exports that same timing truth through the
existing block and performance snapshot families.

## What changed

- added `RuntimeBlockDeadlinePressure` to `signal-runtime`
- extended `RuntimeEngineBlockSnapshot` with bounded block timing fields:
  - latest execution duration
  - derived deadline budget
  - utilization percent
  - overrun amount
  - pressure classification
  - bounded overrun and peak timing counters
- aligned `RuntimeBlockExecutionSummary`, `RuntimePerformanceSnapshot`, and
  `RuntimePerformanceTraceReceipt` to the same timing seam
- instrumented `process_engine_block()` so measured block execution updates the
  runtime-owned timing fields and now feeds runtime `cpu_load_percent` and
  `graph_latency_ms` once real blocks have been processed
- added focused runtime proofs for:
  - real block-path timing capture
  - deterministic pressure classification and trace rollup
  - updated performance snapshot and trace behavior after timing instrumentation

## Validation

- `effigy test --plan --repo .`
  - failed as expected in this workspace because Effigy falls through to
    `ctest --plan`, and this CTest build does not support `--plan`
- `cargo fmt --all`
- `cargo test -p signal-runtime --lib --no-run`
- `cargo test -p signal-runtime --lib runtime_process_engine_block_records_bounded_timing_and_budget_fields -- --nocapture`
- `cargo test -p signal-runtime --lib runtime_block_timing_pressure_rolls_into_performance_snapshot_and_trace_receipt -- --nocapture`
- `cargo test -p signal-runtime --lib runtime_performance_ -- --nocapture`

## Deferred

- consumer-facing boundary proof for the new timing receipts
- stable host-edge and supervisor-tools descriptor coverage
- hot-node, critical-path, and worker-lane timing attribution
- any broader tracing or history-buffer depth

## Next

Continue `g06.006` with Batch 6.3 by proving the new per-block timing and
pressure snapshots remain consumable through shared runtime, supervisor, and
host-edge surfaces without private tracing hooks.
