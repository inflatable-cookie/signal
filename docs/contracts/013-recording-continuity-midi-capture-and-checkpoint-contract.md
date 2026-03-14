# 013 Recording Continuity, MIDI Capture, And Checkpoint Contract

Status: complete
Owner: core-product
Updated: 2026-03-14
Related contracts: `docs/contracts/002-supervisor-export-schema-and-report-boundary.md`, `docs/contracts/005-runtime-work-orchestration-and-deferred-service-policy.md`, `docs/contracts/012-runtime-interruption-taxonomy-and-resumability-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the shared recording continuity and checkpoint vocabulary for `g06.002`
 so later audio capture, MIDI capture, restart handling, soak evidence, and
 host recovery all build on runtime-owned meaning instead of product-local
 policy.

## Authority hierarchy

Recording continuity has one authority chain:

1. `signal-runtime` owns the canonical capture truth:
   - active capture readiness and failure state
   - checkpoint continuity and buffered progress
   - committed capture evidence
   - interruption and restart relationship to capture identity
2. supervisor/export surfaces and stable host edges may expose that truth, but
   they must not reinterpret it:
   - `RuntimeRecordingCaptureSnapshot`
   - `RuntimeRecordingCaptureCommitReceipt`
   - `RuntimeObservationReport`
   - `RuntimeSupervisorReport`
   - `RuntimeSupervisorApi`
3. products may react to capture continuity state, but they must not become the
   authority for deciding whether a capture resumed, restarted, committed, or
   failed terminally

If a recording continuity claim cannot be explained through runtime-owned
snapshots, receipts, or export surfaces, it is not yet part of the shared
contract.

## Shared capture terms

This contract freezes six shared terms.

### Capture identity

A capture identity is the runtime-owned recording work identity that survives
through progress, interruption, restart, commit, or failure reporting.

Current audio capture already has a partial identity through:

- `take_id`
- `track_id`
- `capture_path`
- `capture_start_samples`

Later MIDI capture must gain the same authoritative identity instead of hiding
behind separate product workflow objects.

### Checkpoint

A checkpoint is any runtime-owned progress marker that says what durable or
restart-relevant capture evidence currently exists.

The first shared checkpoint classes are:

- `armed`
  - capture was accepted and assigned a runtime-owned identity
- `streaming`
  - capture is actively accumulating timeline-linked data
- `buffered`
  - runtime has captured data that is not yet committed but is still part of
    the authoritative capture identity
- `committed`
  - runtime finalized a capture receipt and durable artifact/result under the
    same authoritative identity
- `failed`
  - runtime can no longer safely continue or commit the current capture
    identity

Later milestones may add richer typed checkpoint DTOs, but they must keep these
classes stable in meaning.

### Resumable capture

`Resumable` means the same runtime-owned capture identity may continue after an
interruption without allocating a new authoritative capture boundary.

For recording continuity this requires:

- the same capture identity remains authoritative
- buffered or checkpointed capture evidence is still valid
- products do not need to create a new take just to continue the same runtime
  capture attempt

### Restartable capture

`Restartable` means runtime must rebuild capture execution state, but the shared
recording boundary still explains what survived and what must restart under a
new capture attempt.

Restartable does not imply same-capture continuity. It means runtime, not the
host, owns the restart boundary and exposes whether prior checkpointed evidence
survived.

### Terminal capture

`Terminal` means the current capture attempt cannot safely continue or commit as
the same authoritative recording boundary.

Terminal capture must remain explicit on runtime surfaces rather than being
inferred from missing files, missing clips, or product-local error copy.

### Committed evidence

Committed evidence is the runtime-owned result that survives a successful
capture finalization.

For current audio capture that already includes:

- committed take identity
- committed path
- committed duration
- committed channel count
- committed peak level

Later MIDI capture must expose an equivalent runtime-owned commit receipt rather
than treating MIDI commit as product-private metadata.

## Capture continuity rules

This contract freezes five shared rules.

### Rule 1: interruption vocabulary applies to recording

Recording continuity must compose with contract `012` instead of creating a
parallel recovery language.

- resumable capture aligns with `Resumable`
- restarted capture aligns with `Restartable`
- failed capture aligns with `Terminal`

### Rule 2: buffered evidence is runtime-owned checkpoint state

Buffered frames, events, and checkpoint progress belong to runtime-owned
capture surfaces, not private host scratch state.

Products may consume them, but they must not be the primary ledger.

### Rule 3: audio and MIDI share one continuity model

Audio capture and MIDI capture may differ in artifact type, but they must share
the same checkpoint and interruption language.

MIDI must not become a second-class recording path with weaker continuity
promises.

### Rule 4: commit receipts are distinct from product arrangement policy

Runtime owns whether capture committed and what evidence survived.

Products still own later arrangement, naming, take comping, or editor workflow,
but those flows must build on the runtime commit receipt instead of replacing
it.

### Rule 5: host edges expose capture truth, not private heuristics

Local/server hosts may surface capture continuity through shared observation and
supervisor export, but they must not infer resumability or restart state from
file existence, buffer sizes, or UI workflow rules.

## Current runtime mapping

The current baseline already has part of the capture continuity seam.

### Audio capture state

`RuntimeRecordingCaptureSnapshot` is the current broad audio-capture state
surface:

- `capture_ready`
- `state`
- active take and track identity
- active capture path
- buffered block and frame counts
- captured channel count
- peak level
- pressure event count
- last committed identity and duration
- last error

This is now a typed runtime-owned checkpoint and readiness seam:

- `capture_kind`
- `active_checkpoint`
- `last_checkpoint`
- typed checkpoint classes
- interruption-aligned checkpoint continuity
- explicit buffered event count for the shared audio/MIDI continuity family

### Audio capture commit

`RuntimeRecordingCaptureCommitReceipt` is the current committed evidence seam:

- committed take identity
- committed track identity
- timeline start
- duration
- channel count
- peak level
- capture path

This is already the authoritative commit receipt for audio capture.

### Shared observation export

`RuntimeObservationReport` and `RuntimeSupervisorReport` now carry
`recording_capture_snapshot` through runtime-owned observation boundaries, so
stable host edges and downstream consumers can inspect the same typed capture
truth without reconstructing checkpoint state from logs or private helpers.

### Repo-owned proof boundary

`signal-supervisor-tools` now exposes the machine-readable
`signal.runtime.recording-continuity-boundary` descriptor, and
`effigy acceptance:recording-continuity --repo .` is the repo-owned runnable
proof that resumable, restartable, and terminal capture outcomes stay visible
through shared runtime and host-edge surfaces.

## Consumer promises

This contract keeps four promises.

### Recording recovery remains runtime-owned

Consumers may inspect recording continuity, but they should not need to decide
resumability or restart survival from unrelated host-local state.

### Audio and MIDI remain one continuity family

Later MIDI capture work must extend the same runtime-owned continuity boundary
used by audio capture.

### Checkpoint truth is stronger than prose or logs

Capture checkpoint and commit meaning should be visible through typed runtime
surfaces, not inferred from log lines or ad hoc progress text.

### Future milestones refine DTOs, not semantics

Later `g06` work may add richer checkpoint receipts, restart receipts, or
MIDI-specific commit surfaces, but it must preserve this continuity meaning.

## Deferred scope

This Batch 2.1 contract intentionally defers:

- concrete MIDI capture DTOs and artifact receipts
- restart-aware capture receipts that distinguish same-identity resume from
  new-attempt restart
- product-local take arrangement, comping, naming, or editor workflow
- remote/distributed capture orchestration or collaboration policy

Those areas belong to later `g06` batches, but they should now extend one
shared recording continuity vocabulary.

## Current baseline surfaces

The current repo-owned baseline that this contract builds on is:

- `crates/signal-runtime/src/interfaces.rs`
- `crates/signal-runtime/src/runtime.rs`
- `RuntimeRecordingCaptureSnapshot`
- `RuntimeRecordingCaptureCommitReceipt`
- `RuntimeObservationReport`
- `RuntimeSupervisorReport`
- `RuntimeSupervisorApi`
- `docs/contracts/012-runtime-interruption-taxonomy-and-resumability-contract.md`

## Next Task

Continue `g06.003` with Batch 3.1 by defining the shared plugin rebind,
placement, and shared-sandbox continuity contract on top of the now-closed
recording and interruption continuity vocabulary.
