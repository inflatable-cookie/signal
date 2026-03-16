# 029 Analysis Metadata Extraction And Library-Service Contract

Status: complete
Owner: core-product
Updated: 2026-03-16
Related contracts: `docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md`, `docs/contracts/019-deferred-work-scheduler-priority-backpressure-and-cancellation-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`, `docs/architecture/dsp-analysis-feature-reference.md`

## Purpose

Freeze the first reusable Signal-owned analysis-metadata and library-service
boundary so later asset browsing, placement, preview, and advisory workflows
build on one typed descriptor family instead of product-local metadata caches,
private extraction pipelines, or host-specific media summaries.

## Authority hierarchy

Analysis metadata and library-service meaning have one authority chain:

1. source files, decode libraries, and cache artifacts provide raw bytes,
   media duration, channel layout, and decode success or failure evidence
2. `signal-analysis` and the `signal-analysis-*` family own reusable analysis
   algorithms, domain result types, and per-family confidence or summary logic
3. `signal-runtime` owns canonical consumer-visible service meaning for:
   - which registered asset a reusable analysis descriptor belongs to
   - whether analysis metadata is missing, pending, ready, stale, invalidated,
     or intentionally unavailable
   - which bounded analysis families are currently represented on the asset
     descriptor surface
   - which runtime-owned invalidation or rebuild state still constrains library
     and preview consumers
4. hosts and downstream products may request extraction, browse results, or UI
   grouping, but they must not become the authority for:
   - a competing reusable asset-metadata taxonomy
   - private readiness or staleness rules for shared analysis descriptors
   - product-local portability claims about which analysis families are
     available for a registered asset

If an analysis-metadata or library-service claim cannot be explained through
shared analysis crates plus runtime-owned asset and service receipts, it is
not yet part of the reusable Signal boundary.

## Existing anchors

This contract is grounded in the currently closed media-service and analysis
surface:

- `RuntimeMediaAssetRegistration`
- `RuntimeMediaAssetSnapshot`
- `RuntimeMediaPipelineSnapshot`
- `RuntimeMediaServiceSnapshot`
- `RuntimeMediaAssetState`
- `RuntimeMediaIndexingState`
- `RuntimeMediaPreviewState`
- `RuntimeObservationApi::get_media_pipeline_snapshot()`
- `RuntimeObservationApi::get_media_service_snapshot()`
- `RuntimeObservationReport`
- `RuntimeSupervisorReport`
- `signal-analysis`
- `signal-analysis-rhythm`
- `signal-analysis-tonal`
- `signal-analysis-loudness`
- `signal-analysis-character`
- `signal-analysis-embed`

Batch 18.1 does not claim those anchors already expose reusable metadata
descriptors. It freezes how later DTO widening and service proofs must compose
from them.

## Shared vocabulary

### Analysis metadata descriptor

`analysis metadata descriptor` means one runtime-owned, typed summary attached
to a registered media asset that exposes bounded reusable analysis meaning for
later browsing, placement, preview, or advisory workflows.

This is not a product-local library row, recommendation record, playlist
entry, or UI card.

### Library-service descriptor

`library-service descriptor` means the consumer-facing runtime-owned bundle of
asset identity, readiness, and bounded analysis metadata needed to build later
library and browse workflows on Signal services instead of app-local extractors.

The library-service descriptor is allowed to stay smaller than future product
needs. It still must remain typed and reusable.

### Analysis family coverage

`analysis family coverage` means which bounded reusable analysis families have
consumer-visible descriptor meaning for a registered asset.

Batch 18.1 only freezes the categories, not the full future payload depth:

- timing or rhythm-facing descriptors
- tonal or pitch-facing descriptors
- loudness or dynamics-facing descriptors
- character or timbre-facing descriptors
- embedding or similarity-facing descriptors

Coverage is additive. A family may be explicitly unavailable or deferred
without invalidating the rest of the descriptor surface.

### Ready metadata

`ready metadata` means the runtime-owned library-service boundary considers the
currently registered asset eligible to expose a bounded analysis descriptor
without contradicting media indexing or invalidation truth.

### Stale metadata

`stale metadata` means previously extracted reusable analysis meaning is no
longer trusted because asset identity, decode evidence, or invalidation state
changed.

Staleness must remain runtime-owned and inspectable rather than inferred later
from cache misses or product-local timestamp heuristics.

### Advisory analysis

`advisory analysis` means reusable metadata that may guide browsing, preview,
placement, or later intelligence features but is not itself a canonical media
identity field.

Advisory does not mean optional host-local reconstruction. It means the shared
result should not be mistaken for immutable asset identity.

## Rules

### Rule 1: runtime owns consumer-facing analysis descriptor meaning

Shared consumers must not need product-local extraction pipelines to understand
whether reusable analysis metadata is pending, ready, stale, or unavailable.

### Rule 2: metadata must align with media readiness and invalidation

Reusable analysis descriptors must compose from the closed `g06.017`
media-service seam. A descriptor cannot remain ready if runtime-owned media
identity or invalidation receipts no longer trust the underlying asset.

### Rule 3: analysis crates own algorithms, runtime owns service-state meaning

The `signal-analysis*` crates own algorithm families and result shapes.
`signal-runtime` owns reusable consumer-facing descriptor identity, readiness,
staleness, and library-service orchestration.

### Rule 4: library-service descriptors are not product browser models

This boundary may support later library or browse workflows, but it does not
freeze collection UX, tagging UX, playlist semantics, recommendation models, or
editorial product structure.

### Rule 5: analysis family coverage must stay explicit

Consumers must be able to distinguish:

- ready reusable metadata
- pending extraction depth
- stale or invalidated metadata
- intentionally unsupported or deferred analysis families

Those distinctions must stay typed and Signal-owned rather than reconstructed
from missing cache files or absent product database columns.

### Rule 6: later intelligence breadth must deepen this contract additively

Later metadata, similarity, library, or advisory-feature milestones may widen
descriptor payloads and extraction depth, but they must build on this contract
instead of inventing new product-local metadata authorities.

## Deferred scope

Batch 18.1 intentionally keeps the following outside the shared contract:

- product-local tagging, collection, playlist, or browser UX
- recommendation ranking, search ranking, or editorial curation policy
- remote catalog sync and publication-grade library import/export
- full-corpus indexing guarantees across every future media family
- ML classification breadth beyond bounded reusable analysis descriptors
- product-local intelligence prompts, summaries, or narrative content

These may later gain additive Signal-owned surfaces, but they are not promised
by Batch 18.1.

## Batch 18.1 outcome

Batch 18.1 freezes the first reusable analysis-metadata and library-service
contract:

- Signal now has one authority line for reusable asset-analysis descriptor
  meaning, readiness, staleness, and bounded analysis-family coverage
- metadata ownership is explicitly aligned to the closed `g06.017`
  media-service boundary instead of floating separately from indexing and
  invalidation truth
- later `g06.018` work can widen one bounded descriptor family rather than
  growing product-local extraction or library metadata models again

## Batch 18.2 outcome

Batch 18.2 materializes the first real runtime-owned descriptor family on top
of this contract:

- `signal-runtime` now owns `RuntimeMediaLibraryServiceSnapshot` plus
  per-asset `RuntimeMediaLibraryAssetDescriptor` records that stay aligned with
  media indexing, preview readiness, and invalidation truth
- the first bounded reusable payload depth is now explicit:
  `RuntimeMediaLoudnessDescriptor` and `RuntimeMediaCharacterDescriptor`
- loudness and character families can be `Ready`, while rhythm, tonal, and
  embedding remain explicitly `Deferred` instead of silently absent
- invalidated and unavailable metadata are now runtime-owned receipt outcomes,
  including the server-host path where indexed media may still remain
  non-analyzable
- the same descriptor family now flows through runtime observation, supervisor
  export, and both shared host report surfaces without reopening host-local or
  product-local metadata ownership

## Batch 18.3 outcome

Batch 18.3 closes the bounded consumer-facing proof seam for this contract:

- downstream-style runtime proofs now show that reusable analysis metadata and
  library-service descriptors remain consumable through public runtime surfaces
  without product-local extraction or metadata reconstruction
- both stable host edges now prove they forward the same runtime-owned
  descriptor family, including ready, invalidated, unavailable, and
  explicitly deferred analysis-family coverage
- the machine-readable boundary `signal.runtime.analysis-metadata-boundary`
  now exists in `signal-supervisor-tools`, with
  `effigy acceptance:analysis-metadata-boundary` as the repo-owned validation
  seam for this shared descriptor family
- this contract is therefore closed as the first reusable analysis-metadata and
  library-service boundary, while richer rhythm, tonal, embedding, and
  product-local browse or recommendation breadth remain deferred

## Next Task

Continue `g06.019` with Batch 19.1 by freezing the shared fault-injection
harness and multi-backend acceptance contract, separating required integrated
acceptance evidence from optional longer-running soak depth.
