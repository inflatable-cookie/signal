# 005 Runtime Work Orchestration And Deferred Service Policy

Status: active
Owner: core-product
Updated: 2026-03-12
Related contracts: `docs/contracts/002-supervisor-export-schema-and-report-boundary.md`, `docs/contracts/003-crate-maturity-and-public-runtime-boundary-baseline.md`, `docs/contracts/004-runtime-multicore-scheduling-and-anticipative-execution-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first explicit Signal-owned contract for deferred and non-realtime
runtime work so later `g04` orchestration changes can deepen service behavior
without pushing admission, defer, or abort policy into host-local heuristics.

## Work classes

The first orchestration contract treats deferred work as runtime-owned classes,
not as an undifferentiated host queue.

### Finalization and materialization

These classes package or persist work that was already planned or rendered by
runtime-owned execution surfaces:

- offline render queue execution and per-request progress receipts
  (`RuntimeOfflineRenderQueueResult`,
  `RuntimeOfflineRenderQueueProgressReceipt`,
  `RuntimeDeferredServiceReceipt`)
- offline artifact/report materialization and purge
  (`RuntimeOfflineRenderManifest`,
  `RuntimeOfflineRenderReportReceipt`,
  `RuntimeOfflineRenderArtifactReceipt`,
  `RuntimeOfflineRenderPurgeReceipt`)
- delegated offline outcome merge and delivery rewrite
  (`RuntimeOfflinePluginExecutionBoundary`,
  delegated execution request/result/outcome receipts, and manifest updates)

These classes are mutable in volume and latency cost, but they must remain
derived from runtime-owned render and manifest state rather than from ad hoc
host packaging queues.

### Analysis and inspection

These classes summarize or inspect runtime state without changing the live
engine graph:

- runtime profiling and soak receipt derivation
  (`RuntimeProfilingReceipt`, `RuntimeSoakReceipt`)
- supervisor and observation report materialization
  (`RuntimeObservationReport`, `RuntimeSupervisorReport`,
  `RuntimeHostObservationReport`)
- report and export formatting layered on those typed receipts

These classes are allowed to be deferred or throttled, but the authority for
their content remains the typed runtime-owned snapshots and receipts.

### Recovery-adjacent service work

These classes coordinate cleanup or merge work that may span multiple runtime
epochs and interact with recovery:

- lingering transport cleanup queue planning and deferred retry work
  (`LingeringCleanupPlan`, `LingeringCleanupQueueReceipt`,
  `RuntimeTransportConcurrencySnapshot`)
- delegated offline plugin execution handoff and later outcome merge
- future runtime-owned background reconciliation services that derive from the
  same transport, render, or supervisor receipts

These classes are orchestration-sensitive because they may need to pause,
throttle, or abort under recovery overlap, transport gating, or degraded
runtime state.

## Authority hierarchy

Deferred-work policy has one authority chain:

1. typed runtime state and result surfaces remain authoritative for what work
   exists and what it is allowed to operate on
2. orchestration receipts describe admission, defer, throttle, pause, resume,
   or abort outcomes for that work
3. supervisor/export envelopes deliver those typed receipts to consumers and
   automation
4. hosts may request or observe deferred work, but they must not become the
   authority for work-class semantics, queue state, or completion meaning when
   runtime-owned receipts already expose that information

If later consumers need more orchestration detail, that detail must be
promoted into `signal-runtime` receipts rather than inferred from callback
timing, filesystem polling, or host task runners.

## Admission classes and policy outcomes

The first policy baseline uses a small shared vocabulary across deferred-work
classes.

### Allowed outcomes

- `Run`: the work may execute immediately because the runtime state is
  compatible with that class
- `Defer`: the work stays queued or pending because runtime state allows the
  work eventually but not right now
- `Throttle`: the work may continue, but only in a reduced or bounded form so
  it does not compete with realtime or recovery obligations
- `Abort`: the work must stop and report failure because runtime state or input
  validity makes continuation unsafe or meaningless

### Run by default

The following classes may run when the runtime is configured, not in a
conflicting recovery mode, and the work does not require missing inputs:

- report materialization and export formatting from already-captured
  observation/supervisor state
- offline render queue jobs that already have valid requests and runtime-owned
  inputs
- purge operations for explicit runtime-owned artifact/report paths
- delegated outcome merge/finalization after a valid runtime-owned boundary and
  matching delegated outcome exist

### Prefer defer under live pressure

The following classes should defer instead of competing with time-critical
runtime activity:

- analysis-heavy report or profiling derivation during elevated realtime
  pressure
- offline queue advancement when recovery, restart, or transport-fault cleanup
  is the higher-priority obligation
- lingering cleanup retries before their declared ready epoch or while stricter
  recovery-pre-attach policy is active
- future background summarization services that can tolerate stale-but-coherent
  data instead of forcing immediate recomputation

### Prefer throttle over unbounded catch-up

These classes may continue in bounded form when runtime state is compatible but
the workload should not expand without limit:

- multi-request offline queue progression
- cleanup retry waves that may span many lingering sessions
- receipt/export materialization that can be chunked or delayed without losing
  correctness

The contract does not yet freeze one generic scheduler implementation, but it
does freeze the rule that bounded progress must remain runtime-owned and
inspectable rather than delegated to host-local pacing heuristics.

### Must abort

Deferred work must abort when:

- the request is invalid or missing authority data owned by runtime
- a delegated outcome does not match the runtime-owned request or boundary it
  claims to satisfy
- recovery or degraded state invalidates the work's preconditions rather than
  merely delaying them
- filesystem or resource failure makes finalization or purge semantically
  incomplete and the class cannot safely continue in partial form

Abort conditions should produce typed runtime errors or receipts rather than
silent host-side drops.

## Canonical inspection surfaces

Consumers should inspect deferred-work state in this order:

- use `RuntimeTransportConcurrencySnapshot` for lingering cleanup queue depth,
  deferred retry visibility, pending waves, and recovery-sensitive transport
  service state
- use offline render queue, manifest, report, artifact, purge, and delegated
  execution receipts when the question is about render/materialization work
- use `RuntimeObservationReport` and `RuntimeSupervisorReport` when the
  question is whether current runtime state permits or explains deferred-work
  progression
- use `RuntimeProfilingReceipt` and `RuntimeSoakReceipt` when the question is
  performance/fault inspection rather than service authority

Hosts and tools may aggregate these surfaces for UX, but they must not invent a
parallel deferred-work model from filesystem side effects, callback cadence, or
private runtime internals.

## Public export boundary

The first public orchestration boundary is intentionally narrow:

- typed runtime-owned queue, manifest, report, purge, profiling, soak, and
  transport-concurrency receipts remain the Rust authority surface, including
  `RuntimeDeferredServiceReceipt` plus its typed class/decision/reason enums
- `RuntimeObservationReport` and `RuntimeSupervisorReport` remain the shared
  export envelopes that deliver orchestration-relevant state to consumers
- new public orchestration receipts may be added additively, but existing
  meanings should not be silently repurposed into host-local scheduling hints

This contract does not yet freeze a generic job-runner API or distributed task
protocol. It only freezes the Signal-owned meaning of deferred-work classes and
the receipt families that must remain authoritative.

## Deferred from the first orchestration freeze

The following remain outside Batch 3.1:

- a reusable runtime job scheduler or queue executor API
- distributed/cloud orchestration or fleet policy
- product-facing task-manager UX or prioritization features
- cost-aware fairness rules across all deferred-work classes
- retention, storage lifecycle, or cross-process artifact caching policy beyond
  the current runtime-owned manifest and purge receipts

## Current proof boundary

This contract is grounded in runtime-owned surfaces that already exist today:

- offline render queue and purge receipts provide the first explicit
  multi-request and cleanup-oriented deferred-service DTOs
- the offline render queue now emits a typed `RuntimeDeferredServiceReceipt`
  that classifies the queue as `Run`, `Throttle`, or `Defer` according to live
  runtime state instead of leaving queue cadence to host-local heuristics
- offline render purge now emits the same typed receipt surface and can defer
  cleanly under safe mode or recovery-sensitive state instead of forcing hosts
  to guess when artifact cleanup is allowed
- the current runtime-owned queue policy is intentionally bounded:
  - healthy, non-running runtime executes the full queue
  - running runtime throttles queue execution to bounded progress
  - safe mode, degraded recovery, and pending cleanup state defer queue
    execution without dropping requests
- `RuntimeObservationReport` and `RuntimeSupervisorReport` now carry the latest
  deferred-service receipt so consumers can inspect the last orchestration
  decision through the shared export path rather than private runtime access
- delegated offline execution boundary, request/result, and outcome merge
  receipts prove that later host work still folds back into runtime-owned
  delivery state
- transport concurrency snapshots already expose pending cleanup waves and
  deferred retry visibility instead of leaving lingering cleanup state entirely
  implicit
- profiling and soak receipts already prove that heavy inspection/export work
  can stay typed and runtime-owned rather than being reconstructed in
  host-local benchmark schemas

Later `g04.003` work may add explicit orchestration snapshots or queue
receipts, but it should extend this authority model rather than replacing it.

## Next Task

Continue `g04.004` with Batch 4.2 and implement stronger clock-domain and
fallback handling in Signal-owned runtime and hardware crates on top of the
closed scheduler and deferred-work substrate.
