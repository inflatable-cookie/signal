# 002 - Shared Host Convenience API And Consumer-Edge Contracts

Status: complete
Owner: core-product
Created: 2026-03-12
Depends on: g05.001
Vision tags: `HOSTS`, `CONSUMERS`, `APIS`

## Problem

`g04` intentionally left host convenience APIs outside the first stable Signal
promise. The next generation needs an explicit decision about which of those
surfaces become stable shared consumer edges and which remain intentionally
unstable.

Without a dedicated host-edge contract milestone:

- consumers will keep inferring stability from current host implementations
- release packaging will over-claim shared API support that was never frozen
- backend breadth work can leak adapter-specific affordances into nominally
  shared host surfaces
- downstream conformance will not know which host entry points belong in the
  shared acceptance boundary

## Goals

- [x] define which host convenience APIs are stable shared consumer edges
- [x] keep runtime/export authority separate from host-edge convenience layers
- [x] document which host surfaces remain intentionally unstable
- [x] prepare a stronger boundary for later packaging and consumer automation

## Non-Goals

- [ ] no product workflow or UI ownership
- [ ] no consumer-specific helper layer in Signal
- [ ] no backend-specific convenience surface promoted without a contract

## Execution Plan

### Batch 2.1 - Host-Edge Stability Contract

- [x] classify shared host convenience APIs by stability tier
- [x] define how those APIs depend on the existing runtime/export/plugin
  authority rather than replacing it

### Batch 2.2 - Host-Edge Receipt And Export Alignment

- [x] make the chosen stable host-edge surfaces inspectable through Signal-owned
  receipts or exports
- [x] keep intentionally unstable host behavior explicit and bounded

### Batch 2.3 - Consumer Proof

- [x] add a focused consumer-facing proof that stable host-edge surfaces remain
  usable without private host internals

## Progress Notes

- 2026-03-12: seeded `g05.002` so host convenience APIs are promoted or
  deferred by explicit contract rather than accidental reuse.
- 2026-03-12: completed Batch 2.1 by freezing
  `docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md`,
  classifying the first shared-stable host edges around construction,
  `RuntimeSupervisorApi`, and `supervisor_report()`, while explicitly keeping
  host-specific summary/enrichment surfaces, scenario boot helpers, and local
  delegated-executor convenience methods outside the stable shared tier.
- 2026-03-12: completed Batch 2.2 by adding the machine-readable
  `signal.host.edge.boundary` descriptor to `signal-supervisor-tools` and the
  repo-owned `acceptance:host-edge-boundary` task, making the stable host-edge
  classification inspectable without promoting the intentionally unstable host
  helper families into the shared tier.
- 2026-03-12: completed Batch 2.3 by adding downstream-style integration proofs
  in `signal-host-local` and `signal-host-server`, then folding that proof into
  the repo-owned `acceptance:host-edge-consumer` task and the runnable
  conformance matrix so the shared-stable host edge is proven without private
  host internals or unstable summary/helper surfaces.

## Acceptance Criteria

- [x] stable host convenience APIs are explicitly named
- [x] intentionally unstable host edges stay explicit
- [x] consumers can tell which host surfaces are part of the shared Signal
  boundary

## Risks And Mitigations

- Risk: host-edge work becomes a stealth product-integration backlog.
- Mitigation: freeze only shared consumer boundaries, not app-specific workflow.
- Risk: convenience APIs drift away from runtime/export authority.
- Mitigation: require typed linkage back to Signal-owned runtime surfaces.

## Evidence Requirements

- [x] log each meaningful host-edge tranche
- [x] run focused validation for the chosen stable host-edge boundary
- [x] record what remains intentionally unstable after the first pass

## Next Task

COMPLETE. `g05.002` closed after the shared-stable host edge was classified,
made inspectable, and proven through public consumer-facing host paths without
private host internals or unstable summary/helper surfaces.

Continue `g05.003` with Batch 3.3 by proving the publication packaging manifest
and release-receipt family stay consumable without promoting unstable host
helpers into the shared release edge.
