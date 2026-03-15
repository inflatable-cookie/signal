# g06.008 - Deferred-Work Scheduler Policy Contract Opening Tranche

Date: 2026-03-14
Milestone: `g06.008`
Batch: `8.1`
Status: complete

## Summary

Opened the deferred-work scheduler policy lane with one bounded runtime-owned
contract. Signal now has a frozen vocabulary for deferred-work classes,
priority bands, starvation, backpressure, and cancellation, explicitly tied to
the already-closed timing and hotspot receipt families.

## What changed

- added
  `docs/contracts/019-deferred-work-scheduler-priority-backpressure-and-cancellation-contract.md`
- froze one runtime-owned scheduler-policy authority chain across:
  - `RuntimeDeferredServiceReceipt`
  - `RuntimeTransportConcurrencySnapshot`
  - offline render queue, progress, purge, and continuity receipts
  - `RuntimeEngineBlockSnapshot`
  - `RuntimePerformanceSnapshot`
  - `RuntimePerformanceTraceReceipt`
- defined the first bounded deferred-work priority bands:
  - `CriticalRecovery`
  - `UserBlockingFinalization`
  - `Maintenance`
  - `AdvisoryAnalysis`
- made starvation, backpressure, and cancellation explicit shared terms instead
  of host-local queue heuristics
- recorded that distributed or remote orchestration still remains deferred

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Deferred

- richer runtime-owned priority, starvation, backpressure, and cancellation
  receipts
- stable consumer-facing proof for that widened orchestration family
- any distributed or remote deferred-work scheduler ownership

## Next

Continue `g06.008` with Batch 8.2 by implementing richer runtime-owned
priority, starvation, backpressure, and cancellation receipts on top of the
closed timing, hotspot, and deferred-service boundary.
