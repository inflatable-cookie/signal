# 012 Runtime Interruption Taxonomy And Resumability Contract

Status: active
Owner: core-product
Updated: 2026-03-13
Related contracts: `docs/contracts/002-supervisor-export-schema-and-report-boundary.md`, `docs/contracts/005-runtime-work-orchestration-and-deferred-service-policy.md`, `docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md`, `docs/contracts/011-shared-downstream-conformance-and-release-acceptance-automation-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first shared interruption and resumability contract for `g06.001`
so later recording, plugin, render, hardware, and soak work can all build on
one runtime-owned vocabulary instead of host-local interpretation.

## Authority hierarchy

Runtime interruption has one authority chain:

1. `signal-runtime` owns the canonical interruption and recovery meaning:
   - runtime readiness and failure state
   - fault, degradation, restart, and recovery snapshots
   - deferred-work pause/defer/throttle receipts
   - offline render execution pause/resume/interrupt progress
2. supervisor/export surfaces and stable host edges may expose that meaning,
   but they must not reinterpret it:
   - `RuntimeSupervisorReport`
   - `RuntimeObservationReport`
   - `RuntimeFaultStatusSnapshot`
   - `RuntimeDegradationSummary`
   - `RuntimeSoakReceipt`
   - `RuntimeSupervisorApi::supervisor_report()`
3. products may react to interruption state, but they must not become the
   authority for classifying whether runtime work is resumable, restartable,
   recoverable, or terminal

If interruption meaning cannot be explained through runtime-owned snapshots,
receipts, or export surfaces, it is not yet part of the shared contract.

## Interruption taxonomy

This contract freezes five shared terms.

### Interruption

An interruption is any runtime-owned loss of uninterrupted progress for
playback, capture, plugin transport, deferred work, or offline render
execution.

Interruptions may be caused by faults, policy gates, pause requests, device
loss, transport detach, safe mode, or explicit runtime-owned execution control.

### Resumable

`Resumable` means the same runtime-owned work identity may continue after the
interruption without allocating a new authoritative consumer boundary.

Current examples already present in runtime:

- paused or interrupted offline render execution resumed through the same
  request identity
- deferred offline render queue work that advances after safe mode clears
- bounded deferred work that yields or pauses while preserving the same queued
  request identity

### Restartable

`Restartable` means the same shared runtime boundary may survive, but runtime
must re-establish internal execution state before steady progress resumes.

Current examples already present in runtime:

- watchdog-driven restart sequences
- plugin sandbox restart cycles
- transport attach/detach recovery that preserves the runtime-owned consumer
  surface while rebuilding execution continuity
- future device-supervision restarts that keep the runtime authority intact

Restartable does not guarantee same-request progress continuity. It guarantees
that runtime, not the host, owns the repair path.

### Recoverable

`Recoverable` means runtime is currently inside an active repair or degraded
path, but the shared boundary remains authoritative and may return to steady
operation without a new product-owned taxonomy.

In current runtime terms this is the broad class represented by
`RuntimeRecoveryState::Recovering`.

### Terminal

`Terminal` means the current runtime-owned boundary cannot safely continue as
the same authoritative execution surface and must be treated as failed.

In current runtime terms this is the broad class represented by
`RuntimeRecoveryState::Faulted`.

### Rebindable

`Rebindable` is a property of some resumable or restartable interruptions where
transport, device, or plugin attachment may be re-established without changing
the authoritative runtime-owned boundary.

Rebindable is not a competing top-level recovery class. It describes how
runtime may repair a boundary that is already classified as resumable or
restartable.

## Current runtime mapping

The first shared contract maps directly onto current runtime surfaces.

### Broad fault and recovery state

`RuntimeFaultStatusSnapshot` is the current broad classification seam:

- `Steady` means no active interruption class needs promotion
- `Recovering` means runtime is inside a recoverable interruption path
- `Faulted` means the current runtime boundary is terminal

This snapshot also owns the current primary-fault and active-fault context
needed to keep interruption semantics concrete rather than narrative-only.

### Degradation and active repair context

`RuntimeDegradationSummary` owns the adjacent evidence around interruption:

- safe mode
- plugin faults
- transport fault events
- recovery event count
- recovery-overlap sessions
- plugin and transport gating
- last watchdog trigger

This is the runtime-owned context for understanding whether an interruption is
active and what kind of repair pressure exists around it.

### Recovery history

`RecoveryRecord` and `RuntimeSoakReceipt` own the current shared history of
restart and recovery activity:

- restart count
- watchdog restart count
- recovery event count
- peak recovery-overlap sessions
- last recovery intent
- last stop reason

Products may inspect that history, but they must not invent a parallel recovery
ledger to decide what happened.

### Deferred and offline work continuity

`RuntimeDeferredServiceReceipt`, `RuntimeOfflineRenderQueueResult`, and the
offline render execution progress APIs are the current continuity seam for
non-realtime work:

- defer, throttle, pause, resume, and abort are runtime-owned orchestration
  decisions
- offline render execution may pause, resume, interrupt, advance, or cancel
  under the same request identity
- later milestones should map those controls into the shared interruption
  taxonomy instead of naming a second vocabulary

## Consumer promises

This contract keeps four promises.

### Products observe runtime truth

Products may inspect interruption state through runtime/export/host-edge
surfaces, but they should not need to infer it from unrelated counters,
watchdog prose, or host-private helper state.

### One vocabulary spans realtime and deferred work

Realtime recovery, transport/plugin repair, deferred work, and offline render
continuity may differ operationally, but they must share one interruption
taxonomy.

### Host edges do not get a competing taxonomy

Stable host edges may expose interruption state, but they must point back to
runtime-owned meaning. Local/server host summaries or helpers must not become a
second classification system.

### Future milestones refine fields, not meaning

Later `g06` milestones may widen typed DTOs, add richer interruption receipts,
or sharpen mapping for specific subsystems. They must extend this vocabulary,
not replace it.

## Deferred interruption classes

This Batch 1.1 contract intentionally defers:

- explicit per-subsystem interruption DTO fields for recording continuity,
  plugin rebind, render resumability, and device supervision
- promotion of current deferred `server soak` acceptance into stronger shared
  release evidence
- remote or distributed orchestration policy
- product UX copy, user choice, or session-level recovery workflow

Those areas belong to later `g06` milestones, but they should now build on the
same shared vocabulary.

## Current baseline surfaces

The current repo-owned baseline that this contract builds on is:

- `crates/signal-runtime/src/interfaces.rs`
- `crates/signal-runtime/src/runtime.rs`
- `RuntimeFaultStatusSnapshot`
- `RuntimeDegradationSummary`
- `RecoveryRecord`
- `RuntimeSoakReceipt`
- `RuntimeSupervisorReport`
- `RuntimeObservationReport`
- `RuntimeSupervisorApi::supervisor_report()`
- offline render execution pause/resume/interrupt APIs in `signal-runtime`
- `docs/contracts/002-supervisor-export-schema-and-report-boundary.md`
- `docs/contracts/005-runtime-work-orchestration-and-deferred-service-policy.md`
- `docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md`

## Next Task

Continue `g06.001` with Batch 1.2 by applying this interruption vocabulary to
the active runtime-owned snapshots, receipts, and host-facing shared boundary
surfaces without creating host-local reconstruction paths.
