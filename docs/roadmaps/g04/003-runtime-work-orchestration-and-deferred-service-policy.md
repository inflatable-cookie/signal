# 003 - Runtime Work Orchestration And Deferred Service Policy

Status: complete
Owner: core-product
Created: 2026-03-12
Depends on: g04.001, g04.002
Vision tags: `RUNTIME`, `ORCHESTRATION`, `SERVICES`

## Problem

Signal now owns more than realtime graph execution: offline render finalization,
analysis-heavy paths, delegated execution boundaries, and supervisor/report
materialization all exist. The repo still lacks one explicit runtime-owned
policy for how deferred or non-realtime work is admitted, paused, throttled, or
resumed relative to playback, capture, and recovery.

Without a dedicated orchestration milestone:

- hosts will keep filling the policy gap with local task heuristics
- background work can interfere with scheduling and recovery in unclear ways
- later portability and release work will lack one Signal-owned service model
- deferred work will remain observable after the fact instead of planned

## Goals

- [ ] define a Signal-owned policy for deferred and non-realtime runtime work
- [ ] keep hosts as observers or requesters rather than schedulers
- [ ] make defer/throttle/resume semantics explicit and inspectable
- [ ] align deferred work with recovery and multicore execution policy

## Non-Goals

- [ ] no product UX queue or task-manager work
- [ ] no distributed fleet scheduler
- [ ] no broad workflow orchestration outside Signal-owned services

## Execution Plan

### Batch 3.1 - Deferred Work Contract

- [x] define the work classes Signal needs to reason about:
  render finalization, analysis jobs, delegated merge work, report/materialization,
  and other deferred services
- [x] classify when those classes may run, must defer, or must abort
- [x] decide which orchestration receipts belong in public exports

### Batch 3.2 - Runtime Orchestration Baseline

- [x] implement the first reusable orchestration policy in Signal-owned crates
- [x] connect defer/throttle/resume behavior to runtime state instead of
  implicit host timing
- [x] keep the audio-thread boundary free from blocking or heavy aggregation

### Batch 3.3 - Validation And Consumer Proof

- [x] prove at least one deferred service path behaves coherently under
  playback/capture pressure and recovery transitions
- [x] expose enough receipts for consumers to observe the policy without
  rebuilding it locally

## Progress Notes

- 2026-03-12: completed Batch 3.1 by freezing the first runtime-owned deferred
  work contract around offline render queue/purge/materialization receipts,
  delegated offline merge boundaries, transport cleanup queue state, and
  profiling/soak export, while making `Run`, `Defer`, `Throttle`, and `Abort`
  the shared policy vocabulary for later orchestration work.
- 2026-03-12: completed Batch 3.2 by making the offline render queue the first
  runtime-owned orchestration baseline: `signal-runtime` now emits typed
  deferred-service receipts for `Run`, `Throttle`, and `Defer`, throttles
  queue progress while the runtime is live, and preserves deferred requests
  across safe-mode and recovery-sensitive states without widening the
  audio-thread boundary.
- 2026-03-12: completed Batch 3.3 and closed `g04.003` by extending the same
  typed deferred-service receipt surface to offline render purge, carrying the
  latest deferred-service decision through observation/supervisor export, and
  proving the consumer-facing export path through `signal-supervisor-tools`
  without private runtime inspection.

## Acceptance Criteria

- [ ] Signal owns an explicit deferred-work policy
- [ ] hosts no longer need to invent background-service scheduling
- [ ] later portability and packaging work can rely on a stable service model

## Risks and Mitigations

- Risk: orchestration scope expands into product workflow management.
- Mitigation: keep the milestone to Signal-owned runtime services only.
- Risk: deferred-work policy becomes too abstract to validate.
- Mitigation: require at least one real service path and one recovery interaction proof.

## Evidence Requirements

- [ ] log each meaningful orchestration tranche
- [ ] run focused validation for defer/throttle/resume behavior
- [ ] record explicit deferred service classes that remain outside the first policy

## Next Task

Continue `g04.004` with Batch 4.2 and implement stronger clock-domain and
fallback handling in Signal-owned runtime and hardware crates on top of the
now-closed scheduling and deferred-work substrate.
