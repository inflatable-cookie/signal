# 017 - Media Indexing, Waveform Analysis, And Preview Service Baseline

Status: complete
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

- [x] define indexing, waveform-analysis, preview, and invalidation semantics
- [x] align media identity, analysis output, and rebuild state at the boundary

### Batch 17.2 - Service Baseline

- [x] materialize media indexing, waveform, and preview services in Signal-owned
  crates and exports
- [x] keep host-edge and supervisor surfaces aligned to the same readiness model

### Batch 17.3 - Focused Proof

- [x] add focused proofs showing asset-analysis and preview services remain
  consumable without product-local reconstruction

## Acceptance Criteria

- [x] Signal has a real media indexing/waveform/preview baseline
- [x] products can observe asset readiness and invalidation through reusable surfaces
- [x] later media workflows can build on Signal services instead of local shims

## Risks And Mitigations

- Risk: media-service work drifts into product content-management scope.
- Mitigation: keep the milestone on reusable analysis/readiness services only.
- Risk: preview outputs become opaque cache artifacts only.
- Mitigation: require typed readiness and invalidation receipts.

## Evidence Requirements

- [x] log each meaningful media-service tranche
- [x] run focused validation for waveform and preview services
- [x] record deferred content-management breadth explicitly

## Batch 17.1 Outcome

Batch 17.1 froze the first runtime-owned media-service contract in
`docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md`.
That contract now fixes the authority line between source files and decode
evidence, the shared `signal-analysis*` crates, and `signal-runtime` service
state for media asset identity, indexing, invalidation, waveform readiness,
preview readiness, and analysis-ready service meaning. It also makes the split
between analysis algorithms and runtime-owned service orchestration explicit
before Batch 17.2 widens any DTO or export surfaces.

## Batch 17.2 Outcome

Batch 17.2 turned that frozen media-service boundary into a real shared runtime
surface. `signal-runtime` now carries `media_pipeline_snapshot` and
`media_service_snapshot` directly on `RuntimeObservationReport`, and the same
runtime-owned indexing, invalidation, waveform, and preview truth now flows
through `RuntimeSupervisorReport` plus the shared local and server
`supervisor_report()` render paths. This keeps media readiness and preview
state inside one Signal-owned observation/export seam instead of leaving the
media pipeline as a direct-API-only subsystem.

## Batch 17.3 Outcome

Batch 17.3 closes the media-service seam as a real shared consumer boundary.
Public runtime proofs, stable local and server host-edge proofs, the new
`signal.runtime.media-service-boundary` descriptor in
`signal-supervisor-tools`, and `effigy acceptance:media-service-boundary`
now prove that media indexing, waveform readiness, preview state, and
invalidation receipts remain consumable without product-local reconstruction.
That closes `g06.017` and moves the active queue to `g06.018`.

## Next Task

Continue `g06.018` with Batch 18.1 by freezing the first reusable
analysis-metadata and library-service descriptor family on top of the closed
media-service boundary.
