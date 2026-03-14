# 004 - Offline Render Execution Recovery And Resumability Depth

Status: complete
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

- [x] define resumable, restartable, recoverable, and terminal render outcomes
- [x] align render interruption semantics with the shared `g06.001` contract

## Progress Notes

- 2026-03-14: completed Batch 4.1 by freezing contract `015`, anchoring
  offline-render recovery to the shared interruption taxonomy from contract
  `012`, and making render request identity, checkpoints, execution progress,
  queue orchestration, cancellation, manifest, artifact, and purge alignment
  part of one runtime-owned recovery story before runtime session-depth work
  widens DTOs or proofs.

### Batch 4.2 - Runtime Session Depth

- [x] deepen render-session receipts, checkpoint survival, and rebind surfaces
- [x] keep manifest, artifact, and purge semantics coherent with the new model

- 2026-03-14: completed Batch 4.2 by adding the runtime-owned
  `RuntimeOfflineRenderSessionSnapshot` / `RuntimeOfflineRenderSessionStateSnapshot`
  family to observation and supervisor export, preserving active and last render
  session continuity across begin, pause, resume, recoverable interruption,
  completion, cancellation, queue completion, and purge paths, and proving
  checkpoint survival plus completion/cancellation/purge coherence through
  focused runtime tests.

### Batch 4.3 - Recovery Proof

- [x] add focused proofs for interrupted, resumed, restarted, and failed render
  session paths

- 2026-03-14: completed Batch 4.3 by adding restartable and failed-terminal
  render-session outcomes to the runtime-owned session snapshot family,
  proving resumable, restartable, and terminal continuity through focused
  runtime tests plus downstream-style runtime and host-edge proofs, and
  freezing the dedicated `signal.runtime.offline-render-continuity-boundary`
  descriptor plus repo-owned acceptance task instead of deferring that
  consumer-facing boundary again.

## Acceptance Criteria

- [x] Signal has explicit offline-render recovery and resumability semantics
- [x] later product hosts can rely on runtime-owned render recovery truth
- [x] render interruption evidence is typed and inspectable

## Risks And Mitigations

- Risk: render session recovery drifts into product queue policy.
- Mitigation: keep this milestone on runtime session truth and receipts only.
- Risk: artifact semantics get out of sync with recovery state.
- Mitigation: require manifest/report alignment as part of the runtime contract.

## Evidence Requirements

- [x] log each meaningful render-recovery tranche
- [x] run focused runtime recovery tests for render sessions
- [ ] record deferred queue/distribution depth explicitly

## Next Task

`g06.004` is complete. Continue `g06.005` with Batch 5.2 by materializing
typed fault-cause and contributing-evidence receipts so later profiling,
pressure, and soak work can point at canonical diagnostic evidence instead of
counter-only summaries.
