# 017 Per-Block Execution Timing And Pressure Snapshot Contract

Status: active
Owner: core-product
Updated: 2026-03-14
Related contracts: `docs/contracts/004-runtime-multicore-scheduling-and-anticipative-execution-contract.md`, `docs/contracts/005-runtime-work-orchestration-and-deferred-service-policy.md`, `docs/contracts/011-shared-downstream-conformance-and-release-acceptance-automation-contract.md`, `docs/contracts/012-runtime-interruption-taxonomy-and-resumability-contract.md`, `docs/contracts/016-runtime-fault-cause-attribution-and-diagnostic-receipt-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first shared per-block execution timing and pressure snapshot
contract for `g06.006` so later scheduler instrumentation, hot-node analysis,
deferred-work tuning, and soak acceptance all build on one bounded
runtime-owned measurement seam instead of mixed counters, ad hoc traces, or
host-local timing stories.

## Authority hierarchy

Per-block timing and pressure have one authority chain:

1. `signal-runtime` owns the canonical execution measurement meaning:
   - the currently processed block boundary
   - the runtime block sequence and processing epoch
   - bounded timing and deadline-pressure observations
   - budget-overrun posture for the current or most recent block
   - scheduler and prework pressure context that explains the measurement
2. supervisor and stable host-edge surfaces may expose that meaning, but they
   must not reinterpret it:
   - `RuntimeEngineBlockSnapshot`
   - `RuntimeSchedulerSnapshot`
   - `RuntimePerformanceSnapshot`
   - `RuntimePerformanceTraceReceipt`
   - `RuntimeObservationReport`
   - `RuntimeSupervisorReport`
   - stable host-edge `supervisor_report()` surfaces
3. host and backend crates may contribute adjacent callback cadence, backend
   xrun, and device timing detail, but they must not become the authority for:
   - the canonical per-block timing boundary
   - deadline pressure classification
   - whether one block should be treated as over-budget or merely adjacent to
     host timing noise

If a timing or pressure conclusion cannot be explained through Signal-owned
snapshots or receipts, it is not yet part of the shared measurement contract.

## Shared terms

This contract freezes seven shared terms.

### Block timing snapshot

A block timing snapshot is the runtime-owned measurement boundary for one
processed block or the most recent block completed by runtime.

It must stay anchored to the same runtime block identity already surfaced
through:

- `processed_blocks`
- `last_processing_epoch`
- `last_block_sequence`
- `last_frame_count`
- `last_channel_count`

This contract does not require an unbounded historical trace. It freezes the
meaning of one bounded block snapshot first.

### Deadline budget

Deadline budget is the runtime-owned execution allowance implied by the current
engine configuration for one block boundary.

At minimum it is shaped by:

- effective sample rate
- configured block size
- realtime-versus-background execution context
- scheduler or prework policy that changes what work is attempted inside that
  block

Hosts may expose callback interval detail, but the canonical per-block budget
must stay explainable from runtime-owned context first.

### Budget overrun

Budget overrun means runtime-owned work for one block boundary exceeded the
shared deadline budget or otherwise consumed enough of it that later pressure or
degradation interpretation should treat that block as over-budget.

This is stronger than raw CPU load and narrower than a long-running profiler. It
must stay explainable through one bounded block observation.

### Deadline pressure

Deadline pressure is the reusable shared classification that says whether recent
per-block execution posture is comfortably inside budget, approaching budget, or
materially exceeding it.

This contract freezes deadline pressure as a runtime-owned classification, not a
host callback taxonomy.

### Timing evidence

Timing evidence is the bounded set of runtime-owned fields that explain why one
block is considered healthy, pressured, or over-budget.

The first shared timing evidence families are:

- block identity and size
- measured or derived execution duration
- derived deadline budget
- over-budget or near-budget posture
- scheduler and prework pressure context active for that block

### Bounded measurement

Bounded measurement means the shared contract exposes only the fields needed for
runtime, supervisor, acceptance, and downstream consumers to reason about
pressure without requiring a full trace buffer, always-on profiler, or
product-local log scraping.

This milestone is intentionally snapshot-first.

### Advisory host timing evidence

Advisory host timing evidence is host callback interval, callback overrun, or
backend cadence detail that may sharpen diagnosis but does not outrank
runtime-owned per-block timing meaning.

This mirrors the cause-vs-advisory split already frozen in contract `016`.

## Measurement rules

This contract freezes seven shared rules.

### Rule 1: per-block timing stays downstream of runtime block identity

Per-block timing must remain attached to runtime block sequence and processing
epoch instead of a host-only callback frame counter or export-only digest.

### Rule 2: runtime timing meaning outranks callback anecdotes

Host callback interval and callback overrun evidence may sharpen diagnosis, but
they must not replace the runtime-owned answer to whether one block was
in-budget, near-budget, or over-budget.

### Rule 3: bounded snapshots precede full tracing

`g06.006` starts with bounded high-value measurements only. Full history,
always-on sampling, or per-node flame-style tracing remain later work.

### Rule 4: pressure classification must compose with existing runtime posture

Per-block timing observations must align with existing runtime-owned posture
surfaces such as:

- `RuntimePreworkServicePressure`
- `RuntimeDeferredServiceReceipt`
- `RuntimeFaultStatusSnapshot`
- `RuntimeFaultDiagnosticReceipt`

Timing must extend that story, not fork it.

### Rule 5: runtime-facing and export-facing surfaces have different depth

`RuntimeEngineBlockSnapshot` is the authoritative per-block surface and may
carry the richest bounded timing detail.

`RuntimePerformanceSnapshot` and `RuntimePerformanceTraceReceipt` are the
consumer-facing digests and should stay narrower:

- enough to reason about pressure and budget posture
- not so wide that products depend on unstable internal counters

### Rule 6: timing contract does not freeze hot-node attribution yet

Per-node, critical-path, worker-lane, or dispatch-hotspot timing remains later
`g06.007` work. Batch 6.1 only freezes the per-block seam those later receipts
must compose with.

### Rule 7: acceptance and soak use the same bounded measurement seam

Later acceptance harnesses, downstream automation, and soak gates must consume
the same runtime-owned timing fields rather than inventing a second benchmark
schema.

## Current runtime mapping

The repo already contains the runtime-owned surfaces this contract is intended
to stabilize.

### RuntimeEngineBlockSnapshot

`RuntimeEngineBlockSnapshot` is already the per-block execution authority for:

- graph shape and scheduler topology
- prework service pressure and budget realization
- processed block count, block sequence, and processing epoch
- per-block output telemetry
- transport position at the processed block boundary

This milestone freezes that type as the future home for per-block timing truth.

### RuntimeSchedulerSnapshot

`RuntimeSchedulerSnapshot` remains the lifecycle and control-state companion
surface. It explains how runtime was configured around the measured block but is
not itself the per-block timing authority.

### RuntimePerformanceSnapshot

`RuntimePerformanceSnapshot` already exposes the current bounded consumer digest
for:

- sample rate and block size
- processed block count
- CPU load and graph latency
- xrun count
- scheduler width and dispatch shape
- prework service pressure, queue depth, and budget realization
- deferred-work service posture

This milestone freezes it as the narrow shared timing-and-pressure summary that
later products and automation should consume instead of mining internal
instrumentation directly.

### RuntimePerformanceTraceReceipt

`RuntimePerformanceTraceReceipt` already provides bounded multi-report rollup for
shared automation and profiling consumers. Later timing delta or overrun rollup
belongs here rather than in host-local benchmark scripts.

## Consumer promises

This contract keeps four promises.

### Products observe one bounded timing seam

Consumers should not need private runtime hooks or log parsing to understand
whether runtime is operating within block budget.

### Runtime remains the measurement authority

Timing and pressure semantics remain Signal-owned even when host or backend
cadence detail is available.

### Later profiling work can widen fields without changing meaning

Batch 6.2 and `g06.007` may add timing fields and richer instrumentation, but
they must preserve the meanings frozen here.

### Acceptance depth can start from typed measurements

Downstream automation and later soak gates can promote timing evidence using the
same bounded snapshot family instead of inventing release-only measurement
formats.

## Explicitly deferred

This Batch 6.1 contract does not yet freeze:

- full tracing or history buffers
- per-node or per-dispatch elapsed-time attribution
- worker-lane occupancy or multicore cost modeling
- publication of host callback cadence as canonical runtime timing authority
- fleet or remote telemetry aggregation

Those belong to Batch 6.2, `g06.007`, or later acceptance work.

## Batch 6.1 outcome

Batch 6.1 freezes the meaning first:

- `RuntimeEngineBlockSnapshot` is the authoritative per-block measurement seam
- `RuntimeSchedulerSnapshot` stays the control-state companion
- `RuntimePerformanceSnapshot` and `RuntimePerformanceTraceReceipt` stay the
  bounded consumer and automation digests
- host callback timing remains additive evidence rather than a competing timing
  authority

## Next Task

Continue `g06.006` with Batch 6.2 by instrumenting bounded per-block execution
timing, deadline pressure, and budget-overrun fields on the frozen runtime-owned
measurement seam, then align supervisor and host-edge export to the same
observations without widening into full tracing yet.
