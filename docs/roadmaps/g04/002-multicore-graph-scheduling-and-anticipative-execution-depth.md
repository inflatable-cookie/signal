# 002 - Multicore Graph Scheduling And Anticipative Execution Depth

Status: complete
Owner: core-product
Created: 2026-03-12
Depends on: g04.001
Vision tags: `ENGINE`, `SCHEDULING`, `PERFORMANCE`

## Problem

`g03` proved routed execution, prework windows, and runtime hardening, but the
engine still needs a stronger reusable answer for multicore graph scheduling and
anticipative execution under real workload variation.

Without a dedicated scheduling-depth milestone:

- multicore execution policy stays partly implicit in local host/runtime paths
- anticipative work remains harder to reason about under changing graph cost
- later hardware and plugin portability work will inherit unclear scheduling
  assumptions
- performance work risks becoming benchmark-first without a stronger execution
  model

## Goals

- [ ] define a reusable multicore scheduling contract inside Signal
- [ ] deepen anticipative execution without breaking timing truth or recovery
- [ ] keep scheduling decisions runtime-owned rather than host-local
- [ ] surface enough runtime-owned receipts to validate the scheduling policy

## Non-Goals

- [ ] no host-specific performance dashboard work
- [ ] no product workflow heuristics
- [ ] no “optimize everything” campaign without an explicit execution model

## Execution Plan

### Batch 2.1 - Scheduling Contract

- [x] define how planning groups, execution classes, and anticipative eligibility
  translate into multicore execution policy
- [x] document what stays deterministic versus what may vary by runtime profile
- [x] freeze the runtime-owned receipts needed to inspect scheduler choices

### Batch 2.2 - Runtime Scheduling Depth

- [x] deepen multicore execution and anticipative preparation in
  `signal-runtime`
- [x] preserve correct invalidation, transport, and degradation behavior while
  the work partitioning grows more dynamic
- [x] keep realtime-safe boundaries explicit

### Batch 2.3 - Focused Stress Proofs

- [x] add focused tests or fixtures for mixed execution-class graphs,
  invalidation-heavy transitions, and constrained anticipative windows
- [x] record residual performance risks that belong to later regression work

## Progress Notes

- 2026-03-12: completed Batch 2.1 by freezing the first runtime-owned multicore
  scheduling contract around `RuntimeEngineBlockSnapshot`,
  `RuntimeSchedulerSnapshot`, `RuntimeSchedulerExportSummary`, and
  `RuntimeExecutionTopologySummary`, explicitly separating deterministic
  planning/lane/topology rules from runtime-profile and degraded-state variance
  such as prework pressure, schedule-stream width, and transport/plugin gating.
- 2026-03-12: completed the first partial Batch 2.2 runtime-depth tranche by
  making compatible `ScheduleProjection.stream_count` widen anticipative
  prework service budget in `signal-runtime` instead of remaining export-only
  metadata, while proving that missing/incompatible schedules stay at the base
  budget and that the existing normal-pressure scheduler path remains stable.
- 2026-03-12: completed the second partial Batch 2.2 runtime-depth tranche by
  making compatible `ScheduleProjection.stream_count` widen requested
  anticipative prework service cadence at the realtime service call sites,
  while proving elevated pressure still clamps widened cycle requests back to
  the existing bounded scope through runtime-owned scheduler receipts.
- 2026-03-12: completed the third partial Batch 2.2 runtime-depth tranche by
  making schedule projection and running forecast-plan refresh/rebuild paths
  reuse the same widened runtime-owned prework service scope, while proving
  widened requests still yield cleanly under plugin and transport gates instead
  of introducing a parallel host-local refresh model.
- 2026-03-12: closed Batch 2.2 with a compact acceptance proof that the same
  schedule-width policy now survives restart, reconfigure, and mixed
  execution-class graph transitions while keeping runtime-owned scheduler
  receipts coherent through those lifecycle changes.
- 2026-03-12: completed Batch 2.3 and closed `g04.002` by adding focused stress
  proofs for mixed execution-class graph churn, invalidation-heavy transition
  bursts, and constrained anticipative windows, then recording the remaining
  deferred performance risks for later regression work instead of leaving them
  implicit.

## Acceptance Criteria

- [ ] Signal has a clearer reusable multicore scheduling policy
- [ ] anticipative execution remains runtime-owned and inspectable
- [ ] later backend and plugin work can rely on explicit execution semantics

## Risks and Mitigations

- Risk: scheduling policy becomes too host-specific.
- Mitigation: keep policy receipts in runtime-owned exports and docs.
- Risk: multicore work weakens determinism around invalidation and recovery.
- Mitigation: require focused transition and degradation proofs in the same milestone.

## Deferred Performance Risks

- Cost-aware or work-stealing multicore dispatch is still not implemented; the
  current policy uses schedule-stream width as a bounded planning/service proxy
  rather than a true measured load balancer.
- There is still no long-duration threshold or benchmark fail-gate for this
  scheduler policy; those belong to later regression infrastructure rather than
  this milestone.
- The current stress fixtures validate receipt coherence and bounded behavior,
  not absolute throughput on high-core or memory-contention hardware.

## Evidence Requirements

- [ ] log each meaningful scheduling tranche
- [ ] run focused runtime validation for multicore or anticipative behavior
- [ ] record deferred edge cases rather than silently relying on benchmarks

## Next Task

Open `g04.003` with Batch 3.1 and define the runtime-owned deferred-work
contract for render finalization, analysis jobs, delegated merge work, and
report/materialization services on top of the now-closed scheduling substrate.
