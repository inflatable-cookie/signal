# 004 - Offline Render Execution Recovery And Resumability Depth

Status: active
Owner: core-product
Created: 2026-03-13
Depends on: g06.001
Vision tags: `RENDER`, `RECOVERY`, `OFFLINE`

## Problem

Signal already owns offline render execution and session checkpoints, but the
current surface still needs a stronger reusable answer for interruption,
rebindability, resumability, and explicit non-resumable completion failure.

## Goals

- [ ] define runtime-owned recovery and resumability semantics for active
  offline render sessions
- [ ] decide what checkpoint evidence can survive interruption or restart
- [ ] keep render products observing runtime session truth rather than staging
  host-local recovery models

## Non-Goals

- [ ] no product-specific render queue UI or artifact browser work
- [ ] no durable distributed job orchestration yet

## Execution Plan

### Batch 4.1 - Render Session Recovery Contract

- [ ] define resumable, restartable, recoverable, and terminal render outcomes
- [ ] align render interruption semantics with the shared `g06.001` contract

### Batch 4.2 - Runtime Session Depth

- [ ] deepen render-session receipts, checkpoint survival, and rebind surfaces
- [ ] keep manifest, artifact, and purge semantics coherent with the new model

### Batch 4.3 - Recovery Proof

- [ ] add focused proofs for interrupted, resumed, restarted, and failed render
  session paths

## Acceptance Criteria

- [ ] Signal has explicit offline-render recovery and resumability semantics
- [ ] later product hosts can rely on runtime-owned render recovery truth
- [ ] render interruption evidence is typed and inspectable

## Risks And Mitigations

- Risk: render session recovery drifts into product queue policy.
- Mitigation: keep this milestone on runtime session truth and receipts only.
- Risk: artifact semantics get out of sync with recovery state.
- Mitigation: require manifest/report alignment as part of the runtime contract.

## Evidence Requirements

- [ ] log each meaningful render-recovery tranche
- [ ] run focused runtime recovery tests for render sessions
- [ ] record deferred queue/distribution depth explicitly

## Next Task

Continue `g06.004` with Batch 4.1 by defining resumable, restartable,
recoverable, and terminal offline-render session outcomes, then align render
interruption and checkpoint survival semantics with the shared `g06.001`
taxonomy before runtime session-depth work begins.
