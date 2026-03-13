# 001 - Backend-Neutral Plugin Capability And Adapter Breadth Baseline

Status: complete
Owner: core-product
Created: 2026-03-12
Depends on: g04.006
Vision tags: `PLUGINS`, `BACKENDS`, `CONTRACTS`

## Problem

`g04` proved a CLAP-first, runtime-owned plugin boundary, but the next
generation needs a stronger answer for broader adapter breadth without
reopening format-specific or host-local ownership.

Without a dedicated backend-breadth baseline:

- Signal consumers will keep guessing which capability surfaces are genuinely
  adapter-neutral
- new backend work risks bypassing the runtime/export/plugin authority frozen in
  `g04`
- wider discovery, capability, and lifecycle coverage will drift between hosts
  instead of staying reusable
- later packaging and conformance work will not know which backend claims are
  actually part of the shared contract

## Goals

- [x] define the first post-CLAP backend-neutral plugin capability boundary
- [x] classify what remains adapter-specific versus shared consumer contract
- [x] keep discovery, lifecycle, and delegation ownership runtime-owned
- [x] create a contract strong enough for later packaging and conformance work

## Non-Goals

- [ ] no product-specific plugin browser or preset workflow work
- [ ] no backend breadth for its own sake without contract value
- [ ] no host-local convenience surface promoted by accident

## Execution Plan

### Batch 1.1 - Capability And Adapter Contract

- [x] define the backend-neutral capability, lifecycle, and delegation promises
- [x] document which backend details remain adapter-private

### Batch 1.2 - Discovery And Capability Receipt Depth

- [x] widen runtime-owned discovery/catalog surfaces to cover the chosen
  backend-neutral breadth without host-local reconstruction
- [x] keep delegated/offline/plugin execution receipts aligned with the widened
  boundary

### Batch 1.3 - Conformance Proof

- [x] add focused proofs showing a wider backend path stays consumable through
  Signal-owned runtime/export surfaces

## Progress Notes

- 2026-03-12: opened `g05.001` from the promoted post-`g04` backlog item to
  widen plugin backend breadth only through runtime-owned capability and
  contract surfaces.
- 2026-03-12: completed Batch 1.1 by freezing
  `docs/contracts/008-backend-neutral-plugin-capability-and-adapter-breadth-contract.md`,
  making the first post-CLAP backend-neutral capability promises explicit and
  separating shared Signal-owned capability meaning from adapter-private
  backend detail before receipt or packaging work widens.
- 2026-03-12: completed Batch 1.2 by widening `signal-runtime` discovery
  receipts with runtime-owned format-coverage and capability-coverage aggregates,
  then proving the widened receipt family through runtime tests, the public
  contract boundary test, and supervisor export without adapter-local
  reconstruction.
- 2026-03-12: completed Batch 1.3 by promoting the widened backend-breadth
  conformance proofs into the repo-owned `acceptance:plugin-backend-breadth`
  Effigy task, then closing the milestone with explicit public-runtime and
  supervisor-export coverage proofs rather than leaving the widened receipt
  family as an informal extension of older CLAP-first conformance checks.

## Acceptance Criteria

- [x] Signal has an explicit backend-neutral plugin capability contract beyond
  the CLAP-first baseline
- [x] wider adapter breadth does not reopen host-local ownership
- [x] later packaging and conformance work can rely on the widened plugin claim

## Risks And Mitigations

- Risk: backend breadth drifts into format-specific implementation sprawl.
- Mitigation: require one Signal-owned capability and receipt vocabulary first.
- Risk: consumer pressure promotes host convenience behavior by accident.
- Mitigation: keep the milestone on runtime/export/plugin contracts only.

## Evidence Requirements

- [x] log each meaningful backend-breadth tranche
- [x] run focused validation for widened capability/receipt surfaces
- [x] record deferred backend breadth that still remains out of scope

## Next Task

COMPLETE. `g05.001` closed after the widened backend-neutral discovery and
capability receipt family was proven through public runtime reexports,
supervisor export, and the repo-owned `acceptance:plugin-backend-breadth`
consumer task.

Continue `g05.002` with Batch 2.1 by defining which host convenience APIs
become stable shared consumer edges without weakening the runtime/export/plugin
authority frozen in `g04` and widened in `g05.001`.
