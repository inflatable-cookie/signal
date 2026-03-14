# 001 - Runtime Interruption Taxonomy And Resumability Contract

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g05.005
Vision tags: `RUNTIME`, `RECOVERY`, `CONTRACTS`

## Problem

Signal currently exposes several recoverable-versus-terminal paths, but the
meaning of interruption, resumability, rebindability, and terminal failure is
still spread across narrower feature surfaces. The next generation needs one
typed interruption vocabulary that later recording, plugin, render, hardware,
and soak work can all share.

## Goals

- [ ] define one runtime-owned interruption taxonomy across playback, capture,
  plugin transport, deferred work, and offline render
- [ ] freeze the difference between resumable, restartable, recoverable, and
  terminal runtime outcomes
- [ ] keep products observing runtime interruption truth instead of
  reconstructing it host-locally

## Non-Goals

- [ ] no product-specific session UX or recovery copy
- [ ] no broad remote/distributed orchestration policy yet

## Execution Plan

### Batch 1.1 - Interruption Contract

- [x] define the interruption classes, boundaries, and resumability vocabulary
- [x] align the contract with existing runtime/export/host-edge receipts

### Batch 1.2 - Runtime Surface Alignment

- [x] apply the contract to existing runtime-owned snapshots and receipts
- [x] keep local and server host consumers aligned to the same meaning

### Batch 1.3 - Public Boundary Proof

- [x] add focused proof that a downstream consumer can inspect interruption and
  resumability state without host-local reconstruction

## Progress Notes

- 2026-03-13: activated `g06.001` as the first active `g06` milestone so the
  generation opens on one explicit recovery and interruption vocabulary.
- 2026-03-13: completed Batch 1.1 by freezing contract `012`, defining the
  shared interruption taxonomy around `resumable`, `restartable`,
  `recoverable`, `terminal`, and `rebindable`, and mapping that vocabulary
  directly onto current runtime-owned fault, degradation, recovery-history, and
  deferred/offline continuity surfaces.
- 2026-03-13: completed Batch 1.2 by adding runtime-owned `fault_status` and
  `interruption_summary` export to `RuntimeObservationReport` /
  `RuntimeSupervisorReport`, tagging deferred-work and offline-render progress
  receipts with interruption class, and proving local/server host-edge
  consumers inherit the same meaning without host-local reconstruction.
- 2026-03-14: completed Batch 1.3 by adding focused downstream-style runtime
  proofs for restartable and resumable interruption inspection, a repo-owned
  `acceptance:interruption-boundary` task, and a machine-readable
  interruption-boundary descriptor so the milestone closes on consumable public
  evidence rather than crate-local tests alone.

## Acceptance Criteria

- [ ] Signal has one explicit interruption and resumability contract
- [ ] later recovery milestones can build on typed runtime-owned meaning
- [ ] host consumers no longer need to infer recovery class from unrelated state

## Risks And Mitigations

- Risk: interruption semantics become too abstract to guide implementation.
- Mitigation: require direct mapping to existing runtime features and receipts.
- Risk: host convenience APIs reintroduce competing taxonomies.
- Mitigation: freeze the consumer contract at runtime/export/host-edge surfaces first.

## Evidence Requirements

- [ ] log each meaningful contract tranche
- [ ] run focused contract validation and public-boundary proof
- [ ] record any deferred interruption classes explicitly

## Next Task

Continue `g06.002` with Batch 2.1 by defining the recording continuity and
checkpoint contract for audio and MIDI capture, then freeze what capture
evidence survives interruption, restart, and failed runtime boundaries.
