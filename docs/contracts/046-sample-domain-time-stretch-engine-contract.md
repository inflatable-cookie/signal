# 046 Sample-Domain Time-Stretch Engine Contract

Status: complete; amended for first-party quality-depth program, the 2026-07-27
Transparent defect correction, and the 2026-07-27 cache identity audit
Owner: core-product
Updated: 2026-07-27
Related contracts: `docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md`, `docs/contracts/029-analysis-metadata-extraction-and-library-service-contract.md`, `docs/contracts/034-multi-bus-graph-execution-and-auxiliary-topology-contract.md`, `docs/contracts/084-stretch-candidate-isolation-and-promotion-contract.md`, `docs/contracts/085-creative-time-stretch-product-and-routing-contract.md`; historical proof policy `082`
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

The first absolute OfflineHighQuality correctness gate is
`offline-high-quality-v1`:

- output length drift: at most `0.5` frame
- active-source endpoint RMS change: at most `7 dB`
- added full-render silence: `0` frames
- positive peak growth: at most `6 dB`

Endpoints below the configured source silence floor are reported as inactive
instead of manufacturing a `240 dB` delta. CPU realtime factor uses measured
Signal render time divided by rendered-audio duration. Peak working memory uses
measured live-heap growth above the pre-render baseline. Neither resource
metric may be reconstructed from report wall time, output length, or buffer
capacity.

Synthetic draft comparisons are regression evidence only. A product-facing
OfflineHighQuality receipt must also carry:

- a passing `offline-high-quality-v1` absolute integrity result
- non-zero external-comparator coverage meeting its declared row requirement
- corpus coverage meeting its declared case requirement
- completed blind-listening findings for percussion, bass, vocals,
  pads/sustains, and full mix

The blind pack must conceal Signal/external assignment until notes are frozen,
level-match source and candidates under one documented policy, and require
transient, tonal, stereo, formant, boundary, and preference fields. A synthetic
receipt may pass its own policy while product-facing use remains blocked.

Output-length drift and transient-event placement are separate evidence.
Timing claims must refine detected source attacks against their ratio-projected
output positions and report signed and absolute offsets. Transient spike review
must use level-invariant local crest evidence with source/output event locations;
full-render peak growth alone cannot identify the failing attack. Diagnostic
search bounds and incomplete event matches must remain explicit and must not be
promoted into acceptance limits without corpus calibration.

A transient candidate must report its result at the current path's failing
source event and its own worst event. Moving the largest crest to another attack
is not an improvement. Phase-lock changes must also retain event-placement
evidence; a local crest reduction cannot promote a path that regresses timing
across the bounded corpus.

Long-stretch tonal diagnostics must remain source-relative and separate static
spectral change from fast texture movement. At minimum, report gain-invariant
spectral residual, energy added outside source-supported bins, local
frame-to-frame spectral movement, and short-time envelope movement at
ratio-projected positions. Added-ringing claims require unsupported-bin
evidence; spectral movement alone is a temporal-coherence finding. These are
offline diagnostic proxies, not audio-thread work or promotion limits. Fixed
windowing and source/output alignment bounds must stay explicit, and objective
evidence does not replace completed listening review.

Formant diagnostics for no-pitch-shift stretch must compare a gain-invariant,
broadly smoothed source/output spectral envelope at ratio-projected positions.
Broad-envelope residual and centroid movement are classification evidence, not
exact vowel-formant tracking or promotion limits. A fixed-ratio formant
correction requires a measured envelope failure; it must not be introduced from
operator vocabulary alone.

Boundary diagnostics must distinguish endpoint energy and silence spans from
the actual exterior transition. Head evidence is silence to first sample; tail
evidence is final sample to silence. Do not substitute the largest derivative
inside an endpoint span. Relative crest evidence must exclude inactive edges,
while absolute dBFS remains visible so near-silence cannot create a false large
ratio. A boundary candidate must retain source content and pass full-render,
transient, tonal-texture, and formant-envelope regression checks together.

A tail control must declare whether it targets the source endpoint or digital
silence. Source matching is a content-preservation control, not proof of a
standalone-safe artifact. A silence-target control must report its changed
frame span and peak correction, retain the absolute endpoint-energy gate, and
remain unpromoted until linked-stereo behavior and listening evidence cover the
same policy.

Passing full-render and interior quality proxies does not promote a
silence-target tail correction. The evidence must also expose the corrected
span followed by digital silence in a level-matched listening artifact, cover
the largest corrections and loudest original edges, and keep candidate identity
concealed until notes are frozen. Mono listening may qualify the correction's
local sound; linked-stereo evidence remains mandatory for production routing.

Fixed endpoint envelopes must not be chained through shape variants after
material-dependent listening failures. An adaptive policy requires a
deterministic selector derived from tail-local signal measurements, not corpus
case labels or endpoint amplitude alone. If labeled wins and losses are not
separable by those measurements, tail-envelope promotion stops and the boundary
remains unmodified pending a different algorithm class. Any selector must later
share its decision across linked stereo.

Planning authority:
`docs/roadmaps/g10/030-stretch-consolidation-and-completion.md`.

## Historical 2026-07-10 OfflineHighQuality Structural Hybrid Addendum

The next OfflineHighQuality candidate is a fixed-ratio, report-only structural
hybrid. It has three local owners:

- a `1024/256` independent-bin transient branch
- the current `2048/512` identity-lock/reset branch for mixed and uncertain
  regions
- a `4096/1024` identity-lock/reset tonal branch for stable expansion regions

The classifier is local synthesis policy, not the globally normalized benchmark
detector. The first candidate freezes the current `0.30` spectral-flux ratio,
`1.20` energy ratio, and `0.70` spectral-stability evidence. Transient guards
cover one short hop before through three short hops after an onset. Tonal entry
requires four consecutive stable short hops outside a transient guard. Failed
combined evidence rejects this shape rather than opening a threshold sweep.

All branches must retain continuous phase state and map their window centres to
one exact output timeline. Ownership transitions are bounded to two branches,
use a `256`-sample raised-cosine crossfade whose linear weights sum to one, and
may move by at most one short hop to a low-energy point outside a transient
guard. A transition requires at least `0.50` outgoing/incoming correlation and
at most `1 dB` correlation-aware energy-normalization correction; otherwise the
current mixed branch owns the region. Stereo uses one shared transition gain.
Start and end guards stay on the current branch. Centred boundary padding,
exact cropping, identity bypass, determinism, and full-render integrity remain
mandatory.

Linked stereo requires one multichannel core. Classification, branch ownership,
transient resets, shared peak regions, and transitions are channel-shared.
Per-channel instantaneous frequency remains independent; shared-peak synthesis
must retain the analysis interchannel phase difference. Mid/side conversion
followed by independent mono engines is transport, not shared stereo policy.

No fixed-ratio formant correction or tail envelope is authorized. Pitch
composition and dynamic-ratio routing remain on their current prototype paths.
A later dynamic hybrid must carry classifier, phase, branch, and synthesis state
through ratio changes instead of concatenating independent segment renders.

The first implementation sequence is kernel extraction with bit-exact default
proof, fixed-ratio mono gating, shared-decision linked stereo, then concealed
listening and dynamic-ratio reassessment. Production routing, cache identity,
product receipts, and RealtimePreview support remain unchanged until the full
contract gates pass.

Detailed design and stop conditions:
`docs/logs/2026-07/10-g10-029-structural-hybrid-design.md`.

The independent-output hybrid and its successors were rejected and removed in
the 2026-07-19 consolidation. Contract `082` preserves that proof history.
Contract `084` now governs isolated complete-system successor work.

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

## 2026-07-19 Creative Stretch Separation

Contract `085` adds an offline creative-expansion intent beside this tier
vocabulary. It does not change `Repitch`, `RealtimePreview`, or
`OfflineHighQuality`; it does not widen transparent quality claims; and it adds
no runtime or public Rust surface until separately promoted.

## 2026-07-27 Transparent Renderer Defect Correction Addendum

Planning authority: `docs/roadmaps/g10/036-transparent-stretch-correctness-recovery.md`.
Evidence: `docs/logs/2026-07/27-g10-036-stretch-audit-intake.md`,
`docs/logs/2026-07/27-g10-036-defect-authority.md`.

The 2026-07-27 audit measured four defects in the retained Transparent
renderer. This addendum freezes the laws they violate. It corrects defects in
the frozen baseline; it does not widen quality claims, open successor
research, or change any tier vocabulary.

### Overlap coverage law

Synthesis hop is `analysis_hop * ratio`. When it grows past the window, the
overlap-add sum collapses and the `1.0e-3` normalization gate zeroes output
samples.

The renderer must satisfy `analysis_hop * ratio <= 0.75 * window_size`. When a
configured geometry would exceed it, the renderer reduces the analysis hop to
`floor(0.75 * window_size / ratio)`, with a floor of `1`. Window size and the
caller's requested hop are otherwise unchanged.

The bound is measured, not assumed. Interior 512-frame RMS blocks, 48 kHz,
`2048` window, `512` requested hop:

| ratio | hop now | ripple now | zeroed now | hop under law | ripple under law | zeroed under law |
| --- | --- | --- | --- | --- | --- | --- |
| `1.0` | `512` | `0.276 dB` | `0` | `512` | `0.276 dB` | `0` |
| `2.0` | `512` | `0.276 dB` | `0` | `512` | `0.276 dB` | `0` |
| `3.0` | `512` | `0.276 dB` | `0` | `512` | `0.276 dB` | `0` |
| `4.0` | `512` | `1.396 dB` | `0` | `384` | `0.276 dB` | `0` |
| `6.0` | `512` | `237.126 dB` | `183/547` | `256` | `0.276 dB` | `0` |
| `8.0` | `512` | `237.126 dB` | `368/734` | `192` | `0.358 dB` | `0` |

A three-tone broadband source gives the same result: `1.615 dB` to `0.447 dB`
at ratio `4.0`, and `231.781 dB` with `368` zeroed blocks to `0.477 dB` with
none at ratio `8.0`.

At the frozen `2048/512` geometry the law leaves every ratio through `3.0`
byte-identical. Only `3.0 < ratio` changes. Cost is a finer analysis hop:
`2.67x` more analysis frames at ratio `8.0`.

### Dynamic-ratio segment law

No admitted ratio curve may produce a render segment shorter than
`window_size`. Sub-window segments currently fall through to time-domain
interpolation, which pitch-shifts, so a curve sampled finer than one window
silently converts a pitch-preserving render into varispeed.

Adjacent curve spans must be coalesced until every segment is at least
`window_size + 8 * analysis_hop` source frames. The coalesced segment's target
frame count is the sum of the target frame counts its constituent spans would
have produced, so total output length and average tempo over the span are
preserved exactly and the segment renders at the mean ratio of the spans it
covers.

The minimum is not one window, for two measured reasons.

Pitch. A single-window segment avoids the interpolation fallback but gives the
phase vocoder one analysis frame, so it tracks the source poorly. Measured on a
`440 Hz` tone through a curve sampled every `1024` frames at ratio `2.0`, at
the retained `2048/512` geometry: `window + 3 hops` leaves `19.6` cents of
error, `window + 8 hops` leaves `2.8`, `window + 32 hops` leaves none.

Seam-rate modulation. Segments render independently and are butt-joined, so
every join leaves an envelope dip and the render modulates at the segment rate.
Concealed listening at `window + 8 hops` heard this directly, as a secondary
rhythmic pulse. Measured envelope modulation at the segment period, against the
`0.04 dB` floor of the same material rendered whole:

| minimum | source frames | joins | modulation | pitch error |
| --- | --- | --- | --- | --- |
| `window + 8 hops` | `6144` | `31` | `0.545 dB` | `2.8` cents |
| `window + 16 hops` | `10240` | `18` | `0.268 dB` | `2.8` cents |
| `window + 32 hops` | `18432` | `10` | `0.115 dB` | `0` cents |
| `window + 64 hops` | `34816` | `5` | `0.039 dB` | `0` cents |

`window + 32 hops` is frozen: `18432` source frames, `384 ms` at 48 kHz. Sixty-
four extra hops reaches the modulation floor, but its `725 ms` minimum swallows
realistic tempo-ramp spans, and a longer minimum only trades modulation against
ratio-curve time resolution.

### Dynamic-ratio segmentation is not transparent

Two concealed listening rounds rejected segment-length tuning, and measurement
found why. Segments render independently, so each restarts the phase vocoder
and the phase relationship across every join is arbitrary. Rendering a constant
ratio through the segmented path produces a waveform almost uncorrelated with
the same ratio rendered whole:

| measurement | value |
| --- | --- |
| correlation | `0.034` |
| peak sample difference | `1.1470` |
| difference RMS against signal RMS | `0.2474` against `0.1784` |

The audible result is a periodic pulse whose rate tracks segment length, not
amplitude modulation. Lengthening segments changes the rate and never removes
the defect, so no minimum can fix it. Contract `084` Rule 7 closes segmentation
tuning as a mechanism.

This predates the 2026-07-27 correction. Contract `046` already required that
dynamic-ratio output "preserves continuous algorithm state or uses an explicitly
measured transition mechanism rather than raw segment concatenation", and the
implementation has never satisfied it.

The correction is admitted anyway, by explicit operator decision under Rule 5,
because it replaces an octave-wide pitch error with a milder seam artifact and
cuts joins from about `46` to `10` on the measured curve. The residual pulse is
a recorded limitation, not a tuning target. `g10.039` removes it by carrying
renderer state across the join, and
`segmented_render_matches_whole_render_at_constant_ratio` is its acceptance
target.

The interpolation fallback remains valid for one case only: a whole input
shorter than one window. It must never be reached through segmentation.

This law governs the renderer. `plan_offline_stretch_chunks` still segments on
raw curve points, so a dense curve can still produce sub-window chunks in the
artifact path. `g10.039` owns making the plan and the renderer share one
segmentation, because the durable fix there is a renderer that carries state
across a chunk rather than a longer minimum.

### Seam parity law

Segment-join treatment must not depend on channel count. Whatever mechanism
the renderer applies at a dynamic-ratio join must be applied identically for
mono and interleaved renders, and the frozen seam-click metric must agree
across channel counts within one stated tolerance.

The current mechanism is an interim measure. It derives a midpoint from the two
samples either side of the join and adds a decaying offset, which is not a
crossfade. `g10.039` replaces it with a renderer that carries state across the
boundary. Parity is required now; the mechanism is not frozen.

### Output bound

Whole-buffer stretch entry points must refuse renders whose output exceeds a
frozen sample ceiling rather than attempting the allocation. Ratio `1.0e6` over
`4096` input frames currently allocates `4096000000` samples and returns after
roughly one minute.

`TimeStretcher` is fallible. `stretch_mono` and every whole-buffer entry point
beside it return a typed render result, so a backend that cannot serve a
request says so instead of attempting the allocation. This is a breaking change
to an in-repo-only trait, taken deliberately pre-1.0 with no compatibility
shim; `signal-render-plane` and `signal-runtime` update in the same batch.

The ceiling is `268435456` output samples, one gibibyte of `f32`. At 48 kHz
that is roughly `93` minutes mono or `46` minutes stereo in one whole-buffer
call. Renders above it are the chunk plan's responsibility, and `g10.039` owns
making that path carry state.

### Correction classes

Corrections to the frozen baseline fall into two classes with different
evidence requirements:

- extension: behavior changes only outside the retained `0.5x..4x` product
  range, or only for inputs the renderer previously destroyed. Byte-exact
  output inside the unaffected range is the acceptance proof.
- audible correction: behavior changes inside the retained product range.
  Objective rows plus concealed listening are required before admission, under
  Contract `084` Rule 5.

Under the measured overlap law the audible window is narrow. The classes are:

| defect | class | affected range |
| --- | --- | --- |
| overlap coverage | audible correction | `3.0 < ratio <= 4.0` |
| overlap coverage | extension | `ratio > 4.0` |
| dynamic-ratio segments | audible correction | any curve with sub-window spans |
| seam parity | audible correction | mono dynamic-ratio renders |
| output bound | extension | refused renders only |

## 2026-07-27 Stretch Cache Identity Addendum

Planning authority: `docs/roadmaps/g10/037-stretch-cache-identity-completeness.md`.
Evidence: `docs/logs/2026-07/27-g10-037-identity-gap-audit.md`.

The 2026-07-19 promotion evidence list named the cache identity fields as
"engine version, tier, content hash, ratio/pitch curves, warp markers, channel
layout, and tick/sample projection epoch". That list is incomplete: three
classes of input change rendered output without changing the key.

### Every input that changes rendered output

| input | in the identity today |
| --- | --- |
| engine version | yes |
| tier | yes |
| offline renderer path | yes |
| source content hash | yes |
| channel layout | yes |
| ratio curve | yes |
| pitch curve | yes |
| warp markers | yes |
| projection epoch | yes |
| STFT window size | **no** |
| analysis hop | **no** |
| offline chunk policy | **no** |
| overlap coverage fraction | **no** |
| dynamic-ratio segment minimum | **no** |
| segment seam smoothing length | **no** |
| transient detector thresholds | **no** |
| short-window selector gates | **no** |
| pitch-shift resample quality | **no** |
| creative character, `space`, `cycle`, seed | **no identity exists**, and Contract `085` now declares creative renders uncacheable |

### Measured collisions

Chunk policy. One identity, `stable_hash=2e0b01234f55947c`, rendered through a
single chunk and through eight chunks over the same `96000`-frame source at
ratio `1.25`:

| measurement | value |
| --- | --- |
| output frames | `120000` both |
| correlation between the two renders | `-0.296620` |
| peak sample difference | `0.5428` |

Two unrelated renders share one key. Chunk boundaries move where segment
renders restart phase, and phase restarts are already recorded above as
producing near-uncorrelated output.

Render geometry. `OfflineHighQualityStretcher::with_window` is public, so
`2048/512` and `1024/256` renders of the same source collide on one key.

Behavior version. The 2026-07-27 defect correction changed rendered output at
every ratio above `3.0` and for every dynamic-ratio curve, while
`SIGNAL_STRETCH_ENGINE_VERSION` stayed `signal-native-stretch-v2`. Artifacts
written before and after that correction are indistinguishable by key. Any
cache populated before it now serves stale audio.

### Frozen rules

Cache identity must cover every input that changes rendered output. That is the
nine fields above plus render geometry, chunk policy, and a behavior version
that advances whenever any renderer constant or law changes.

Key material must use explicit stable tokens. Derived formatting, including
`Debug`, is not a stability contract: renaming a variant silently changes every
key, and reusing a name silently aliases two renders onto one.

The behavior version must advance in the same change that alters renderer
output. A correction that changes audio without advancing it is incomplete.

The canonical key is authoritative for equality. The stable hash is a
non-cryptographic 64-bit FNV-1a digest and is a bucketing aid only. A consumer
that treats a hash match as identity is relying on a guarantee this contract
does not make; it must compare the canonical key on a candidate hit.

### Schema advance

`STRETCH_CACHE_IDENTITY_SCHEMA_VERSION` advances from `signal-stretch-cache-v2`
to `signal-stretch-cache-v3`, and `SIGNAL_STRETCH_ENGINE_VERSION` advances to
`signal-native-stretch-v3`. Every `v2` artifact is invalid: it was keyed without
geometry or chunk policy, and its renderer predates the defect correction.
There is no migration, because a `v2` key cannot describe which render it
holds.

## 2026-07-27 Resumable Offline Render Addendum

Planning authority: `docs/roadmaps/g10/039-resumable-offline-stretch-render.md`.
Evidence: `docs/logs/2026-07/27-g10-039-state-boundary-audit.md`.

### Renderer state that resets at every boundary

Each segment and each chunk constructs a fresh renderer, so all of this starts
from zero at every join:

| state | role | cost of resetting |
| --- | --- | --- |
| `synthesis_phase` | accumulated output phase per bin | the join's phase relationship is arbitrary; this is the dominant defect |
| `previous_phase` | previous frame's analysis phase per bin | first frame after a join cannot propagate, so it re-initialises from analysis phase |
| `previous_magnitudes` | spectral-flux baseline | the transient detector cannot fire on the first frame after a join |
| `previous_energy` | energy-rise baseline | same |
| `frame_index == 0` branch | forces phase initialisation | guarantees the reset rather than merely allowing it |
| overlap-add and normalization buffers | windowed output accumulation | each render's windup and tail are cropped away, so joins carry no shared overlap |

The transient-detector resets are why a transient landing on the first frame
after a join cannot be detected. That is the leading hypothesis for the
low-end pops heard in both sides of the `g10.036` listening round.

### Measured boundary cost at the production default

A `60`-second stereo source at ratio `1.25` rendered through the artifact path
under the shipped `30`-second chunk policy, against the same source rendered as
one chunk:

| measurement | value |
| --- | --- |
| chunks | `2` against `1` |
| output frames | `3600000` both |
| correlation | `0.389976` |
| peak sample difference | `1.0752` |
| step at the seam sample | `-240 dBFS` production, `-45.14 dBFS` control |

The seam itself is flat, because the boundary smoother forces sample
continuity. The renders still diverge: continuity of value is not continuity of
phase. Any export longer than the chunk policy is affected today.

### Frozen boundary

A resumable offline renderer must satisfy all of the following.

Carried state. Synthesis phase, previous analysis phase, previous magnitudes,
previous energy, and the overlap-add accumulation must persist across chunk and
ratio-segment boundaries. A boundary must not force the `frame_index == 0`
initialisation branch.

Chunk-size independence. Output must be identical for any chunk policy over the
same source and curve. This is the acceptance law, and it is exact: a tolerance
would readmit the defect it exists to remove.

Memory bound. Working state must be a function of window size, hop, and channel
count only, never of source duration. Bounding by geometry is what makes the
renderer usable for exports of any length, which is the reason the chunk plan
exists.

The bound needs a maximum geometry to be a number. `with_window` currently
clamps only to a power of two at or above `64`, with no upper limit, so the
resumable renderer freezes a maximum window of `65536` frames. Every state item
is then derived:

| geometry | total, stereo |
| --- | --- |
| `2048` window, the retained default | `266300 B`, `0.254 MiB` |
| `65536` window, the maximum | `8519740 B`, `8.125 MiB` |

The frozen ceiling is `9 MiB`, which covers the maximum supported geometry in
stereo with `917444 B` of headroom. No term in the inventory references source
or output length.

The first published figure was `8 MiB` from an inventory that counted the
overlap-add and normalization rings but omitted the input ring. Three rings of
twice the window, not two, is the real cost. The ceiling is corrected here
rather than the renderer squeezed to meet a number that was wrong.

The whole-render overlap-add and normalization buffers are the only
duration-dependent state today, and they are what the resumable design replaces
with rings of twice the window. A renderer that still sizes any buffer from
source length has not met this bound.

Ratio curve ownership. The renderer consumes the ratio curve directly. Caller-
side segmentation is what creates the joins, so leaving it in place would defeat
the purpose; a state-carrying renderer needs the active ratio per analysis
frame, not a pre-cut list of independent spans. `plan_offline_stretch_chunks`
remains the bounded-memory authority and stops being a segmentation authority.

Seam mechanism. Once state is carried, the boundary smoother and the render-
plane chunk crossfade are removed rather than retained. If either is still
needed, the state is not actually being carried and the work is incomplete.

## Next Task

Execute `g10.037` Batch 37.2: explicit stable tokens, render geometry, chunk
policy, behavior version, and the schema advance. Keep the existing tier
behavior frozen apart from the defect corrections these addenda authorize.
Contract `085` has no admitted creative owner after `g10.031` closed explicit
`Cyclic`; no creative implementation may change this public tier surface
without new authority.
