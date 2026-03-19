# 048 Post-Warp Render, Cache, And Transform-Artifact Contract

Status: complete
Owner: core-product
Updated: 2026-03-18
Related contracts: `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`, `docs/contracts/047-warp-marker-transient-anchor-and-tempo-assist-analysis-contract.md`, `docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first runtime-owned post-warp render, cache, and transform-artifact
boundary for `g07.017` so later artifact reuse, preview, and audition work
deepens one shared Signal vocabulary instead of reopening host-local render
cache policy, private preview artifacts, or product-specific transform stores.

## Authority hierarchy

Post-warp render and transform-artifact depth has one authority chain:

1. source media files, decode libraries, and media-cache artifacts provide raw
   audio bytes, duration, sample-rate, channel-layout, and decode success or
   failure evidence
2. `docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md`
   remains the authority for runtime-owned media identity, indexing,
   invalidation, waveform readiness, preview readiness, and analysis-ready
   service meaning
3. `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`
   remains the authority for:
   - stretch-engine class, readiness, degraded state, and fallback posture
   - render, preview, and export-facing stretch receipts
   - the rule that later artifact work must widen from one shared stretch
     substrate instead of inventing a second transform authority
4. `docs/contracts/047-warp-marker-transient-anchor-and-tempo-assist-analysis-contract.md`
   remains the authority for:
   - warp-marker, transient-anchor, and tempo-assist posture
   - analysis readiness, invalidation, and bounded transform-analysis meaning
   - the rule that later cache or audition work must widen from one shared
     marker-analysis seam instead of reopening host-local heuristics
5. `signal-runtime` must own the canonical consumer-visible meaning for:
   - post-warp render artifact identity
   - transform-artifact readiness, invalidation, reuse, and degraded posture
   - preview-facing, render-facing, and export-facing artifact receipts
   - observation, supervisor, and stable host-edge export
6. future cache engines, preview services, or storage policies may deepen raw
   evidence, but they must not become the authority for:
   - a second artifact taxonomy detached from runtime DTOs
   - host-local artifact reuse heuristics as the consumer boundary
   - product-local cache browsers or preview state as the transform truth

If a transform-artifact claim cannot be explained through the closed media,
stretch, and marker-analysis contracts plus runtime-owned receipts, it is not
yet part of the reusable Signal contract.

## Existing anchors

Batch 17.1 freezes this contract on top of the current bounded render and
analysis surface family:

- `RuntimeClipProcessingSnapshot`
- `RuntimeWarpClipSnapshot`
- `RuntimeStretchEngineSnapshot`
- `RuntimeMarkerAnalysisSnapshot`
- `RuntimeClipRenderResult`
- `RuntimeOfflineRenderContractPreview`
- `RuntimeMediaPipelineSnapshot`
- `RuntimeMediaServiceSnapshot`
- `RuntimeObservationReport`
- `RuntimeSupervisorReport`
- stable host-edge `supervisor_report()` export

Batch 17.1 does not claim those anchors already expose true transform-artifact
identity or reuse semantics. It freezes how later DTOs and proofs must widen
from them instead of inventing a separate host-private cache shell.

## Shared vocabulary

### Post-warp render artifact

`post-warp render artifact` means the runtime-owned transformed result or
cached transform evidence that corresponds to a bounded media identity, warp
intent, stretch posture, and analysis input set after warp processing has been
resolved.

This is not a host-local scratch file, not a product-local preview blob, and
not a private export-only cache.

### Transform-artifact identity

`transform-artifact identity` means the bounded runtime-owned descriptor that
ties one artifact candidate to the media asset, clip scope, transform posture,
and invalidation inputs that produced it.

Identity must stay typed and runtime-owned instead of being reconstructed from
cache-path strings or product-local naming policy.

### Transform-artifact readiness

`transform-artifact readiness` means whether the runtime-owned artifact path is
currently ready for downstream consumers to trust for reuse, preview, or
export.

Readiness must remain explicit and runtime-owned instead of inferred later from
file presence or UI-local warm-cache assumptions.

### Transform-artifact invalidation

`transform-artifact invalidation` means the runtime-owned fact that a cached or
candidate artifact can no longer be trusted because upstream media identity,
stretch posture, marker-analysis posture, or render scope changed.

Invalidation must stay typed and explainable through the closed stretch and
marker-analysis seams.

### Transform-artifact reuse

`transform-artifact reuse` means the bounded runtime-owned answer for whether a
consumer is observing an artifact candidate that can be reused as-is, must be
rebuilt, or is unavailable for the requested scope.

Reuse must not collapse into host-local preview-cache heuristics.

### Transform-artifact degraded posture

`transform-artifact degraded posture` means the runtime-owned answer for why
the promoted artifact path cannot currently provide the ideal reuse or render
behavior, while still allowing bounded fallback classification.

## Rules

### Rule 1: artifact work must widen from the closed stretch and analysis seams

`g07.017` must deepen the existing clip-processing, stretch-engine, and
marker-analysis surfaces from `046` and `047`. It must not create a second
preview or cache engine detached from runtime-owned transform truth.

### Rule 2: artifact identity, readiness, and invalidation must stay runtime-owned

Shared consumers must not infer artifact reuse from cache files, export paths,
or product-local preview state.

### Rule 3: preview, render, and export must share one artifact vocabulary

Artifact meaning may differ by scope, but the bounded vocabulary for identity,
readiness, invalidation, reuse, and degraded posture must stay shared across
preview, post-warp render, offline export, and later audition work.

### Rule 4: no product-local cache UX is implied

This contract may freeze artifact meaning and reuse posture, but it does not
freeze cache-browsing UX, prefetch policy, or product-specific storage
workflow.

### Rule 5: invalidation must compose with stretch and marker-analysis truth

If an artifact becomes stale because media, stretch readiness, or
marker-analysis posture changes, the invalidation answer must remain
runtime-owned and explainable through the closed upstream contracts.

### Rule 6: later preview and audition work must widen from this boundary

Future `g07.018` work must reuse this transform-artifact boundary instead of
inventing new preview-local or audition-local cache authorities.

## Deferred scope

Batch 17.1 intentionally does not claim:

- a realized runtime transform-artifact cache yet
- broad cache eviction, archival, or retention policy
- low-latency audition, scrub, or preview-transform execution behavior
- product-local cache browser or artifact management UX
- exhaustive storage-backend breadth
- final export packaging or publication policy

Those belong to later `g07.017`, `g07.018`, and `g07.020` batches.

## Batch 17.1 outcome

Batch 17.1 freezes the first bounded transform-artifact contract:

- Signal now has one explicit runtime-owned target for post-warp render
  artifact identity, readiness, invalidation, reuse, and degraded posture
  instead of host-local preview caches or private export scratch stores
- the authority line is explicit: media identity, stretch-engine truth, and
  marker-analysis truth remain the anchors, which prevents later artifact work
  from reopening a second transform-cache shell
- Batch 17.2 can now focus on materializing the first credible runtime-owned
  transform-artifact receipt family instead of reopening what artifact meaning
  belongs to Signal

## Batch 17.2 outcome

Batch 17.2 materializes the first bounded runtime-owned transform-artifact
receipt family:

- `signal-runtime` now owns typed transform-artifact readiness, invalidation,
  reuse, cached-media readiness, and per-clip identity instead of leaving
  post-warp artifact posture implicit in preview or export code
- the same receipt family now flows through runtime observation, supervisor
  export, clip-render results, offline-render preview, and stable host-edge
  JSON instead of splitting artifact meaning across separate render and host
  surfaces
- artifact posture now composes directly with the closed media, stretch, and
  marker-analysis seams, which keeps later cache or audition depth additive on
  one shared substrate

Batch 17.2 intentionally does not claim a full cache engine, low-latency
audition path, or broader artifact-retention policy. Those remain later work.

## Batch 17.3 outcome

Batch 17.3 closes the downstream-style proof seam for the bounded
transform-artifact contract:

- public runtime proof now shows `RuntimeTransformArtifactSnapshot` remains
  consumable through shared runtime observation, supervisor, clip-render, and
  offline-preview surfaces without host-local preview-cache reconstruction
- both stable host edges now prove they forward the same runtime-owned
  transform-artifact readiness, invalidation, and reuse receipts instead of
  rebuilding host-specific cache heuristics
- `signal-supervisor-tools` now exposes
  `signal.runtime.transform-artifact-boundary`, and Effigy now owns
  `acceptance:transform-artifact-boundary` as the repo-owned rerun lane

This closes the bounded `g07.017` contract seam while keeping fuller cache
retention, low-latency audition, and richer storage-policy depth explicitly
deferred.

## Next Task

Continue `g08.001` with Batch 1.1 by freezing the runtime-owned live Linux
audio backend ownership and session-lifecycle contract before deeper ALSA,
JACK, and PipeWire runtime realization widens.
