# 004 - Plugin Complex I/O Topology And Multi-Output Instrument Depth

Status: planned
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

- [ ] define complex plugin I/O, multi-output, and sidechain-capable topology meaning
- [ ] align adapter capability receipts with the widened topology contract

### Batch 4.2 - Runtime Adapter Depth

- [ ] implement the first credible complex plugin-I/O runtime path
- [ ] keep sandbox, lifecycle, and render semantics aligned to the same model

### Batch 4.3 - Focused Proof

- [ ] add focused proofs for multi-output and sidechain-capable plugin behavior

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

## Next Task

Continue `g07.005` by making spatial execution build on the newly explicit
routing and multichannel substrate.

