# 047 Warp-Marker, Transient-Anchor, And Tempo-Assist Analysis Contract

Status: complete
Owner: core-product
Updated: 2026-03-18
Related contracts: `docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md`, `docs/contracts/029-analysis-metadata-extraction-and-library-service-contract.md`, `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first runtime-owned warp-marker, transient-anchor, and tempo-assist
analysis boundary for `g07.016` so later stretch-assist, artifact, and preview
work deepens one shared Signal vocabulary instead of reopening host-local warp
analysis tools, private editor heuristics, or product-specific marker shells.

## Authority hierarchy

Warp-marker and transient analysis have one authority chain:

1. source media files, decode libraries, and cache artifacts provide raw audio
   bytes, sample-rate, duration, and decode success or failure evidence
2. `docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md`
   remains the authority for runtime-owned media identity, indexing,
   invalidation, waveform readiness, preview readiness, and analysis-ready
   service meaning
3. `docs/contracts/029-analysis-metadata-extraction-and-library-service-contract.md`
   remains the authority for reusable bounded analysis-descriptor readiness and
   invalidation meaning whenever later marker or tempo-assist depth composes
   with broader descriptor families
4. `docs/contracts/046-sample-domain-time-stretch-engine-contract.md` remains
   the authority for:
   - sample-domain stretch-engine class, readiness, degraded state, and
     fallback posture
   - clip-render, preview, and export-facing stretch receipts
   - the rule that later marker and artifact work must widen from one shared
     stretch substrate instead of inventing a second transform authority
5. `signal-runtime` must own the canonical consumer-visible meaning for:
   - warp-marker and transient-anchor analysis descriptors
   - tempo-assist posture and bounded tempo-support hints
   - analysis readiness, degraded state, and invalidation outcome
   - observation, supervisor, and stable host-edge export
6. future analysis engines, media services, or preview systems may deepen raw
   evidence, but they must not become the authority for:
   - a second marker taxonomy detached from runtime DTOs
   - host-local marker or anchor heuristics as the consumer boundary
   - product-local editing state as the analysis truth surface

If a warp-marker, anchor, or tempo-assist claim cannot be explained through
the closed media, analysis-metadata, and stretch contracts plus runtime-owned
receipts, it is not yet part of the reusable Signal contract.

## Existing anchors

Batch 16.1 freezes this contract on top of the current bounded stretch and
media surface family:

- `RuntimeMediaPipelineSnapshot`
- `RuntimeMediaServiceSnapshot`
- `RuntimeMediaLibraryServiceSnapshot`
- `RuntimeClipProcessingSnapshot`
- `RuntimeWarpClipSnapshot`
- `RuntimeStretchEngineSnapshot`
- `RuntimeClipRenderResult`
- `RuntimeOfflineRenderContractPreview`
- `RuntimeObservationReport`
- `RuntimeSupervisorReport`
- stable host-edge `supervisor_report()` export

Batch 16.1 does not claim those anchors already expose true marker, anchor, or
tempo-assist analysis depth. It freezes how later DTOs and proofs must widen
from them instead of inventing a separate host-private analysis shell.

## Shared vocabulary

### Warp marker

`warp marker` means a runtime-owned analysis record that identifies one
bounded timing-relevant point in source media for use by later stretch,
alignment, or artifact-aware workflows.

Warp markers are Signal-owned analysis evidence, not product-local editor
handles or UI-only marker points.

### Transient anchor

`transient anchor` means a runtime-owned analysis record that identifies a
bounded attack or timing-stable anchor point useful for stretch alignment,
tempo assistance, or later artifact-aware render work.

Anchors may compose with warp markers, but they are not automatically the same
thing. The shared contract must keep that distinction explicit.

### Tempo-assist analysis

`tempo-assist analysis` means the bounded runtime-owned evidence that helps
Signal reason about candidate tempo support for a clip or asset without
promoting a full beat-editing or DAW-style grid-authoring model.

Batch 16.1 freezes tempo assist as reusable runtime meaning rather than
product-local tap-detection or editor convenience logic.

### Analysis readiness

`analysis readiness` means whether the runtime-owned marker, anchor, or
tempo-assist surface is currently ready for downstream consumers to trust.

Readiness must stay typed and runtime-owned instead of inferred from cache
files, waveform presence, or editor-local state.

### Analysis degraded state

`analysis degraded state` means the runtime-owned answer for why marker,
anchor, or tempo-assist analysis cannot yet be fully realized through the
promoted shared path.

Batch 16.1 freezes degraded state as reusable Signal meaning for cases such
as:

- missing or invalidated media identity
- decode or analysis evidence not ready yet
- unsupported source layout or transform scope
- guarded or fallback-only analysis posture

### Analysis invalidation

`analysis invalidation` means the runtime-owned fact that marker, anchor, or
tempo-assist evidence must be considered stale because upstream media,
stretch-engine posture, or analysis inputs changed.

Invalidation must remain explicit and typed. It must not become an implicit gap
reconstructed from missing files or product-local refresh heuristics.

## Rules

### Rule 1: marker and anchor depth must widen from the closed stretch seam

`g07.016` must deepen the existing media, warp, clip-processing, and
stretch-engine surfaces from `046`. It must not create a second marker or
tempo-analysis engine detached from runtime-owned transform truth.

### Rule 2: analysis readiness and invalidation must stay runtime-owned

Shared consumers must not infer marker or anchor readiness from waveform
files, editor state, or product-local cache probes.

### Rule 3: markers, anchors, and tempo assist must share one vocabulary

Marker, anchor, and tempo-assist meaning may differ by scope, but the bounded
vocabulary for readiness, degraded state, and invalidation must remain shared
across observation, render planning, and later artifact work.

### Rule 4: no product-local editing semantics are implied

This contract may freeze analysis meaning and tempo-assist posture, but it
does not freeze marker-editing UX, arrangement workflow, or editor gesture
semantics.

### Rule 5: invalidation must compose with media and stretch truth

If marker or anchor evidence becomes stale because source media, stretch
readiness, or render posture changes, the invalidation answer must remain
runtime-owned and explainable through the closed media and stretch contracts.

### Rule 6: later artifact and audition work must widen from this boundary

Future `g07.017` and `g07.018` work must reuse this analysis boundary instead
of inventing new preview-local or artifact-local marker authorities.

## Deferred scope

Batch 16.1 intentionally does not claim:

- a realized runtime marker or anchor analysis service yet
- exhaustive beat-grid, barline, or arrangement intelligence
- product-local marker editing or corrective analysis UI
- ML-heavy media understanding breadth
- post-warp artifact caching and transform reuse depth
- low-latency audition, scrub, or preview-transform behavior

Those belong to later `g07.016`, `g07.017`, and `g07.018` batches.

## Batch 16.1 outcome

Batch 16.1 freezes the first bounded marker-analysis contract:

- Signal now has one explicit runtime-owned target for warp-marker,
  transient-anchor, tempo-assist, readiness, degraded-state, and invalidation
  meaning instead of host-local analysis tools or private editor heuristics
- the authority line is explicit: media identity, analysis-metadata, and
  stretch-engine truth remain the anchors, which prevents later marker work
  from reopening a second transform-analysis shell
- Batch 16.2 can now focus on materializing the first credible runtime-owned
  marker and anchor receipt family instead of reopening what analysis meaning
  belongs to Signal

## Batch 16.2 outcome

Batch 16.2 materializes the first bounded runtime-owned marker-analysis
receipt family:

- `signal-runtime` now owns typed warp-marker, transient-anchor,
  tempo-assist, readiness, and invalidation receipts derived from the closed
  clip-processing, stretch, warp, and media-library seams
- observation, supervisor, and stable host-edge exports now expose the same
  marker-analysis truth without host-local stretch-analysis reconstruction
- Batch 16.3 can now focus on the downstream-style proof seam instead of
  reopening what marker-analysis meaning belongs to runtime

The realized Batch 16.2 baseline remains intentionally bounded:

- marker and anchor counts are derived from the existing reusable media
  character descriptors instead of claiming a fuller beat-grid or editor
  marker-authoring engine
- tempo-assist posture is bounded to guarded versus ready runtime hints rather
  than broad transport or arrangement intelligence
- artifact-cache, low-latency audition, and richer transient-placement depth
  remain deferred

## Batch 16.3 outcome

Batch 16.3 closes the downstream-style proof seam for the bounded
marker-analysis contract:

- public runtime proof now shows `RuntimeMarkerAnalysisSnapshot` remains
  consumable through shared runtime observation and supervisor surfaces
  without host-local stretch-analysis reconstruction
- both stable host edges now prove they forward the same runtime-owned
  warp-marker, transient-anchor, tempo-assist, readiness, and invalidation
  receipts instead of rebuilding host-specific marker heuristics
- `signal-supervisor-tools` now exposes
  `signal.runtime.marker-analysis-boundary`, and Effigy now owns
  `acceptance:marker-analysis-boundary` as the repo-owned rerun lane

This closes the bounded `g07.016` contract seam while keeping fuller
editor-grade marker tooling, artifact-cache depth, and low-latency audition
explicitly deferred.

## Next Task

Continue `g07.017` with Batch 17.2 by materializing the first runtime-owned
post-warp render, cache, transform-artifact readiness, invalidation, and reuse
receipt family across runtime, supervisor, render-preview, and stable
host-edge surfaces without reopening host-local preview-cache ownership.
