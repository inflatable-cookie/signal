# 049 Low-Latency Audition, Scrub, And Preview-Transform Service Contract

Status: complete
Owner: core-product
Updated: 2026-03-19
Related contracts: `docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md`, `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`, `docs/contracts/047-warp-marker-transient-anchor-and-tempo-assist-analysis-contract.md`, `docs/contracts/048-post-warp-render-cache-and-transform-artifact-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first runtime-owned low-latency audition, scrub, and
preview-transform service boundary for `g07.018` so later browser, editor, and
workflow depth widens one shared Signal vocabulary instead of reopening
host-local preview players, private scrub transforms, or product-specific media
audition shells.

## Authority hierarchy

Low-latency audition and preview-transform depth has one authority chain:

1. source media files, decode libraries, and media-cache artifacts provide raw
   audio bytes, duration, sample-rate, channel-layout, and decode success or
   failure evidence
2. `docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md`
   remains the authority for runtime-owned media identity, indexing,
   invalidation, waveform readiness, preview readiness, and analysis-ready
   service meaning
3. `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`
   remains the authority for:
   - stretch-engine class, readiness, degraded state, fallback, and scope
   - the rule that preview and audition work must widen from one shared
     stretch substrate instead of inventing a second transform engine
4. `docs/contracts/047-warp-marker-transient-anchor-and-tempo-assist-analysis-contract.md`
   remains the authority for:
   - warp-marker, transient-anchor, and tempo-assist posture
   - bounded marker-analysis readiness and invalidation meaning
   - the rule that later scrub or audition work must compose with shared
     marker-analysis truth instead of host-local heuristics
5. `docs/contracts/048-post-warp-render-cache-and-transform-artifact-contract.md`
   remains the authority for:
   - transform-artifact identity, readiness, invalidation, reuse, and degraded
     posture
   - preview-facing and render-facing artifact alignment
   - the rule that preview work must reuse one shared artifact vocabulary
6. `signal-runtime` must own the canonical consumer-visible meaning for:
   - low-latency audition and scrub request scope
   - preview-transform service class, readiness, degraded state, and fallback
   - preview artifact alignment and bounded reuse posture
   - observation, supervisor, render-preview, and stable host-edge export
7. future browser, editor, or remote-control workflows may deepen user-facing
   behavior, but they must not become the authority for:
   - a second preview taxonomy detached from runtime DTOs
   - host-local scrub or audition heuristics as the consumer boundary
   - product-local preview cache state as the transform truth

If a preview-transform claim cannot be explained through the closed media,
stretch, marker-analysis, and transform-artifact contracts plus runtime-owned
receipts, it is not yet part of the reusable Signal contract.

## Existing anchors

Batch 18.1 freezes this contract on top of the current bounded preview and
transform surface family:

- `RuntimeMediaPipelineSnapshot`
- `RuntimeMediaServiceSnapshot`
- `RuntimeStretchEngineSnapshot`
- `RuntimeMarkerAnalysisSnapshot`
- `RuntimeTransformArtifactSnapshot`
- `RuntimeClipRenderResult`
- `RuntimeOfflineRenderContractPreview`
- `RuntimeObservationReport`
- `RuntimeSupervisorReport`
- stable host-edge `supervisor_report()` export

Batch 18.1 does not claim those anchors already expose a realized low-latency
preview engine. It freezes how later DTOs and proofs must widen from them
instead of inventing a separate host-private preview model.

## Shared vocabulary

### Low-latency audition

`low-latency audition` means the runtime-owned bounded preview path that lets a
consumer request short-form transformed playback aligned with the shared
stretch, marker-analysis, and transform-artifact seams.

This is not a product-local browser player, not a private editor monitor path,
and not a host-local audio callback shell.

### Scrub preview

`scrub preview` means the runtime-owned bounded transform-preview posture for
short, position-driven audition requests that need to stay aligned with shared
stretch and artifact truth.

Scrub preview is a scope of the shared preview-transform service, not a second
preview engine.

### Preview-transform service

`preview-transform service` means the runtime-owned service boundary that
answers whether bounded audition and scrub requests can be served through the
promoted shared transform path.

Batch 18.1 freezes the service meaning, not final implementation breadth.

### Preview service class

`preview service class` means the bounded runtime-owned category for how Signal
is currently serving preview-transform work.

Batch 18.1 freezes the categories, not final execution breadth:

- `Unavailable`
- `StretchAligned`
- `ArtifactBacked`
- `Fallback`

### Preview readiness

`preview readiness` means whether the runtime-owned preview-transform service
can currently serve the bounded request in a way downstream consumers can
trust.

Readiness must stay typed and runtime-owned instead of inferred from file
presence, UI-local warm state, or host-local playback shortcuts.

### Preview degraded state

`preview degraded state` means the runtime-owned answer for why the current
audition or scrub request cannot be fully realized through the promoted path.

Batch 18.1 freezes degraded posture as reusable Signal meaning for cases such
as:

- missing or unavailable media identity
- stretch or marker-analysis not ready
- transform-artifact unavailable or invalidated
- preview scope unsupported by the promoted service
- fallback-only preview behavior

### Preview fallback

`preview fallback` means the runtime-owned reduced behavior Signal provides
when the promoted preview-transform service cannot fully realize the request.

Fallback must remain explicit and typed. It must not silently collapse into a
host-local preview player or editor-only shortcut.

### Preview scope

`preview scope` means the bounded runtime-owned context where preview-transform
truth is being consumed:

- low-latency audition
- scrub preview
- runtime observation and diagnostics
- supervisor and stable host-edge export

Scope is part of the contract because later workflow and browser work must
widen from the same preview truth.

## Rules

### Rule 1: preview work must widen from the closed stretch and artifact seams

`g07.018` must deepen the existing runtime stretch-engine, marker-analysis,
and transform-artifact surfaces. It must not create a second preview engine
detached from runtime-owned transform truth.

### Rule 2: readiness, degraded state, and fallback must stay runtime-owned

Shared consumers must not infer preview service posture from cache files,
decode logs, transport side effects, or product-local UI state.

### Rule 3: audition, scrub, and preview export must share one vocabulary

Preview meaning may differ by scope, but the bounded vocabulary for service
class, readiness, degraded state, fallback, and artifact alignment must stay
shared across audition, scrub, observation, supervisor export, and later
workflow depth.

### Rule 4: preview alignment must compose with transform-artifact truth

If preview requests can reuse transformed artifacts, that reuse answer must
remain explainable through the closed transform-artifact contract instead of a
new preview-cache taxonomy.

### Rule 5: no product-local browser or editor workflow is implied

This contract may freeze preview service meaning and degraded posture, but it
does not freeze browser UX, scrub interaction design, remote preview transport,
or product-specific editing workflows.

### Rule 6: low-latency is bounded reusable runtime meaning

Batch 18.1 freezes a bounded reusable preview contract. It does not promise
every hardware path, algorithm, or workflow will realize identical latency or
preview breadth.

## Deferred scope

Batch 18.1 intentionally does not claim:

- a realized low-latency preview engine implementation yet
- full browser-remote audition or scrub transport
- richer editor interaction semantics
- exhaustive artifact-retention or preview-cache policy
- final device-routing or monitoring behavior for preview playback
- wider workflow automation around preview requests

Those belong to later `g07.018` batches and follow-on milestones.

## Batch 18.1 outcome

Batch 18.1 freezes the first bounded low-latency preview-transform contract:

- Signal now has one explicit runtime-owned target for low-latency audition,
  scrub preview, preview service class, readiness, degraded state, fallback,
  and artifact alignment instead of host-local preview players or product-local
  browser shells
- media identity, stretch-engine truth, marker-analysis truth, and
  transform-artifact truth remain the anchors, which prevents later preview
  work from reopening a second transform authority
- Batch 18.2 can now focus on materializing the first credible preview-service
  receipt family instead of reopening which preview semantics belong to Signal

## Batch 18.2 outcome

Batch 18.2 materializes the first bounded runtime-owned preview-transform
receipt family:

- `signal-runtime` now owns typed preview service class, readiness, degraded
  state, fallback, active audition, and scrub-supported posture instead of
  leaving preview readiness implicit in media preview state or host-local
  playback code
- the same receipt family now flows through runtime observation, supervisor
  export, clip-render results, offline render preview, and stable host-edge
  JSON instead of splitting preview meaning across render and host surfaces
- preview posture now composes directly with the closed media-service,
  stretch-engine, marker-analysis, and transform-artifact seams, which keeps
  later browser or workflow depth additive on one shared substrate

Batch 18.2 intentionally does not claim a full low-latency preview engine,
browser-remote transport, or richer preview-device routing policy. Those
remain later work.

## Batch 18.3 outcome

Batch 18.3 closes the bounded consumer seam for the preview-transform service:

- public runtime, both stable host edges, and `signal-supervisor-tools` now
  prove one shared runtime-owned preview vocabulary for low-latency audition,
  scrub support, readiness, degraded state, fallback, and artifact alignment
- the machine-readable `signal.runtime.preview-transform-boundary` descriptor
  and repo-owned `acceptance:preview-transform-boundary` lane now make that
  seam rerunnable without reading host-local preview playback code
- `g07.018` is therefore closed as a bounded reusable Signal seam, while
  fuller low-latency execution, preview-device routing, and browser workflow
  depth remain later work

## Next Task

Continue `g08.001` with Batch 1.1 by freezing the runtime-owned live Linux
audio backend ownership and session-lifecycle contract before deeper ALSA,
JACK, and PipeWire runtime realization widens.
