---
title: g01.007 recovery-aware scheduler policy tranche
status: completed
owner: core-product
updated: 2026-03-11
tags: [signal, runtime, scheduler, transport, roadmap, g01.007]
---

## Summary

Completed a meaningful `007.2` runtime batch that moves prework scheduler
policy closer to real runtime authority instead of relying on manual pressure
projection alone.

## Delivered

- made prework scheduler state react to transport recovery conditions, not only
  plugin-backed degradation
- added runtime-owned transport scheduler fields to the engine snapshot:
  recovery overlap count, lingering count, detach-faulted count, and transport
  gate state
- made `reconcile_prework_service_state(...)` surface `Yielding` directly when
  pending prework is blocked by plugin or transport gates
- tightened `service_prework_lane_with_policy(...)` so recovery overlap
  throttles normal-pressure realtime servicing and lingering/faulted transport
  state gates elevated-pressure servicing
- changed transport/session mutation paths to refresh scheduler policy and
  state immediately instead of waiting for a later manual service call
- refreshed configure/start/stop and forecast profile/policy transitions so the
  scheduler state stays coherent when runtime role/profile assumptions change

## Test coverage

- added a realtime engine-block test proving recovery-overlap transport state
  throttles scheduler service under normal pressure
- added a runtime-state test proving lingering transport state pushes the
  scheduler into `Yielding` under elevated pressure once pending prework exists
- added a realtime engine-block test proving forecast profile changes while the
  engine is running keep the scheduler window and service state coherent
- retained the existing restart/reconfigure realtime scheduler coverage and the
  existing degraded plugin gate coverage

## Validation

- `cargo test -p signal-runtime`
- `effigy health`
- `effigy validate`
- `effigy test`
- touched-file `git diff --check`

## Notes

- this tranche completes the first three `007.2` checklist items
- the remaining open `007.2` item is structural: proving scheduler phase/lane
  ordering can carry future track-lane and console-node execution groups
  without host-local reinterpretation
