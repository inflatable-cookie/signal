# 002 - Recording Continuity, MIDI Capture, And Checkpoint Truth

Status: planned
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

- [ ] define resumable, restartable, and terminal capture checkpoints for audio
  and MIDI paths
- [ ] freeze what evidence survives runtime interruption and restart

### Batch 2.2 - Runtime Capture Depth

- [ ] deepen runtime capture state and checkpoint receipts for audio and MIDI
- [ ] align host-edge export and supervisor views to the same capture truth

### Batch 2.3 - Focused Recovery Proof

- [ ] add focused proofs for resumed, restarted, and failed capture cases
- [ ] verify downstream consumers can distinguish them without parsing logs

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

Continue `g06.003` by applying the same recovery model to plugin transport and
shared-sandbox continuity.
