# 018 - Analysis Metadata Extraction And Library-Service Depth

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g06.017
Vision tags: `MEDIA`, `ANALYSIS`, `LIBRARY`

## Problem

Waveform and preview services are necessary, but Loophole and future consumers
also need richer reusable asset metadata and analysis services for browsing,
placement, preview, and later intelligence workflows.

## Goals

- [ ] define reusable analysis metadata and library-service semantics
- [ ] expose typed asset descriptors beyond simple waveform/preview readiness
- [ ] support later product workflows and advisory intelligence without
  reimplementing analysis services locally

## Non-Goals

- [ ] no product-local tagging UX or recommendation interface
- [ ] no broad ML/classification generation here

## Execution Plan

### Batch 18.1 - Metadata Contract

- [x] define the first reusable asset-metadata and analysis-service descriptor family
- [x] align metadata ownership with the earlier media-service baseline

### Batch 18.2 - Service Depth

- [x] materialize the chosen metadata and library-service outputs in
  Signal-owned crates and exports
- [x] keep runtime, supervisor, and host-edge surfaces on the same typed descriptors

### Batch 18.3 - Consumer Proof

- [x] add focused proofs that downstream consumers can rely on analysis metadata
  without product-local extraction pipelines

## Acceptance Criteria

- [x] Signal has reusable analysis metadata and library-service depth
- [x] later products can consume asset-analysis descriptors through Signal-owned surfaces
- [x] advisory feature work has stronger runtime/media inputs to build on

## Risks And Mitigations

- Risk: metadata extraction scope balloons into product intelligence features.
- Mitigation: keep the milestone on reusable descriptors and services only.
- Risk: metadata semantics drift from waveform/preview service state.
- Mitigation: require explicit alignment with `g06.017` readiness and invalidation.

## Evidence Requirements

- [x] log each meaningful analysis-metadata tranche
- [x] run focused validation for library-service descriptors
- [x] record deferred intelligence breadth explicitly

## Batch 18.1 Outcome

Batch 18.1 freezes the first reusable analysis-metadata and library-service
contract in
`docs/contracts/029-analysis-metadata-extraction-and-library-service-contract.md`.
That contract now fixes the authority line between source or decode evidence,
the shared `signal-analysis*` crates, and `signal-runtime` for reusable asset
descriptors, bounded analysis-family coverage, ready versus stale metadata, and
library-service meaning. It also makes metadata ownership explicitly depend on
the closed `g06.017` media indexing, waveform, preview, and invalidation seam
before Batch 18.2 widens any DTO or export surfaces.

## Batch 18.2 Outcome

Batch 18.2 turns that frozen contract into a real runtime-owned descriptor
family. `signal-runtime` now exposes bounded reusable analysis metadata through
`RuntimeMediaLibraryServiceSnapshot`, `RuntimeMediaLibraryAssetDescriptor`,
`RuntimeMediaLoudnessDescriptor`, and `RuntimeMediaCharacterDescriptor`, with
runtime-owned readiness on top of the already closed media indexing,
waveform, preview, and invalidation seam. The first real payload depth is
loudness plus character analysis; rhythm, tonal, and embedding coverage stay
explicitly deferred instead of disappearing into product-local metadata gaps.

The same descriptor family now flows through `RuntimeObservationReport`,
`RuntimeSupervisorReport`, and the shared local/server host report surfaces.
That keeps ready, invalidated, and unavailable library-service outcomes on one
typed Signal-owned model rather than leaving downstream products to rebuild
analysis availability from raw files or private caches.

## Batch 18.3 Outcome

Batch 18.3 closes the bounded reusable analysis-metadata consumer seam. Public
runtime proofs now show that downstream consumers can inspect
`RuntimeMediaLibraryServiceSnapshot` and its per-asset descriptor family through
runtime reexports alone, while both stable host edges prove the same ready,
invalidated, and deferred-family truth is preserved through
`supervisor_report()` without product-local extraction or metadata
reconstruction.

The machine-readable boundary is now explicit through
`signal.runtime.analysis-metadata-boundary` in `signal-supervisor-tools`, with
the repo-owned acceptance task `effigy acceptance:analysis-metadata-boundary`
anchoring the shared validation path. That closes `g06.018` as the bounded
analysis-metadata and library-service descriptor milestone and moves the active
queue to `g06.019`.

## Next Task

Continue `g06.019` with Batch 19.1 by freezing the shared fault-injection
harness and multi-backend acceptance contract, separating required integrated
acceptance evidence from optional longer-running soak depth.
