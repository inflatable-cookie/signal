# g01.007 Runtime Export Summaries Tranche

Date: 2026-03-11
Roadmap: `docs/roadmaps/g01/007-runtime-transport-scheduler-and-engine-processing-baseline.md`

## Summary

Completed the first `007.3` diagnostics item by promoting transport-adjacent
runtime state into explicit shared export summaries instead of requiring hosts
to reconstruct it from raw snapshots and event streams.

## What Changed

- added `RuntimeSchedulerExportSummary`, `RuntimeBlockExecutionSummary`, and
  `RuntimeDegradationSummary` to `signal-runtime`
- derived those summaries directly from the runtime-owned engine snapshot,
  transport concurrency snapshot, supervision snapshot, and observation
  diagnostics during `RuntimeObservationReport::capture(...)`
- projected the new summaries through compact, multiline, and JSON supervisor
  export surfaces
- tightened the runtime test surface so both block-backed observation capture
  and event-recorder-backed supervisor capture assert the new summary contract

## Outcome

Runtime exports now present three stable, host-consumable views on top of the
existing raw data:

- scheduler intent and topology compatibility
- last block execution and transport boundary state
- degradation and gating state across watchdog, transport, broker, and plugin
  fault conditions

This reduces the amount of host-local interpretation needed to explain runtime
behavior while keeping the underlying raw snapshots available.

## Validation

- `cargo test -p signal-runtime`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy test --repo .`
- touched-file `git diff --check`

## Deferred

- `007.3` still needs engine-block-backed transition tests for seek, loop wrap,
  restart, degraded recovery, and prework invalidation
- node/lane execution detail is still present mostly as raw snapshot structure;
  later mixer-topology debugging will need more explicit export shaping
