# g06.006 - Per-Block Timing Contract Opening Tranche

Date: 2026-03-14
Milestone: `g06.006`
Batch: `6.1`
Status: complete

## Summary

Froze the first per-block execution timing and pressure snapshot contract so the
profiling lane can move from causal diagnostics into bounded runtime-owned
measurement semantics without jumping straight to full tracing.

## What changed

- added `docs/contracts/017-per-block-execution-timing-and-pressure-snapshot-contract.md`
- froze the authority hierarchy around:
  - `RuntimeEngineBlockSnapshot` as the per-block timing authority
  - `RuntimeSchedulerSnapshot` as control-state context
  - `RuntimePerformanceSnapshot` and `RuntimePerformanceTraceReceipt` as the
    bounded consumer and automation digests
- explicitly treated host callback cadence and backend timing as advisory
  evidence rather than a competing timing authority
- updated the active roadmap, contract index, generation pointers, and runtime
  feature reference to close Batch 6.1 and queue Batch 6.2 instrumentation

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Deferred

- actual per-block timing instrumentation fields
- deadline-pressure and budget-overrun runtime export
- consumer-facing proof surfaces for the new measurements
- hot-node, critical-path, and worker-lane timing depth

## Next

Continue `g06.006` with Batch 6.2 by instrumenting bounded per-block execution
timing, deadline pressure, and budget-overrun fields on the frozen runtime-owned
measurement seam, then align supervisor and host-edge export to the same
observations.
