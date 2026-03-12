# 001 - Mixer Graph, Buses, And Routing Topology Depth

Status: complete
Owner: core-product
Created: 2026-03-12
Depends on: g01.006, g01.007, g01.009
Vision tags: `ENGINE`, `MIX`, `ROUTING`

## Problem

Signal’s current graph and runtime baseline can execute routed node plans, but
the mixer-oriented topology contract is still too implicit for real track, bus,
send/return, and console-group engine work. Later metering, automation,
plugin-chain, and render tasks will keep rebuilding routing assumptions until
bus ownership and mixer graph semantics become first-class.

## Goals

- [x] make bus-group, send/return, and console-group topology explicit in the graph/runtime contract
- [x] give runtime one clear routed mixer summary that hosts and supervisor tools can reuse
- [x] keep routing semantics owned by Signal crates instead of host-local projection helpers

## Non-Goals

- [ ] no full DAW workflow or UI work here
- [ ] no final device-chain latency compensation yet

## Execution Plan

### Batch 1.1 - Mixer Topology Contract

- [x] extend graph/runtime-facing topology DTOs with clearer bus-group, send/return, and console ownership semantics
- [x] pin deterministic routing summaries and validation rules for track lanes, returns, and main/aux output paths

### Batch 1.2 - Routed Execution Proof

- [x] exercise fan-in, fan-out, send, and return style routed fixtures through `signal-graph`
- [x] export one runtime-facing mixer topology summary that later diagnostics and metering work can reuse

## Acceptance Criteria

- [x] routed mixer topology is explicit and reusable across Signal-owned crates
- [x] graph/runtime tests prove deterministic send/return and bus execution order
- [x] later metering and automation tasks can depend on one stable topology seam

## Risks and Mitigations

- Risk: hosts continue to own routing policy implicitly.
- Mitigation: freeze routed mixer semantics in graph/runtime DTOs and tests.

## Evidence Requirements

- [x] log the contract-opening and routed-execution tranche under `docs/logs/YYYY-MM/`
- [x] run focused `signal-graph` and `signal-runtime` validation for the new routed mixer fixtures
- [x] record any remaining topology gaps that must stay deferred to later milestones

## Next Task

`g03.001` is complete. Execute `g03.002` and build routed metering plus
diagnostics export on top of the now-explicit mixer-topology seam.
