# g01.007 Scheduler Topology Diagnostics Closure

Date: 2026-03-11
Roadmap: `docs/roadmaps/g01/007-runtime-transport-scheduler-and-engine-processing-baseline.md`

## Summary

Closed the last open `007.2` scheduler-enforcement item by making runtime own a
topology-compatibility summary that is both validated in tests and projected
through shared diagnostics surfaces.

## What Changed

- extended graph contract summaries with lane and bus-group identity so runtime
  can reason about future track-lane, bus, send/return, and console grouping
  without host-side reconstruction
- added `RuntimeSchedulerTopologySummary` and
  `RuntimeSchedulerTopologyIssue` to `signal-runtime`
- refreshed the topology summary whenever graph, schedule, configure, or engine
  block execution state changes
- added runtime tests for:
  - missing schedule projections for track-lane groups
  - insufficient schedule stream counts
  - missing track-lane metadata
  - compatible topology execution through a real engine block
  - projection of scheduler topology into compact, multiline, and JSON runtime
    reports

## Outcome

Runtime now publishes whether its current lane ordering and schedule projection
can represent the applied graph topology directly. The compatibility decision is
available through runtime-owned diagnostics rather than requiring Finch or later
hosts to reinterpret raw lane order, dispatch order, and graph shape on their
own.

## Validation

- `cargo test -p signal-runtime`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy test --repo .`
- touched-file `git diff --check`

## Deferred

- `007.3` still needs broader supervisor/diagnostic expansion beyond scheduler
  topology, especially more explicit degradation and transport/scheduler
  reporting on shared export surfaces
