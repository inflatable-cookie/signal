# 015 Offline Render Recovery And Resumability Contract

Status: complete
Owner: core-product
Updated: 2026-03-14
Related contracts: `docs/contracts/002-supervisor-export-schema-and-report-boundary.md`, `docs/contracts/005-runtime-work-orchestration-and-deferred-service-policy.md`, `docs/contracts/012-runtime-interruption-taxonomy-and-resumability-contract.md`, `docs/contracts/013-recording-continuity-midi-capture-and-checkpoint-contract.md`, `docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first shared offline-render recovery and resumability contract for
`g06.004` so later runtime session-depth work, artifact survival rules, and
host-facing render consumers all extend one runtime-owned recovery meaning
instead of staging host-local queue or retry models.

## Authority hierarchy

Offline-render recovery has one authority chain:

1. `signal-runtime` owns the canonical render-session truth:
   - render request identity
   - execution checkpoints and progress state
   - interruption and resumability meaning
   - cancellation and completion receipts
   - manifest, artifact, and purge alignment around one request boundary
2. supervisor/export surfaces and stable host edges may expose that truth, but
   they must not reinterpret it:
   - `RuntimeOfflineRenderExecutionProgressReceipt`
   - `RuntimeOfflineRenderExecutionReceipt`
   - `RuntimeOfflineRenderExecutionCancellationReceipt`
   - `RuntimeOfflineRenderQueueResult`
   - `RuntimeOfflineRenderResult`
   - `RuntimeObservationReport`
   - `RuntimeSupervisorReport`
3. products may react to render continuity state, but they must not become the
   authority for deciding whether a render request resumed, restarted,
   recovered, completed, cancelled, or failed terminally

If render recovery meaning cannot be explained through runtime-owned snapshots,
receipts, or export surfaces, it is not yet part of the shared contract.

## Shared terms

This contract freezes seven shared terms.

### Render session identity

A render session identity is the runtime-owned request boundary that survives
queue admission, checkpoint emission, pause or interruption handling,
completion, cancellation, and purge reporting.

Current runtime identity already begins with:

- `request_id`
- render timeline start and duration
- export sample rate
- requested main-mix, stem, and freeze outputs
- artifact root path when materialization is requested

Later milestones may add richer execution-owner or host-routing detail, but the
authoritative session identity stays runtime-owned.

### Checkpoint

A checkpoint is any runtime-owned progress receipt that says what trustworthy
render evidence exists for one active session.

The first shared checkpoint stages are:

- `PreparingInput`
- `RenderingGraph`
- `MaterializingOutputs`
- `FinalizingArtifacts`

Checkpoint emission is part of the canonical session story, not optional host
logging.

### Resumable render session

`Resumable` means the same runtime-owned render session identity may continue
after interruption without allocating a new authoritative request boundary.

Current examples already implied by runtime surfaces:

- paused execution that later resumes through the same `request_id`
- queue-deferred work that remains the same request while runtime clears safe
  mode or other bounded policy gates

### Restartable render session

`Restartable` means runtime may preserve the authoritative session identity and
consumer-facing request boundary while it re-establishes internal execution
state before steady progress resumes.

This is stronger than starting over in product code and weaker than guaranteed
same-checkpoint continuity.

### Recoverable render session

`Recoverable` means runtime is inside an active render repair or degraded path,
but the same authoritative session boundary remains live and may return to
steady progress without a new host-local recovery model.

### Terminal render session

`Terminal` means runtime can no longer safely continue the current authoritative
render session as the same request boundary.

Terminal render outcome must be exported explicitly through receipts or aligned
failure state rather than inferred from missing artifacts, missing report
output, or product-local retry policy.

### Rebindable render session

`Rebindable` is a property of some resumable or restartable render
interruptions where runtime may reattach delegated plugin execution, artifact
materialization, or adjacent owned services without changing the authoritative
session identity.

Rebindable is not a second top-level render vocabulary. It composes with the
shared interruption taxonomy from contract `012`.

## Continuity rules

This contract freezes six shared rules.

### Rule 1: interruption taxonomy stays shared

Contract `012` remains the top-level vocabulary for render continuity:

- render sessions may be `Resumable`
- render sessions may be `Restartable`
- render sessions may be `Recoverable`
- render sessions may become `Terminal`
- some repair paths may additionally be `Rebindable`

`g06.004` must extend that vocabulary, not replace it with render-only labels.

### Rule 2: request identity stays runtime-owned

Hosts may submit render intent, but runtime owns the authoritative session
identity once a request is admitted. Consumers must not create a second
render-session identity by wrapping checkpoints or artifacts in product-local
queue objects.

### Rule 3: checkpoint truth precedes artifact interpretation

Consumers should read runtime-owned execution checkpoints and progress receipts
before inventing render survival stories from filesystem side effects,
partially written artifacts, or report timing.

### Rule 4: queue orchestration and execution continuity are one story

Deferred queue decisions, active execution progress, cancellation, completion,
and purge semantics all belong to one runtime-owned render continuity model.
Consumers should not need separate host-local ledgers to understand whether a
session deferred, resumed, completed, or failed terminally.

### Rule 5: manifests and artifacts follow recovery truth

Render manifests, artifact receipts, and report receipts must stay aligned with
the authoritative session outcome. Artifact presence alone must not become the
shared source of truth for whether a session completed or survived interruption.

### Rule 6: host edges expose render continuity truth, not retry policy

Stable host edges may expose render queue or session receipts, but they must
not reinterpret restartability, resumability, or failure outcome through
product-local retry logic or orchestration wrappers.

## Current runtime mapping

The current repo baseline already contains the runtime-owned surfaces this
contract builds on.

### Request and result boundary

`signal-runtime` already owns the request/result family:

- `RuntimeOfflineRenderRequest`
- `RuntimeOfflineRenderResult`
- `RuntimeOfflineRenderManifest`
- `RuntimeOfflineRenderArtifactReceipt`
- `RuntimeOfflineRenderReportReceipt`

These types already keep render products anchored to one runtime-owned request
and one aligned output/result boundary.

### Queue and orchestration continuity

Current runtime queue and deferred-work surfaces already express bounded render
continuity:

- `RuntimeDeferredServiceReceipt`
- `RuntimeOfflineRenderQueueResult`
- `RuntimeOfflineRenderQueueProgressReceipt`
- `RuntimeOfflineRenderPurgeReceipt`

These surfaces already distinguish run, defer, throttle, and abort decisions
through the shared interruption vocabulary.

### Execution progress and checkpoints

Current runtime execution surfaces already expose the first typed execution
continuity seam:

- `RuntimeOfflineRenderCheckpointReceipt`
- `RuntimeOfflineRenderExecutionReceipt`
- `RuntimeOfflineRenderExecutionProgressReceipt`
- `RuntimeOfflineRenderExecutionCancellationReceipt`
- `RuntimeOfflineRenderExecutionState`

`RuntimeOfflineRenderExecutionProgressReceipt` already carries:

- `state`
- `interruption_class`
- `interruption_rebindable`
- optional active checkpoint
- optional completed result

This milestone freezes the meaning of those fields before deeper runtime
session work widens them.

### Shared interruption anchor

The adjacent top-level interruption seam remains:

- `RuntimeInterruptionSummary`
- `RuntimeFaultStatusSnapshot`
- `RuntimeObservationReport`
- `RuntimeSupervisorReport`

These surfaces keep render continuity aligned with the broader runtime recovery
story instead of isolating offline render into a separate host-local model.

## Consumer promises

This contract keeps four promises.

### Products observe runtime render truth

Consumers may inspect render continuity, but they should not need to infer it
from local queue retries, partial artifact directories, or wrapper-specific
status logs.

### Render recovery extends one shared vocabulary

Render interruption and resumability semantics stay aligned with the same
runtime-owned language already used for deferred work, recording, and plugin
continuity.

### Session truth stays stronger than artifact side effects

Consumers can rely on runtime checkpoints and progress receipts first, with
artifacts and reports aligned underneath that same authority.

### Later depth widens DTOs, not semantics

Batch 4.2 and later milestones may add richer render-session receipts,
checkpoint fields, and host-edge export, but they must preserve the meanings
frozen here.

## Batch 4.2 runtime mapping

Batch 4.2 now makes render-session continuity inspectable through one
runtime-owned snapshot family:

- `RuntimeOfflineRenderSessionSnapshot`
- `RuntimeOfflineRenderSessionStateSnapshot`

That widened surface now aligns:

- active render execution state
- last observed render session state after pause, resume, recoverable
  interruption, completion, or cancellation
- last emitted checkpoint per session
- last cancellation receipt
- last purge receipt
- observation and supervisor export of the same continuity truth

This means Batch 4.3 can focus on proving more outcome classes instead of
debating where render continuity truth should live.

## Batch 4.3 proof closure

Batch 4.3 now closes the first shared offline-render continuity boundary:

- resumable render sessions are proven through pause and recoverable
  interruption checkpoints
- restartable render sessions are proven through runtime stop and restart
  without losing authoritative session identity
- terminal render sessions are proven through typed failed-session export on
  delivery or materialization error
- the consumer-facing `signal.runtime.offline-render-continuity-boundary`
  descriptor and repo-owned acceptance task are now part of the reusable
  boundary instead of deferred prose

## Deferred scope

This contract now intentionally defers:

- deeper checkpoint survival rules across runtime restart or process restart
- durable distributed queue orchestration or remote job ownership
- publication or browser UX for incomplete or failed artifact sets

Those areas belong to later recovery or orchestration milestones, but they
should now build on this shared runtime-owned render continuity vocabulary and
the new runtime session snapshot family.

## Current baseline surfaces

The current repo-owned baseline that this contract builds on is:

- `crates/signal-runtime/src/interfaces.rs`
- `crates/signal-runtime/src/runtime.rs`
- `RuntimeOfflineRenderRequest`
- `RuntimeOfflineRenderCheckpointReceipt`
- `RuntimeOfflineRenderExecutionProgressReceipt`
- `RuntimeOfflineRenderExecutionReceipt`
- `RuntimeOfflineRenderExecutionCancellationReceipt`
- `RuntimeOfflineRenderSessionSnapshot`
- `RuntimeOfflineRenderSessionStateSnapshot`
- `RuntimeOfflineRenderQueueResult`
- `RuntimeOfflineRenderPurgeReceipt`
- `RuntimeOfflineRenderResult`
- `RuntimeObservationReport`
- `RuntimeSupervisorReport`
- `crates/signal-supervisor-tools/src/main.rs`
- `effigy.toml`
- `docs/contracts/012-runtime-interruption-taxonomy-and-resumability-contract.md`
- `docs/contracts/005-runtime-work-orchestration-and-deferred-service-policy.md`

## Next Task

Contract `015` is complete. Continue `g06.005` with Batch 5.1 by defining the
runtime-owned fault-cause attribution and diagnostic receipt contract that
should sit beside the now-closed interruption, recording, plugin, and offline
render recovery boundaries.
