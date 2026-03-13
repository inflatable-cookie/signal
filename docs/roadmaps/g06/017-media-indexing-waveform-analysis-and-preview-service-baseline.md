# 017 - Media Indexing, Waveform Analysis, And Preview Service Baseline

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g06.008
Vision tags: `MEDIA`, `ANALYSIS`, `SERVICES`

## Problem

Loophole still needs reusable waveform, preview, and asset-analysis services,
but Signal does not yet have a deliberate media-service baseline beyond the
current runtime and analysis building blocks.

## Goals

- [ ] define runtime-owned media indexing, waveform, and preview service semantics
- [ ] expose asset readiness, invalidation, and rebuild state through reusable
  Signal-owned surfaces
- [ ] avoid per-product preview or waveform pipelines diverging again

## Non-Goals

- [ ] no product-local media browser UX
- [ ] no ML-driven asset intelligence breadth yet

## Execution Plan

### Batch 17.1 - Media-Service Contract

- [ ] define indexing, waveform-analysis, preview, and invalidation semantics
- [ ] align media identity, analysis output, and rebuild state at the boundary

### Batch 17.2 - Service Baseline

- [ ] materialize media indexing, waveform, and preview services in Signal-owned
  crates and exports
- [ ] keep host-edge and supervisor surfaces aligned to the same readiness model

### Batch 17.3 - Focused Proof

- [ ] add focused proofs showing asset-analysis and preview services remain
  consumable without product-local reconstruction

## Acceptance Criteria

- [ ] Signal has a real media indexing/waveform/preview baseline
- [ ] products can observe asset readiness and invalidation through reusable surfaces
- [ ] later media workflows can build on Signal services instead of local shims

## Risks And Mitigations

- Risk: media-service work drifts into product content-management scope.
- Mitigation: keep the milestone on reusable analysis/readiness services only.
- Risk: preview outputs become opaque cache artifacts only.
- Mitigation: require typed readiness and invalidation receipts.

## Evidence Requirements

- [ ] log each meaningful media-service tranche
- [ ] run focused validation for waveform and preview services
- [ ] record deferred content-management breadth explicitly

## Next Task

Continue `g06.018` by adding richer analysis and metadata extraction on top of
the new media-service substrate.
