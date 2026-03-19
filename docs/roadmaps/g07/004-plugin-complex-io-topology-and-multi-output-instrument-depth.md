# 004 - Plugin Complex I/O Topology And Multi-Output Instrument Depth

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g07.001, g07.003, g06.011
Vision tags: `PLUGINS`, `ROUTING`, `EXECUTION`

## Problem

Chorus already calls out complex plugin I/O such as sidechains and multi-output
instruments, but Signal still needs one backend-neutral runtime meaning for
those topologies before broader plugin depth becomes credible.

## Goals

- [ ] define backend-neutral complex plugin-I/O topology semantics
- [ ] support multi-output instruments and richer bus-capable FX behavior
- [ ] keep lifecycle, routing, and render surfaces aligned on one topology model

## Non-Goals

- [ ] no plugin-editor UX or pin-matrix product workflow here
- [ ] no adapter-private behavior promoted without contract meaning

## Execution Plan

### Batch 4.1 - Plugin-I/O Contract

- [x] define complex plugin I/O, multi-output, and sidechain-capable topology meaning
- [x] align adapter capability receipts with the widened topology contract

### Batch 4.2 - Runtime Adapter Depth

- [x] implement the first credible complex plugin-I/O runtime path
- [x] keep sandbox, lifecycle, and render semantics aligned to the same model

### Batch 4.3 - Focused Proof

- [x] add focused proofs for multi-output and sidechain-capable plugin behavior

## Acceptance Criteria

- [ ] Signal has explicit complex plugin-I/O topology semantics
- [ ] later adapter breadth can reuse one backend-neutral multi-I/O model
- [ ] hosts can observe plugin routing truth without inventing pin behavior locally

## Risks And Mitigations

- Risk: adapter-specific pin logic leaks into the shared contract.
- Mitigation: freeze backend-neutral topology meaning before adapter proofs widen.

## Evidence Requirements

- [ ] log each meaningful complex-I/O tranche
- [ ] run focused plugin topology validation
- [ ] record deferred adapter-private behavior explicitly

## Batch 4.1 Outcome

Batch 4.1 froze the backend-neutral complex plugin-I/O boundary in
`docs/contracts/035-plugin-complex-io-topology-and-multi-output-instrument-contract.md`.

Signal now has one bounded shared vocabulary for:

- plugin port class
- complex plugin-I/O topology
- multi-output instrument identity
- bus-capable FX class
- plugin-facing attachment policy and fallback outcome

That gives the next runtime batch one shared target for CLAP, VST3, and AU
adapter breadth instead of letting complex plugin bus behavior drift back into
format-private pin naming or host-local routing interpretation.

## Batch 4.2 Outcome

Batch 4.2 turned the frozen complex plugin-I/O contract into real runtime-owned
receipts across discovery, execution, and offline render surfaces.

Signal now carries one typed `complex_io_summary` family through:

- discovered plugin-type records and cross-format capability coverage
- plugin-chain stage snapshots and routed execution topology
- offline render dependency preview and complex stage summaries
- widened VST3 and AU adapter fixture catalogs that now expose multi-output
  instruments and bus-capable FX instead of only simple instrument or utility
  shapes

This closes the runtime realization tranche for bounded complex plugin-I/O
meaning. The remaining milestone work is consumer proof: public runtime,
supervisor, and stable host-edge surfaces still need to prove they expose the
same multi-output and bus-capable topology truth without adapter-local pin
reconstruction.

## Batch 4.3 Outcome

Batch 4.3 closed the bounded complex plugin-I/O consumer seam across public
runtime, both stable host edges, and a machine-readable supervisor-tools
descriptor.

Signal now proves that:

- complex plugin-I/O discovery and capability coverage remain consumable
  through public runtime reports
- multi-output instrument and bus-capable FX topology remain visible on live
  plugin-chain stage receipts
- the same topology remains visible on deferred render dependency preview
- local and server host edges forward the same runtime-owned complex plugin-I/O
  receipts without adapter-local pin reconstruction

This closes `g07.004` as a bounded runtime and consumer-proof milestone. Broader
spatial routing, immersive buses, and richer pin-matrix or mixer policy remain
later `g07` work rather than implied by this closure.

## Next Task

Continue `g07.006` with Batch 6.2 by materializing runtime-owned surround-bed,
object-role, mix-policy, render-scope, and expanded-fallback receipts across
execution, render, and observation surfaces without reopening host-local or
renderer-local spatial ownership.
