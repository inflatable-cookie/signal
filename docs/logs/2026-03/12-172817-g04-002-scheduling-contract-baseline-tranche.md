# g04.002 Scheduling Contract Baseline Tranche

Date: 2026-03-12
Scope: `docs/contracts/`, `docs/architecture/`, `docs/roadmaps/g04/`

## Summary

Completed Batch 2.1 of `g04.002` by freezing the runtime-owned multicore
scheduling contract before deepening execution behavior.

## What changed

- added `docs/contracts/004-runtime-multicore-scheduling-and-anticipative-execution-contract.md`
  to define the scheduler authority hierarchy across
  `RuntimeEngineBlockSnapshot`, `RuntimeSchedulerSnapshot`,
  `RuntimeSchedulerExportSummary`, and `RuntimeExecutionTopologySummary`
- documented which scheduler behavior is expected to remain deterministic under
  the same graph/runtime mode and which parts may vary by runtime profile,
  forecast policy, or degraded operating state
- recorded the rule that hosts and tools must consume runtime-owned scheduler
  receipts rather than rebuilding a parallel multicore policy model from local
  graph traversal or callback-thread assumptions
- updated the graph/runtime feature reference and `g04` roadmap docs so the
  queue now moves from contract freezing into runtime scheduling depth

## Why this tranche

`g04.002` could not safely deepen multicore execution while the inspection model
for planning groups, execution lanes, dispatch order, and anticipative service
state was still partly implicit. This tranche makes the scheduler contract
explicit first.

## Validation

- `cargo test -p signal-runtime runtime_scheduler`
- `cargo test -p signal-runtime runtime_realtime_block`
- `cargo test -p signal-runtime runtime_recovery_overlap_throttles_realtime_scheduler_under_normal_pressure`
- `git diff --check`
- `effigy health --repo .`

## Next

Continue `g04.002` with Batch 2.2 and deepen runtime multicore scheduling and
anticipative execution against this frozen contract.
