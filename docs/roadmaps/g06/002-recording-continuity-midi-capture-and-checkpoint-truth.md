# 002 - Recording Continuity, MIDI Capture, And Checkpoint Truth

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g06.001
Vision tags: `RUNTIME`, `RECORDING`, `MIDI`

## Problem

Loophole still needs stronger reusable truth for what recording can resume,
restart, checkpoint, or fail explicitly under runtime interruption. Signal also
needs a clearer story for MIDI capture continuity instead of treating audio
capture as the only first-class recording path.

## Goals

- [ ] define recording continuity and checkpoint semantics for audio and MIDI
- [ ] decide which capture progress can resume truthfully after interruption
- [ ] expose runtime-owned recording continuity receipts strong enough for host
  recovery and later soak work

## Non-Goals

- [ ] no product-local take management workflow scope
- [ ] no MIDI editor or arrangement UX work

## Execution Plan

### Batch 2.1 - Capture Continuity Contract

- [x] define resumable, restartable, and terminal capture checkpoints for audio
  and MIDI paths
- [x] freeze what evidence survives runtime interruption and restart

## Progress Notes

- 2026-03-14: completed Batch 2.1 by freezing contract `013`, defining one
  shared audio/MIDI capture continuity vocabulary around capture identity,
  checkpoint classes, resumable versus restartable capture, terminal failure,
  and committed evidence on top of contract `012` instead of a second recovery
  language.

### Batch 2.2 - Runtime Capture Depth

- [x] deepen runtime capture state and checkpoint receipts for audio and MIDI
- [x] align host-edge export and supervisor views to the same capture truth

- 2026-03-14: completed Batch 2.2 by widening
  `RuntimeRecordingCaptureSnapshot` and
  `RuntimeRecordingCaptureCommitReceipt` with typed capture kind and checkpoint
  surfaces, preserving restartable buffered checkpoints across stop or
  reconfigure, and wiring `recording_capture_snapshot` into
  `RuntimeObservationReport` or `RuntimeSupervisorReport` so shared host edges
  consume the same runtime-owned capture truth instead of reconstructing it.

### Batch 2.3 - Focused Recovery Proof

- [x] add focused proofs for resumed, restarted, and failed capture cases
- [x] verify downstream consumers can distinguish them without parsing logs

- 2026-03-14: completed Batch 2.3 by adding focused runtime proofs for
  resumable, restartable, and terminal capture outcomes, adding downstream-style
  runtime and shared host-edge proofs for those cases, and promoting the batch
  into the repo-owned `signal.runtime.recording-continuity-boundary` descriptor
  plus `effigy acceptance:recording-continuity`.

## Acceptance Criteria

- [ ] Signal has explicit audio and MIDI capture continuity semantics
- [ ] recording checkpoints and recovery evidence are runtime-owned
- [ ] later product hosts can build recording recovery on reusable capture truth

## Risks And Mitigations

- Risk: MIDI capture stays a second-class path.
- Mitigation: freeze audio and MIDI continuity in the same milestone.
- Risk: checkpointing drifts into product persistence policy.
- Mitigation: keep this milestone on runtime continuity truth only.

## Evidence Requirements

- [ ] log each meaningful capture-continuity tranche
- [ ] run focused runtime tests for resumed/restarted/failed capture
- [ ] record deferred capture breadth explicitly

## Next Task

Continue `g06.003` with Batch 3.1 by freezing placement-rule vocabulary,
sandbox grouping keys, and shared rebind or continuity semantics before plugin
adapter breadth and richer recovery implementation widen further.
