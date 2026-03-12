# 005 - Plugin Backend Breadth And Host-Neutral Delegation Contracts

Status: complete
Owner: core-product
Created: 2026-03-12
Depends on: g04.001, g04.002, g04.003
Vision tags: `PLUGINS`, `BACKENDS`, `DELEGATION`

## Problem

Signal’s current plugin substrate is credible, but it is still shaped by the
current CLAP-first and delegated-execution proof path. The repo needs a stronger
format-neutral and host-neutral plugin/delegation contract before more consumers
or backends rely on it.

## Goals

- [ ] clarify the reusable plugin backend boundary inside Signal
- [ ] strengthen host-neutral delegation contracts for cases that exceed
  Signal-owned execution
- [ ] keep plugin lifecycle, render, and fault semantics aligned across backends

## Non-Goals

- [ ] no consumer-specific plugin browser or workflow work
- [ ] no format breadth for its own sake without contract value

## Execution Plan

### Batch 5.1 - Backend Boundary Contract

- [x] define which plugin lifecycle and capability surfaces are format-neutral
- [x] document what remains adapter-specific versus reusable
- [x] align delegation receipts with the now-explicit public contract boundary

### Batch 5.2 - Backend And Delegation Depth

- [x] deepen the chosen backend/delegation surfaces in Signal-owned crates
- [x] keep render, recovery, and scheduling semantics coherent across the wider boundary

### Batch 5.3 - Conformance Proof

- [x] add focused fixtures or tests showing the widened backend/delegation
  contract remains consumable without host-local reconstruction

## Progress Notes

- 2026-03-12: completed Batch 5.1 by freezing the first format-neutral plugin
  backend and host-neutral delegation contract in
  `docs/contracts/007-plugin-backend-and-host-neutral-delegation-contract.md`,
  separating `signal-plugin` authority from `signal-runtime` execution/export
  authority and explicitly classifying current `signal-plugin-clap` protocol
  helpers as adapter-specific rather than reusable consumer boundary.
- 2026-03-12: moved Batch 5.2 forward by adding runtime-owned plugin
  scan/discovery receipts, typed sandbox plugin-format tracking, and delegated
  offline stage inputs that carry plugin format/type identity without forcing
  hosts to reconstruct it from adapter-local state.
- 2026-03-12: moved Batch 5.2 forward again by promoting discovered-plugin
  catalog and capability detail into `RuntimePluginDiscoverySnapshot` via
  `RuntimePluginDiscoveredTypeRecord`, wiring both hosts to feed scan results
  back into runtime-owned discovery receipts, and proving one host-observation
  consumer path can inspect those catalogs without adapter-local reconstruction.
- 2026-03-12: completed Batch 5.3 by adding a downstream-style
  `signal-runtime` public-boundary proof plus a `signal-supervisor-tools`
  export-consumer proof that both read the widened discovery catalog through
  runtime-owned receipts and supervisor export without CLAP-side
  reconstruction, while explicitly leaving broader non-CLAP adapter breadth
  deferred.

## Acceptance Criteria

- [x] Signal has a clearer host-neutral plugin backend/delegation contract
- [x] later consumers can integrate richer plugin paths without guessing ownership
- [x] backend breadth does not reopen runtime/host boundaries already frozen in `g04`

## Risks and Mitigations

- Risk: backend work drifts into product-level plugin UX requirements.
- Mitigation: keep the milestone to engine/runtime/delegation contracts only.
- Risk: delegation contracts multiply instead of converging.
- Mitigation: require one reusable receipt family across the widened backend path.

## Evidence Requirements

- [x] log each meaningful backend/delegation tranche
- [x] run focused conformance validation for the widened contract
- [x] record explicit deferred backend breadth that remains out of scope

## Next Task

COMPLETE. `g04.005` closed before the generation closeout, and `g04` is now
complete. The next likely queue is recorded in
`docs/roadmaps/backlog/post-g04-consumer-release-and-backend-breadth.md`.
