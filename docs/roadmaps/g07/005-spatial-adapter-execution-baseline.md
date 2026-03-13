# 005 - Spatial Adapter Execution Baseline

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g07.001, g07.003
Vision tags: `SPATIAL`, `MULTICHANNEL`, `EXECUTION`

## Problem

Chorus already defines spatial adapter intent, but Signal still needs a first
credible runtime-owned execution baseline for those adapters.

## Goals

- [ ] implement the first real spatial adapter execution baseline in Signal
- [ ] keep spatial behavior aligned with multichannel and routing substrate
- [ ] expose host-visible runtime spatial state without host-local reinterpretation

## Non-Goals

- [ ] no product-specific spatial UI variants
- [ ] no full object-audio ecosystem breadth yet

## Execution Plan

### Batch 5.1 - Spatial Execution Contract

- [ ] align existing spatial-adapter semantics with runtime execution meaning
- [ ] define fallback behavior for unsupported layouts and adapters

### Batch 5.2 - Runtime Baseline

- [ ] implement the first credible spatial adapter path
- [ ] expose runtime-owned spatial observation and diagnostics

### Batch 5.3 - Focused Proof

- [ ] add focused proofs for spatial execution and fallback behavior

## Acceptance Criteria

- [ ] Signal has a real spatial adapter execution baseline
- [ ] spatial behavior stays aligned with multichannel and routing truth
- [ ] hosts observe spatial state through one reusable runtime vocabulary

## Risks And Mitigations

- Risk: spatial semantics stay model-only and never become executable.
- Mitigation: require runtime proof and fallback behavior before expansion.

## Evidence Requirements

- [ ] log each meaningful spatial tranche
- [ ] run focused spatial execution validation
- [ ] record deferred spatial breadth explicitly

## Next Task

Continue `g07.006` by widening from the baseline adapter path into surround
bed, object, and mix-policy expansion.

