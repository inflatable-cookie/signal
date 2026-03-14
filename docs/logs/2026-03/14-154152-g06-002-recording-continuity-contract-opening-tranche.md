# 2026-03-14 15:41:52 - g06.002 recording continuity contract opening tranche

## What changed

- completed `g06.002` Batch 2.1
- added contract `013` to freeze one shared recording continuity and
  checkpoint vocabulary for audio and MIDI capture
- defined capture identity, checkpoint classes, resumable capture,
  restartable capture, terminal capture, and committed evidence on top of the
  now-closed interruption vocabulary from contract `012`
- mapped the current runtime baseline to existing audio capture surfaces so
  Batch 2.2 can deepen implementation without inventing a second continuity
  model

## Evidence

- `docs/contracts/013-recording-continuity-midi-capture-and-checkpoint-contract.md`
- `docs/roadmaps/g06/002-recording-continuity-midi-capture-and-checkpoint-truth.md`
- `docs/contracts/README.md`
- `docs/architecture/graph-runtime-feature-reference.md`

## Validation

- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Deferred

- MIDI capture still does not have concrete runtime DTOs or commit receipts
- runtime capture surfaces still need typed checkpoint and restart-survival
  detail beyond the current audio capture snapshot and commit receipt
- downstream consumer proof remains for Batch 2.3 after Batch 2.2 lands real
  capture-depth surfaces

## Next Task

Continue `g06.002` with Batch 2.2 by deepening runtime capture state and
checkpoint receipts for audio and MIDI continuity, then align host-edge export
and supervisor views to the same capture truth without introducing host-local
recording recovery policy.
