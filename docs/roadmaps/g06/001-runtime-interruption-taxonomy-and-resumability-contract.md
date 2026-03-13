# 001 - Runtime Interruption Taxonomy And Resumability Contract

Status: active
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

- [ ] apply the contract to existing runtime-owned snapshots and receipts
- [ ] keep local and server host consumers aligned to the same meaning

### Batch 1.3 - Public Boundary Proof

- [ ] add focused proof that a downstream consumer can inspect interruption and
  resumability state without host-local reconstruction

## Progress Notes

- 2026-03-13: activated `g06.001` as the first active `g06` milestone so the
  generation opens on one explicit recovery and interruption vocabulary.
- 2026-03-13: completed Batch 1.1 by freezing contract `012`, defining the
  shared interruption taxonomy around `resumable`, `restartable`,
  `recoverable`, `terminal`, and `rebindable`, and mapping that vocabulary
  directly onto current runtime-owned fault, degradation, recovery-history, and
  deferred/offline continuity surfaces.

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

Continue `g06.001` with Batch 1.2 by applying the new interruption contract to
active runtime-owned snapshots and receipts, then keep local and server host
consumers aligned to the same meaning.
