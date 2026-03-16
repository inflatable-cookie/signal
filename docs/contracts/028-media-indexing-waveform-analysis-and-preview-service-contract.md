# 028 Media Indexing, Waveform Analysis, And Preview Service Contract

Status: complete
Owner: core-product
Updated: 2026-03-16
Related contracts: `docs/contracts/005-runtime-work-orchestration-and-deferred-service-policy.md`, `docs/contracts/019-deferred-work-scheduler-priority-backpressure-and-cancellation-contract.md`, `docs/contracts/027-external-io-monitoring-tap-point-and-loopback-measurement-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`, `docs/architecture/package-map.md`

## Purpose

Freeze the first runtime-owned media-service boundary for asset indexing,
waveform readiness, preview readiness, and invalidation so later Signal media
work builds on one reusable service model instead of product-local cache,
preview, or waveform pipelines.

## Authority hierarchy

Media indexing, waveform, and preview meaning have one authority chain:

1. source files and decode libraries provide raw media bytes, file metadata,
   and decode success or failure evidence
2. `signal-analysis` and the `signal-analysis-*` family own reusable analysis
   algorithms, result types, and confidence models
3. `signal-runtime` owns canonical consumer-visible service meaning for:
   - media asset identity and registration
   - whether an asset is ingesting, conforming, ready, invalid, or rebuilding
   - whether waveform output is pending or ready
   - whether preview is unavailable, ready, previewing, or invalidated
   - which asset was last invalidated or previewing and what bounded error
     state remains relevant to consumers
4. hosts and downstream products may broker files, asset roots, or UI actions
   into runtime-owned services, but they must not become the authority for:
   - a competing media asset lifecycle taxonomy
   - a product-local preview readiness model
   - private waveform cache semantics that bypass runtime-owned receipts

If a media indexing, waveform, or preview claim cannot be explained through
shared analysis crates plus `signal-runtime` receipts, it is not yet part of
the reusable Signal boundary.

## Existing runtime anchors

This contract is grounded in the current runtime-owned media seam:

- `RuntimeMediaAssetRegistration`
- `RuntimeMediaAssetSnapshot`
- `RuntimeMediaPipelineSnapshot`
- `RuntimeMediaServiceSnapshot`
- `RuntimeMediaAssetState`
- `RuntimeMediaIndexingState`
- `RuntimeMediaPreviewState`
- `RuntimeObservationApi::get_media_pipeline_snapshot()`
- `RuntimeObservationApi::get_media_service_snapshot()`
- `SignalRuntime::reconcile_media_assets(...)`
- `SignalRuntime::start_media_preview(...)`
- `SignalRuntime::stop_media_preview(...)`
- the `RuntimeMediaPipelineStateModel` and current invalidation or preview
  reconciliation logic in `signal-runtime`
- `signal-analysis`
- `signal-analysis-rhythm`
- `signal-analysis-tonal`
- `signal-analysis-loudness`
- `signal-analysis-character`
- `signal-analysis-embed`

Batch 17.1 does not claim these anchors already expose the full reusable
service boundary. It freezes how later DTO widening and service proofs must
compose from them.

## Shared vocabulary

### Media asset

`media asset` means one runtime-owned registered source item identified by a
stable `asset_id` and `content_hash`, along with bounded format and duration
metadata needed for reusable indexing, waveform, and preview service work.

This is not a product-local library row, playlist entry, or browser UX model.

### Indexing

`indexing` means the runtime-owned service process that reconciles registered
assets into the current Signal cache and readiness model.

Indexing covers ingesting, conforming, rebuilding, invalidation, and ready
state. It does not imply a full catalog database or a product-local content
management workflow.

### Waveform readiness

`waveform readiness` means whether runtime-owned media service state claims the
bounded waveform representation requested by the asset registration is ready,
pending, or not presently valid.

This contract freezes readiness meaning, not the final display format or a
product-specific waveform rendering style.

### Preview readiness

`preview readiness` means whether Signal can currently preview a registered
asset through the runtime-owned service boundary.

Preview readiness must stay tied to runtime-owned indexing and invalidation
state rather than a host-local playback heuristic.

### Invalidation

`invalidation` means the runtime-owned state transition where a previously
indexed asset or preview-capable asset is no longer valid because its cache,
decode result, or content identity is no longer trusted.

Invalidation must remain typed and inspectable rather than hidden behind
product-local cache misses or preview failures.

### Analysis readiness

`analysis readiness` means whether the runtime-owned media service boundary
considers an asset ready for downstream analysis or preview work using the
shared analysis crates.

This contract does not promise every analysis family is materialized yet. It
freezes the service meaning that a shared analysis-ready asset exists as a
runtime-owned concept.

## Rules

### Rule 1: runtime owns media asset lifecycle meaning

Products and hosts must not define their own competing lifecycle vocabulary
for reusable media services. Shared consumer meaning must come from:

- `RuntimeMediaAssetState`
- `RuntimeMediaPipelineSnapshot`
- `RuntimeMediaServiceSnapshot`

### Rule 2: waveform and preview readiness stay bounded and typed

Consumers must not need to infer waveform or preview readiness from cache-path
existence, decode logs, or product-local playback state.

If Signal claims a waveform or preview service boundary, readiness must stay
typed and runtime-owned.

### Rule 3: invalidation is canonical runtime state, not just an error side effect

When preview or cached media becomes unusable, the consumer-facing truth must
stay visible through runtime-owned invalidation fields instead of only through
opaque error strings or failed file opens.

### Rule 4: shared analysis crates provide algorithms, runtime provides service state

The `signal-analysis*` crates own algorithm families and result shapes.
`signal-runtime` owns reusable service orchestration and readiness meaning for
media assets inside the Signal boundary.

Neither side should absorb the other’s responsibility.

### Rule 5: later library and media workflows must deepen this contract

Later media indexing, library-service, metadata extraction, waveform, preview,
or analysis-depth milestones may widen DTOs and execution behavior, but they
must build on this contract rather than reintroducing app-local media caches or
private preview readiness taxonomies.

## Deferred scope

Batch 17.1 intentionally keeps the following outside the shared contract:

- product-local media browser, playlist, tagging, or collection UX
- editorial asset management and remote catalog sync
- ML-driven ranking, recommendation, or semantic search behavior
- lossless preview rendering guarantees across every future media format
- final waveform visualization format or UI styling
- publication-grade library import/export workflows

These may later gain additive Signal-owned surfaces, but they are not promised
by Batch 17.1.

## Batch 17.1 outcome

Batch 17.1 freezes the first reusable media-service contract:

- Signal now has one authority line for media asset identity, indexing,
  invalidation, waveform readiness, preview readiness, and analysis-ready
  service meaning
- the split between shared analysis crates and runtime-owned service state is
  explicit instead of implied
- later `g06.017` and `g06.018` work can widen one bounded service seam rather
  than growing new product-local preview or waveform models

## Batch 17.2 outcome

Batch 17.2 widens the frozen contract into shared runtime observation and
export surfaces:

- `RuntimeObservationReport` now carries `media_pipeline_snapshot` and
  `media_service_snapshot`
- `RuntimeSupervisorReport` and the shared local/server `supervisor_report()`
  paths now expose the same runtime-owned media readiness, invalidation, and
  preview state
- media indexing and preview readiness therefore survive as Signal-owned
  receipts in the shared report/export seam instead of only as direct runtime
  API calls

## Batch 17.3 outcome

Batch 17.3 closes the bounded media-service consumer seam:

- public runtime proofs now cover runtime-owned media indexing, waveform
  readiness, preview state, and invalidation receipts directly from shared
  runtime reexports
- both stable host edges now prove the same media-service receipt family
  survives through `supervisor_report()` without product-local reconstruction
- `signal-supervisor-tools` now exposes the machine-readable
  `signal.runtime.media-service-boundary` descriptor and the repo-owned
  `effigy acceptance:media-service-boundary` task

This contract is now closed enough for later metadata and library-service
depth to build on it instead of reopening media readiness ownership.

## Next Task

Continue `g06.018` with Batch 18.1 by freezing the first reusable
analysis-metadata and library-service descriptor family on top of this closed
media-service boundary.
