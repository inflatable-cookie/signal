# g01.007 Runtime Transition Export Tests Tranche

Date: 2026-03-11
Roadmap: `docs/roadmaps/g01/007-runtime-transport-scheduler-and-engine-processing-baseline.md`

## Summary

Completed the second `007.3` diagnostics item by adding engine-block-backed
transition tests that validate runtime-owned export surfaces across seek,
loop-wrap, restart, degraded recovery, and prework invalidation behavior.

## What Changed

- extended `RuntimeBlockExecutionSummary` so exported block state includes:
  - prework cache state
  - prework freshness state
  - last prework invalidation reason
- extended `RuntimeDegradationSummary` so exported degradation state includes:
  - recovery-overlap session count
  - lingering session count
- projected those additions through compact, multiline, and JSON supervisor
  output
- added real engine-path tests for:
  - seek invalidation boundary plus follow-up processed seek state
  - loop-wrap export projection
  - recovery-overlap degradation export projection
  - restart/reconfigure report coherence after resumed engine processing

## Outcome

Runtime export testing is now tied to actual engine-block execution for the
transition classes the roadmap called out, rather than relying only on raw
snapshot checks or non-processing lifecycle paths.

## Validation

- `cargo test -p signal-runtime`
- `effigy health`
- `effigy validate`
- `effigy test`
- touched-file `git diff --check`

## Deferred

- `007.3` still needs documentation of remaining host-only behavior that has not
  been promoted into runtime authority
- later mixer-topology debugging still needs more explicit node/lane export
  shaping beyond the current summaries
