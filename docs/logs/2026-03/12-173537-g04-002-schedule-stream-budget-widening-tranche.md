# g04.002 Schedule Stream Budget Widening Tranche

Date: 2026-03-12
Scope: `crates/signal-runtime/`, `docs/architecture/`, `docs/roadmaps/g04/`

## Summary

Completed the first implementation tranche of Batch 2.2 by making compatible
schedule-stream capacity affect runtime anticipative service behavior rather
than remaining report-only topology metadata.

## What changed

- `signal-runtime` now widens anticipative prework service budget from
  compatible `ScheduleProjection.stream_count` during normal-pressure runtime
  servicing instead of treating schedule width as export-only topology context
- added focused runtime proofs that:
  - a compatible wider schedule increases the effective prework service budget
  - a missing schedule projection does not widen service scope
  - the existing normal-pressure realtime scheduler path remains stable
- updated the scheduling contract and architecture reference so the runtime-owned
  scheduler model now explicitly includes this schedule-width-driven budget
  widening behavior

## Why this tranche

Batch 2.1 froze the inspection hierarchy, but runtime still did not act on the
declared schedule width. This tranche turns one scheduler receipt into real
runtime-owned behavior without widening into unsafe host-local concurrency or a
premature thread model.

## Validation

- `cargo test -p signal-runtime runtime_compatible_schedule_projection_widens_normal_prework_service_budget`
- `cargo test -p signal-runtime runtime_missing_schedule_projection_does_not_widen_prework_service_budget`
- `cargo test -p signal-runtime runtime_realtime_block_services_prework_window_under_normal_pressure`
- `cargo test -p signal-runtime runtime_scheduler`
- `git diff --check`
- `effigy health`

## Next

Continue `g04.002` with the next Batch 2.2 runtime-depth tranche and deepen
dispatch/service behavior beyond budget scaling while preserving realtime-safe
invalidation, transport, and degradation boundaries.
