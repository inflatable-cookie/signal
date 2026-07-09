# 046 Sample-Domain Time-Stretch Engine Contract

Status: complete; amended for first-party quality-depth program
Owner: core-product
Updated: 2026-07-09
Related contracts: `docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md`, `docs/contracts/029-analysis-metadata-extraction-and-library-service-contract.md`, `docs/contracts/034-multi-bus-graph-execution-and-auxiliary-topology-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first runtime-owned sample-domain time-stretch engine boundary for
`g07.015` so later warp-marker, artifact, preview, and audition work deepens
one shared Signal vocabulary instead of reopening host-local stretch engines,
private preview transforms, or product-specific media-processing shells.

## Authority hierarchy

Sample-domain time-stretch execution has one authority chain:

1. source media files, decode libraries, and media-cache artifacts provide raw
   audio bytes, duration, sample-rate, channel-layout, and decode success or
   failure evidence
2. `docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md`
   remains the authority for runtime-owned media identity, indexing,
   invalidation, waveform readiness, preview readiness, and analysis-ready
   service meaning
3. `docs/contracts/029-analysis-metadata-extraction-and-library-service-contract.md`
   remains the authority for reusable analysis-descriptor readiness and
   invalidation meaning whenever later stretch assistance depends on analysis
   evidence
4. existing runtime tempo-map, warp clip, and clip-processing receipts remain
   the authority for:
   - resolved project-tempo provenance
   - realized warp ratio and degraded warp state
   - clip-treatment ordering and post-warp clip-render expectations
5. `signal-runtime` must own canonical consumer-visible meaning for:
   - stretch-engine class and execution posture
   - stretch readiness, degraded state, and fallback outcome
   - render, preview, and export-facing stretch receipts
   - supervisor, observation, and stable host-edge export
6. future analysis, artifact, or preview milestones may deepen raw evidence,
   but they must not become the authority for:
   - a second stretch-engine taxonomy detached from runtime DTOs
   - host-local stretch readiness heuristics
   - product-local transform state as the consumer boundary

If a sample-domain stretch claim cannot be explained through the closed media
and analysis contracts plus runtime-owned tempo, warp, and clip-processing
receipts, it is not yet part of the reusable Signal contract.

## Existing anchors

Batch 15.1 freezes this contract on top of the current bounded runtime warp and
media surface family:

- `RuntimeTimelineSnapshot`
- `RuntimeTempoMapSnapshot`
- `RuntimeWarpClipSnapshot`
- `RuntimeClipProcessingSnapshot`
- `RuntimeMediaPipelineSnapshot`
- `RuntimeMediaServiceSnapshot`
- `RuntimeObservationReport`
- `RuntimeSupervisorReport`
- stable host-edge `supervisor_report()` export

Batch 15.1 does not claim those anchors already expose a true sample-domain
stretch engine. It freezes how later DTOs and proofs must widen from them
instead of inventing a separate host-private transform model.

## Shared vocabulary

### Sample-domain stretch engine

`sample-domain stretch engine` means the runtime-owned execution path that
turns media identity, source tempo, resolved project tempo, and bounded warp
intent into transformed sample output.

This is not a product-local preview effect, not a private editor transform,
and not a backend-specific DSP shell.

### Stretch-engine class

`stretch-engine class` means the bounded runtime-owned category for how Signal
is currently realizing stretch behavior.

Batch 15.1 freezes the categories, not final implementation breadth:

- `Disabled`
- `RatioOnly`
- `SampleDomain`
- `Fallback`

### Stretch readiness

`stretch readiness` means whether the runtime-owned engine can currently serve
the bounded stretch request for the asset or clip in a way that downstream
consumers can trust.

Readiness must stay typed and runtime-owned instead of inferred later from
cache paths, host logs, or render-side failure strings.

### Stretch degraded state

`stretch degraded state` means the runtime-owned answer for why the current
stretch request cannot be fully realized through the promoted engine path.

Batch 15.1 freezes degraded state as reusable Signal meaning for cases such as:

- missing media asset or not-ready cached media
- missing source tempo or unresolved project tempo
- unsupported ratio, layout, or transform scope
- fallback-only execution

### Stretch fallback

`stretch fallback` means the runtime-owned reduced behavior Signal provides
when the promoted sample-domain engine cannot fully realize the request.

Fallback must remain explicit and typed. It must not silently collapse into
host-local preview shortcuts or private offline-render code paths.

### Stretch scope

`stretch scope` means the bounded runtime-owned context where stretch truth is
being consumed:

- observation and diagnostics
- live preview or audition
- clip-processing and render export
- later artifact or cache reuse

Scope is part of the contract because later preview and artifact milestones
must widen from the same engine truth.

## Rules

### Rule 1: stretch work must widen from the current warp and media seam

`g07.015` must deepen the existing runtime tempo-map, warp clip, and
clip-processing receipts. It must not create a second transform or preview
engine detached from runtime-owned media truth.

### Rule 2: engine readiness and degraded state must stay runtime-owned

Shared consumers must not infer stretch readiness from decode logs, cache-file
presence, or product-local transform heuristics.

### Rule 3: render, preview, and export must share one engine vocabulary

Stretch meaning may differ by scope, but the bounded vocabulary for engine
class, readiness, degraded state, and fallback must stay shared across preview,
clip render, and later artifact work.

### Rule 4: no product-local editing semantics are implied

This contract may freeze engine meaning and fallback behavior, but it does not
freeze warp-marker editing UX, timeline manipulation UX, or arrangement
workflow semantics.

### Rule 5: stretch depth must stay bounded to reusable runtime evidence

Batch 15.1 is not permission for open-ended algorithm bakeoffs. Widening must
stay tied to runtime receipts, downstream observability, and later marker or
artifact milestones.

### Rule 6: later marker, artifact, and preview work must deepen this contract

Future `g07.016`, `g07.017`, and `g07.018` work must widen this contract
additively instead of inventing new product-local transform authorities.

## Deferred scope

Batch 15.1 intentionally does not claim:

- a realized sample-domain stretch engine implementation yet
- warp-marker, transient-anchor, or tempo-assist analysis depth
- post-warp artifact caching and invalidation depth
- low-latency audition or scrub transform services
- exhaustive algorithm parity across every ratio, layout, or artifact policy
- product-local warp editing UX or transform workflow policy

Those belong to later `g07.015` batches and follow-on milestones.

## 2026-07-05 Quality-Depth Addendum

Operator decision: pursue a first-party Signal-native high-quality stretch and
pitch engine. The goal is Rubber Band-class behavior from the outset, while
preserving clean-room implementation boundaries.

This addendum does not reopen the closed `g07.015` ownership baseline. It
deepens the quality target for the next active lane:

- Signal owns the DSP engine and tier vocabulary.
- Rubber Band is a behavioral/listening benchmark only, not source material.
- Signalsmith Stretch may inform comparison because it is permissively
  licensed, but it is not the default implementation answer.
- The tier vocabulary is `Repitch`, `RealtimePreview`, and
  `OfflineHighQuality`.
- `Repitch` remains render-plane varispeed and may run on the audio thread.
- `RealtimePreview` must prove bounded latency, dynamic-ratio behavior,
  transient preservation, image stability, and reported latency before it can
  enter realtime playback.
- `OfflineHighQuality` must be deterministic and cache/export safe, with
  sample-accurate or near-sample-accurate alignment where promised.

Required evidence before promotion:

- benchmark corpus covering drums/percussion, bass, vocals, pads/sustains,
  full mixes, tempo ramps, loop seams, and extreme ratios
- measurable timing drift, transient smear, phasiness, vertical coherence,
  stereo image stability, loop-click, CPU, latency, and memory metrics
- explicit cache identity for engine version, tier, content hash, ratio/pitch
  curves, warp markers, channel layout, and tick/sample projection epoch
- focused tests for DSP length/alignment contracts before render/export
  integration widens

## 2026-07-09 RealtimePreview Callback Gate Addendum

`RealtimePreview` callback DSP has two separate gates:

- callback-local DSP safety: bounded work, no allocation, no locks, no I/O,
  deterministic latency, linked stereo, dynamic-ratio alignment, and seam
  evidence
- render-plane source projection: a runtime-owned contract for how output
  frames advance through source media when ratio is not `1.0`

Passing callback-local DSP safety is not enough to enter realtime playback.
The stream contract must expose a source/output timeline mode:

- `QuantumLocked`: caller provides one input quantum for one output quantum.
  This can prove callback-local DSP behavior and is useful for preview
  evidence, but it is not render-plane time-stretch playback.
- `SourceProjected`: the callback path owns or reports ratio-projected source
  advancement, output position, latency, and underrun/fill behavior well
  enough for the render plane to remain sample-domain honest.

`audio_thread_processing_supported` must stay `false` while the
RealtimePreview stream is `QuantumLocked`. `g10.027` proved source-projection
reporting and dynamic-ratio source/output continuity, but it did not give the
callback path ownership of source-buffer fill or underrun behavior. The stream
may only become `CallbackSafeStreaming`/`SourceProjected` after focused tests
prove source advance, output position, bounded input demand, underrun/fill
policy, latency, and no-allocation behavior together.

Planning authority: `docs/roadmaps/g10/028-realtime-preview-source-fill-contract.md`.

## 2026-07-09 Correctness And Listening Gate Addendum

Audit evidence invalidates automatic continuation from callback projection into
source-fill exposure. The current stretch implementation remains prototype
quality until all of these gates pass together:

- offline STFT analysis covers source content at both boundaries instead of
  satisfying length contracts through zero padding after incomplete analysis
- dynamic-ratio output preserves continuous algorithm state or uses an
  explicitly measured transition mechanism rather than raw segment
  concatenation
- linked stereo claims distinguish mid/side transport from genuinely shared
  multichannel phase, peak, and transient decisions
- full-render measurements cover endpoint energy, dropout spans, peak growth,
  CPU, latency, and memory instead of relying on aligned excerpts alone
- promotion requires absolute acceptance limits plus completed real-source
  listening evidence; improvement over the draft backend is insufficient
- callback source projection is coupled to actual kernel input consumption
  before source-fill or render-plane exposure can open

`OfflineHighQuality` remains a product-addressable prototype, not a
Rubber Band-class quality claim, while this gate is open. RealtimePreview keeps
`audio_thread_processing_supported=false`.

Planning authority:
`docs/roadmaps/g10/029-stretch-correctness-and-listening-gate.md`.

## Batch 15.1 outcome

Batch 15.1 freezes the first bounded sample-domain stretch-engine contract:

- Signal now has one explicit runtime-owned target for stretch-engine class,
  readiness, degraded state, fallback, and scope instead of host-local preview
  or export transform logic
- media identity, analysis readiness, tempo-map provenance, warp receipts, and
  clip-processing meaning remain the anchors, which prevents later stretch work
  from reopening a second media-processing shell
- Batch 15.2 can now materialize the first credible sample-domain engine
  baseline instead of reopening which stretch semantics belong to Signal

## Batch 15.2 outcome

Batch 15.2 materializes the first runtime-owned sample-domain stretch receipt
family:

- `signal-runtime` now derives typed stretch-engine class, readiness, degraded
  state, and fallback from the closed clip-processing seam instead of leaving
  stretch truth implicit in warp receipts alone
- render, offline-render preview, observation, supervisor, and stable
  host-edge JSON surfaces now share one stretch snapshot family instead of
  rebuilding transform posture per consumer
- later proof and consumer-boundary work can now widen from one realized
  baseline instead of reopening what counts as a reusable stretch engine

## Batch 15.3 outcome

Batch 15.3 closes the bounded stretch-engine proof seam:

- public runtime now proves `RuntimeStretchEngineSnapshot` remains consumable
  through shared runtime reports, clip-render receipts, and offline-render
  preview without host-local transform reconstruction
- both stable host edges now prove they forward the same stretch-engine
  class, readiness, degraded-state, and fallback truth through supervisor
  export
- `signal-supervisor-tools` now exposes `signal.runtime.stretch-boundary`, and
  Effigy now owns `acceptance:stretch-boundary` as the repo-owned rerun lane

This closes `g07.015` as the bounded sample-domain time-stretch contract.
Marker-analysis, artifact-cache, low-latency audition, and broader algorithm
support remain later work.

## Next Task

Continue `g07.016` with Batch 16.2 by materializing the first runtime-owned
warp-marker, transient-anchor, tempo-assist, readiness, and invalidation
receipt family across runtime, supervisor, and stable host-edge surfaces
without reopening host-local stretch-analysis ownership.
