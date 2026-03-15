# 019 Deferred-Work Scheduler Priority, Backpressure, And Cancellation Contract

Status: active
Owner: core-product
Updated: 2026-03-14
Related contracts: `docs/contracts/005-runtime-work-orchestration-and-deferred-service-policy.md`, `docs/contracts/017-per-block-execution-timing-and-pressure-snapshot-contract.md`, `docs/contracts/018-graph-critical-path-hot-node-and-worker-lane-instrumentation-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first bounded scheduler-policy contract for deferred work so later
`g06.008` runtime depth can expose priority, starvation, backpressure, and
cancellation through one Signal-owned orchestration seam instead of ad hoc host
queues or product-local pacing heuristics.

## Authority hierarchy

Deferred-work scheduling has one authority chain:

1. `signal-runtime` owns canonical deferred-work class, admission, priority,
   starvation, and cancellation meaning
2. runtime-owned orchestration and timing receipts explain why work ran,
   throttled, deferred, starved, or cancelled:
   - `RuntimeDeferredServiceReceipt`
   - `RuntimeTransportConcurrencySnapshot`
   - offline render queue, progress, purge, and continuity receipts
   - `RuntimeEngineBlockSnapshot`
   - `RuntimePerformanceSnapshot`
   - `RuntimePerformanceTraceReceipt`
3. supervisor and stable host-edge surfaces may expose that meaning, but they
   must not reclassify it
4. hosts and tools may add advisory queue or environment detail, but they must
   not become the authority for scheduler priority or cancellation policy

If a deferred-work priority or cancellation conclusion cannot be explained
through Signal-owned receipts, it is not yet part of the shared contract.

## Shared terms

This contract freezes eight shared terms.

### Deferred-work class

A deferred-work class is a runtime-owned category of non-realtime work with
shared scheduling and cancellation expectations. The current bounded class
families are:

- render advancement and queue progression
- artifact or report materialization and purge
- delegated outcome merge or cleanup
- analysis, profiling, and export derivation
- lingering transport cleanup and retry waves

### Priority band

Priority band is the bounded runtime-owned answer to which deferred class should
win when multiple compatible work items could run.

The first shared bands are:

- `CriticalRecovery`: cleanup or merge work required to preserve coherent
  runtime recovery state
- `UserBlockingFinalization`: render or materialization work that already has
  runtime-owned inputs and is waiting for bounded completion
- `Maintenance`: cleanup, purge, or bounded reconciliation work that should
  make progress when pressure allows
- `AdvisoryAnalysis`: profiling, export, or inspection work that is useful but
  should yield first under pressure

### Backpressure

Backpressure is the runtime-owned signal that deferred work volume, age, or
interaction with current engine pressure should reduce admitted scope or delay
new work.

This is stronger than one `Defer` decision but weaker than a hard terminal
fault.

### Starvation

Starvation means a deferred-work class remains eligible but is unable to make
meaningful progress across bounded service opportunities because higher-priority
or safer work keeps consuming the available budget.

The contract freezes starvation as a runtime-owned condition that later receipts
may count or classify. It is not a host-side wall-clock heuristic.

### Cancellation

Cancellation is the runtime-owned decision that previously admitted or queued
deferred work must stop and report a typed outcome because its authority data,
runtime preconditions, or requested result are no longer valid.

Cancellation is distinct from:

- `Defer`: work remains valid and should wait
- `Throttle`: work remains valid but must progress in bounded scope
- `Abort`: work cannot continue because the specific operation failed

### Cancellation cause

Cancellation cause is the bounded reason family explaining why runtime stopped
work intentionally. The first shared causes are:

- authority changed
- runtime interruption or restart invalidated the work
- newer work superseded the request
- backpressure or starvation policy explicitly evicted lower-priority work
- resource or delivery target removal made the result meaningless

### Budget-compatible progress

Budget-compatible progress is the runtime-owned rule that deferred work may only
consume service scope that remains coherent with per-block timing, hotspot, and
pressure receipts.

This ties later orchestration depth directly to contracts `017` and `018`.

### Advisory host queue context

Advisory host queue context is any app-local task-runner, filesystem polling,
or wall-clock progress view that may help UX but does not outrank runtime-owned
priority, starvation, or cancellation meaning.

## Policy rules

This contract freezes seven shared rules.

### Rule 1: priority stays runtime-owned

Priority bands must derive from runtime-owned work classes and continuity
requirements, not from host-local thread pools or product task labels.

### Rule 2: timing and hotspot pressure constrain deferred work

Deferred-work scheduling must compose with the per-block timing and bounded
hotspot contracts:

- high deadline pressure may force `Defer` or stronger throttling
- hotspot or critical-path pressure may narrow budget-compatible progress
- healthy timing posture may permit broader low-priority work

### Rule 3: starvation must be observable, not inferred

If one class repeatedly yields to higher-priority work, later runtime receipts
must expose that starvation explicitly instead of forcing products to infer it
from missing output or delayed files.

### Rule 4: cancellation stays distinct from failure

Later runtime receipts may report both cancellation and failure, but
intentional cancellation must remain distinguishable from execution failure,
delivery error, or terminal runtime fault.

### Rule 5: bounded progress outranks throughput maximization

The scheduler policy may prefer predictable bounded progress and coherent
receipt meaning over maximizing total deferred throughput. This milestone does
not freeze a generic work-stealing or throughput-optimizing background engine.

### Rule 6: stable consumers observe policy through receipts

Stable runtime, supervisor, and host-edge consumers must read scheduling
policy through typed runtime-owned receipts rather than private task queues,
callback timing, or filesystem side effects.

### Rule 7: distributed or remote orchestration remains deferred

This contract freezes one local Signal-owned scheduler-policy seam only. Remote
queue ownership, distributed workers, and cross-process orchestration policy
remain outside `g06.008`.

## Current runtime mapping

The repo already contains the bounded runtime surfaces this contract builds on:

- `RuntimeDeferredServiceReceipt` carries the current decision vocabulary and
  class or reason seam for deferred work
- `RuntimeTransportConcurrencySnapshot` carries pending cleanup and retry-wave
  state that later starvation or priority receipts must compose with
- offline render queue, progress, purge, and continuity receipts already prove
  one real deferred-work family with resumable, restartable, and terminal
  outcomes
- `RuntimeEngineBlockSnapshot`, `RuntimePerformanceSnapshot`, and
  `RuntimePerformanceTraceReceipt` already carry the timing and hotspot pressure
  context that later deferred-work scheduling must respect

Batch 8.1 freezes how these surfaces should align before Batch 8.2 adds richer
priority, starvation, backpressure, and cancellation receipts.

## Explicitly deferred

Batch 8.1 does not yet freeze:

- a generic runtime job scheduler API
- distributed or remote queue ownership
- host-specific worker-pool adapters
- end-user task-manager UX
- cost-aware fairness across every future deferred-work class
- publication or storage-lifecycle policy beyond current runtime-owned receipts

## Batch 8.1 outcome

Batch 8.1 freezes the bounded scheduler-policy seam for deferred work:

- deferred-work classes remain runtime-owned
- priority, starvation, backpressure, and cancellation are now shared terms
- timing and hotspot pressure are explicitly part of the scheduler-policy
  authority chain
- hosts and products may observe or enrich the policy, but must not redefine it

## Batch 8.2 outcome

Batch 8.2 deepens the contract into real runtime-owned receipts:

- `RuntimeDeferredServiceReceipt` now carries typed priority-band,
  blocking-priority, backpressure-source, starvation, and cancellation fields
- invalid-request aborts, throttled queue advancement, and safe-mode or
  recovery deferrals all derive those fields from one shared runtime helper
  instead of per-call-site reconstruction
- `RuntimePerformanceSnapshot` now preserves the latest deferred-work policy
  state alongside the existing timing and hotspot digest
- `RuntimePerformanceTraceReceipt` now preserves starvation, cancellation, and
  backpressure evidence across an observation window so later soak or
  profiling lanes can cite bounded scheduler-policy outcomes without log
  scraping

This keeps the richer scheduler-policy seam inside `signal-runtime` while
still deferring the consumer-facing proof and acceptance boundary to Batch 8.3.

## Batch 8.3 outcome

Batch 8.3 closes the shared consumer boundary around the widened scheduler
policy seam:

- public runtime proofs now show defer, abort, starvation, backpressure,
  cancellation, and bounded trace evidence through public reexports alone
- stable local and server host-edge proofs show `supervisor_report()` forwards
  the same runtime-owned policy truth without private queue helpers or
  host-local reclassification
- `signal-supervisor-tools` now exposes a machine-readable
  `signal.runtime.deferred-work-policy-boundary` descriptor, and the repo-owned
  `effigy acceptance:deferred-work-policy-boundary` task keeps the boundary
  runnable instead of prose-only

This closes `g06.008` on one bounded local runtime-owned scheduler-policy seam
while still deferring any generic future job scheduler or distributed
orchestration model.

## Next Task

Continue `g06.009` with Batch 9.1 by mapping VST3-specific details onto the
existing backend-neutral capability and lifecycle contract before runtime
realization widens.
