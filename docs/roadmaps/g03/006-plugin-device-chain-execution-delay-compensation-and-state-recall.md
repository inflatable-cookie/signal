# 006 - Plugin Device-Chain Execution, Delay Compensation, And State Recall

Status: complete
Owner: core-product
Created: 2026-03-12
Depends on: g03.001, g03.003, g03.005
Vision tags: `ENGINE`, `PLUGIN`, `LATENCY`

## Problem

Signal can host plugin-backed nodes, but the reusable engine contract is still
too thin for credible device-chain execution, plugin latency propagation, and
stable state recall across richer routed graphs. That leaves a major engine
surface under-specified right where products will expect the most stability.

## Goals

- [x] deepen plugin-backed device-chain execution as a reusable engine contract
- [x] propagate latency and delay-compensation semantics through routed graphs
- [x] make chain state recall and degraded plugin-state handling explicit enough for reuse

## Non-Goals

- [ ] no product-specific plugin browser workflow
- [ ] no final cross-format feature-parity guarantee yet

## Execution Plan

### Batch 6.1 - Chain Contract And Latency Model

- [x] make plugin/device-chain ordering, latency propagation, and compensation semantics explicit in Signal-owned crates
- [x] define state-recall and degraded-instance expectations for plugin-backed nodes

### Batch 6.2 - Runtime Proof

- [x] validate compensated chain execution through runtime and host/supervisor-facing exports
- [x] cover plugin faulted or bypassed cases without collapsing chain semantics

### Batch 6.3 - Routed Compensation Follow-Through

- [x] propagate realized chain latency and compensation readiness into routed runtime/export summaries
- [x] cover graph refresh, rebinding, and cold-start cases without losing recall-state clarity

### Batch 6.4 - Recall Payload And Execution Ownership

- [x] define typed plugin state-recall payload/status surfaces for later offline render and recall work
- [x] prove recovered, quarantined, and unavailable recall states through host/supervisor-facing export without host-local bookkeeping

### Batch 6.5 - Recall Ownership Boundary Follow-Through

- [x] define the runtime-owned recall handoff boundary that later offline render/freeze entry points are allowed to consume
- [x] separate authoritative runtime recall payload from any derived export-only fields before `g03.007` opens

### Batch 6.6 - Recall Consumer Contract Follow-Through

- [x] define how later offline render/freeze request assembly references runtime recall handoff stages by stable identity instead of copying recall fields
- [x] add one API-local proof that future recall consumers can depend on the handoff snapshot without parsing supervisor/export-only summaries

## Acceptance Criteria

- [x] plugin chain execution is explicit enough for later offline render and recall work
- [x] latency compensation stays engine-owned rather than host-local bookkeeping
- [x] focused tests prove state recall and degraded plugin handling on real chain shapes

## Risks and Mitigations

- Risk: delay compensation becomes inseparable from one host assembly.
- Mitigation: keep compensation and recall rules in generic runtime/graph contracts.
- Risk: later offline render/freeze work copies recall ownership into host-local
  request assembly instead of consuming runtime-owned recall payload.
- Mitigation: define the handoff boundary inside `signal-runtime` first, then
  let later offline render/freeze work depend on that contract rather than
  recreating recall bookkeeping.
- Risk: later offline request assembly still re-identifies stages from textual
  export or duplicated fields instead of consuming stable runtime handoff
  identities.
- Mitigation: define a consumer-facing identity contract on top of the handoff
  snapshot before any offline render/freeze implementation starts.

## Evidence Requirements

- [x] log the chain-execution tranche
- [x] run focused runtime/plugin tests for latency and degraded-state cases
- [x] capture any intentionally deferred format-specific quirks explicitly

## Next Task

`g03.006` is complete. Hold this boundary as the source of truth until you are
ready to open `g03.007` around offline render, freeze, and stem export on top
of the runtime-owned recall handoff identities.
