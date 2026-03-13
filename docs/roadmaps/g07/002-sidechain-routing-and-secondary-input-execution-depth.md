# 002 - Sidechain Routing And Secondary-Input Execution Depth

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g07.001
Vision tags: `ROUTING`, `PLUGINS`, `EXECUTION`

## Problem

Chorus already treats sidechain routing as first-class authority intent, but
Signal still needs one runtime-owned meaning for secondary inputs, routing
identity, and failure or fallback behavior.

## Goals

- [ ] define runtime-owned sidechain and secondary-input semantics
- [ ] support sidechain-capable routing without host-local reconstruction
- [ ] keep plugin, graph, and render surfaces aligned on one secondary-input model

## Non-Goals

- [ ] no product-specific sidechain UX yet
- [ ] no broad multi-bus expansion beyond what this contract needs

## Execution Plan

### Batch 2.1 - Secondary-Input Contract

- [ ] define sidechain source, target, and fallback meaning
- [ ] align authority routing intent with runtime execution surfaces

### Batch 2.2 - Runtime Execution Depth

- [ ] implement secondary-input execution and runtime observation depth
- [ ] keep plugin and render paths aligned with the new sidechain meaning

### Batch 2.3 - Focused Proof

- [ ] add focused proofs for sidechain routing, fallback, and failure behavior

## Acceptance Criteria

- [ ] Signal has explicit secondary-input and sidechain execution semantics
- [ ] later complex plugin-I/O and spatial work can rely on the same routing truth
- [ ] hosts observe sidechain state without inventing separate models

## Risks And Mitigations

- Risk: sidechain meaning forks between live and offline execution.
- Mitigation: require one runtime-owned contract across both paths.

## Evidence Requirements

- [ ] log each meaningful sidechain tranche
- [ ] run focused routing and execution validation
- [ ] record deferred sidechain breadth explicitly

## Next Task

Continue `g07.003` by widening the same routing contract into multi-bus and
auxiliary topology depth.

