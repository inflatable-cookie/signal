# Signal-Native High-Quality Stretch Program

Status: backlog
Created: 2026-07-05
Effort estimate: XL
Promotion trigger: Loophole g13 or another consumer needs Rubber Band-class
warp/stretch without adopting a GPL/commercial third-party engine.
Governing contract: `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`

## Decision

Pursue a first-party Signal-native time-stretch and pitch-shift engine. The
target is Rubber Band-class operation from the start, not a toy fallback.

Rubber Band may be used as a behavioral and listening benchmark only. Do not
use or translate its GPL source. Signalsmith Stretch may be studied as a
permissive comparator, not as the default answer.

## Current Inventory

Implemented Signal surfaces:

- `signal-render-plane` has RT-safe repitch/varispeed playback through
  `RenderSource::Warped`, implemented as a source-rate multiplier over the
  existing polyphase windowed-sinc media path.
- `signal-render-plane` offline render drives the same executor and render plan
  as realtime playback. Export parity is already the right integration shape.
- `signal-dsp-stretch` contains `TimeStretcher` plus
  `PhaseVocoderStretcher`, an offline whole-buffer draft phase vocoder.
- `signal-runtime` owns tempo-map projection, warp readiness, clip-processing
  snapshots, and media/cache readiness. It now uses Signal-owned `Stretch`
  vocabulary instead of a vendor-shaped draft mode name.
- Chorus ADR-001 makes musical ticks canonical for authoring, while media stays
  sample-addressed and render plans consume deterministic tick-to-sample
  projections.

Known gaps:

- no realtime pitch-preserving streaming stretcher
- no transient detector/reset path in the stretch crate
- no phase locking or vertical phase-coherence strategy
- no independent pitch API above stretch+resample composition
- no dynamic ratio automation path inside the stretcher
- no benchmark corpus, metric harness, or listening-evidence protocol
- no cache identity contract for high-quality offline stretch artifacts

## Target Backend Architecture

### Repitch

Owner: `signal-render-plane`.

Purpose: RT-safe varispeed. Tempo changes alter pitch. This is the existing
source-rate multiplier over `Samples` and `Stream`.

Constraints:

- no allocation, blocking, or frees on the audio thread
- deterministic sample indexing from compiled tick/sample projection
- source-rate conversion stays inside the existing render source path

### RealtimePreview

Owner: `signal-dsp-stretch` DSP, integrated through render-plane prework or a
bounded streaming state object only after RT safety is proven.

Purpose: pitch-preserving preview playback with bounded latency and dynamic
ratio changes. This tier may trade some fidelity for latency, but must still
preserve transients, stereo image, and musical timing well enough for editing.

Initial DSP shape:

- phase-vocoder foundation
- transient-aware phase reset or local time-domain splice strategy
- identity phase locking around spectral peaks
- latency reported as input and output halves for PDC/automation alignment
- ratio automation sampled against the same projected stream timeline that
  render plans use

### OfflineHighQuality

Owner: `signal-dsp-stretch`, consumed by render/export/cache services.

Purpose: deterministic high-quality stretch and pitch shift for export, freeze,
and post-warp artifacts.

Initial DSP shape:

- multiresolution STFT or hybrid STFT/time-domain engine
- transient detection and transient-preserving synthesis
- phase locking for vertical coherence
- shared stereo/multichannel analysis so image stability is measured and owned
- stretch+resample pitch-shift composition for accurate independent pitch
- deterministic dynamic-ratio rendering across tempo ramps and warp markers
- cache keys include engine version, tier, ratio curve, pitch curve, channel
  layout, source content hash, and tick/sample projection epoch

## Benchmark Corpus

Every tier must run a fixed corpus with both objective metrics and listening
notes:

- drums/percussion: close-mic loops, cymbals, kicks, dense transients
- bass: sustained electric bass, synth bass, plucked attacks
- vocals: dry speech, sung legato, breathy consonants, vibrato
- pads/sustains: dense harmonic pads, piano tails, reverb tails
- full mixes: mastered stereo, sparse acoustic, dense electronic
- tempo ramps: 90 to 140 BPM and 140 to 90 BPM over short and long spans
- loop seams: one-bar and two-bar loops, cross-boundary warp markers
- extreme ratios: 0.5x, 0.75x, 1.5x, 2.0x, plus out-of-support degradation

## Measurable Acceptance

Timing and alignment:

- offline fixed-ratio output length is exact to the promised sample count
- offline dynamic-ratio cumulative drift is no more than 1 sample per rendered
  segment boundary
- preview reports latency and keeps automation/ratio changes centered within
  the reported tolerance

Transient behavior:

- detected transient peaks remain within the tier tolerance after stretch
- transient smear is tracked by attack-time widening and peak-energy loss

Spectral and phase behavior:

- phasiness is tracked by spectral-modulation and inter-bin phase-coherence
  metrics across sustained material
- stereo image drift is tracked by inter-channel correlation and mid/side
  energy deltas

Boundary behavior:

- loop-boundary clicks stay below the fixed dBFS threshold for the corpus
- warp-marker seams are click-free or explicitly crossfaded by policy

Resource budgets:

- Repitch: current render-plane RT budget, no allocation on audio thread
- RealtimePreview: bounded latency, bounded memory, no unbounded per-block work
- OfflineHighQuality: deterministic output, bounded peak memory per channel
  and predictable CPU scaling by duration/channel count

## First Implementable DSP Path

Start in `signal-dsp-stretch`:

1. Split the current draft phase vocoder into explicit analysis, phase, and
   synthesis modules.
2. Add a mono offline high-quality prototype behind a new
   `OfflineHighQualityStretcher` constructor, still marked prototype until
   corpus acceptance passes.
3. Add spectral peak tracking and identity phase locking.
4. Add transient detection using energy/spectral-flux features, then reset or
   splice transients without copying Rubber Band implementation details.
5. Add linked stereo processing before this tier is promoted beyond prototype.
6. Add pitch shift as stretch+resample composition, not a separate hidden
   algorithm.
7. Add the benchmark harness before tuning thresholds, so improvement is
   measured against corpus evidence instead of ad hoc listening only.

## Ready Slice Queue

### Slice 1: Corpus Harness Foundation

Status: complete
Repos: `signal`

Work:

- [x] encode required material families in `signal-dsp-stretch`
- [x] encode fixed-ratio output-length drift measurement
- [x] encode lower-is-better metric limit assessment
- [x] add synthetic audio generators for ramp, seam, and extreme-ratio cases
- [x] add benchmark report output that can later compare draft,
  OfflineHighQuality, and external benchmark renders

Acceptance:

- [x] corpus blueprint covers drums/percussion, bass, vocals, pads/sustains,
  full mixes, tempo ramps, loop seams, and extreme ratios
- [x] metric assessment distinguishes pass, warn, and fail
- [x] synthetic cases run without file I/O
- [x] harness output is deterministic and suitable for future CI artifacts

Validation:

- `cargo test -p signal-dsp-stretch`

### Slice 2: Offline Phase-Locking Prototype

Status: complete
Repos: `signal`

Work:

- [x] split current phase-vocoder internals into analysis, phase, and
  synthesis helpers
- [x] add spectral peak tracking per frame
- [x] add identity phase locking around peak neighborhoods
- [x] keep current `PhaseVocoderStretcher` behavior available as the draft
  baseline for regression comparison

Acceptance:

- [x] fixed-ratio length contract remains exact
- [x] tonal pitch preservation does not regress against the draft baseline
- [x] sustained-material coherence metrics improve or log a measured gap
- [x] no new realtime render-plane path is introduced

Validation:

- `cargo test -p signal-dsp-stretch`
- `effigy check:docs` if public docs change

### Slice 3: Transient And Linked-Stereo Depth

Status: complete
Repos: `signal`

Work:

- [x] add transient detection over energy and spectral flux
- [x] add transient-preserving phase reset or local splice strategy
- [x] add linked stereo analysis/synthesis so image movement is measured
- [x] add transient smear metrics to the corpus harness
- [x] add loop seam metrics to the corpus harness

### Slice 4: Render, Cache, And Loophole Contracts

Status: in progress
Repos: `signal`; Loophole integration planning happens later in Chorus

Work:

- [x] define cache identity for engine version, tier, content hash,
  ratio/pitch curves, warp markers, channel layout, and projection epoch
- [x] add render/export/freeze artifact-planning seam that consumes the cache
  identity while keeping product-facing promotion gated
- [x] add runtime/export receipt seam that observes offline stretch artifact
  plans through observation and host supervisor reports
- [x] add explicit corpus-evidence promotion receipt for OfflineHighQuality so
  artifact planning no longer uses a boolean placeholder
- [x] expose an OfflineHighQuality prototype DSP path over the transient-reset,
  identity-locked phase-vocoder foundation while keeping promotion blocked
- [x] add synthetic corpus comparison reporting for OfflineHighQuality
  prototype metrics against the draft baseline
- [x] add a linked mid/side stereo stretch path for the OfflineHighQuality
  prototype so stereo metrics no longer rely on independent left/right renders
- [x] add independent pitch-shift composition for OfflineHighQuality prototype
  mono and linked stereo paths using Signal's band-limited resampler plus stretch
- [x] add stepwise dynamic-ratio automation for OfflineHighQuality prototype
  mono and linked stereo paths, with tempo-ramp timing evidence in the
  synthetic comparison report
- [x] add dynamic-segment seam click metrics to the synthetic comparison report
  so tempo-ramp evidence includes both timing drift and boundary artifacts
- [x] record explicit linked-stereo and pitch-shift evidence in the synthetic
  comparison report, including path labels and requested pitch semitones
- [x] add sustained-material coherence rows and a deterministic quality-priority
  report so the next DSP target is selected from measured regressions
- [x] apply the first priority-led transient tuning slice by routing
  time-compression ratios through phase locking instead of transient resets,
  reducing measured compression-smear regressions while keeping the tier
  prototype-gated
- [x] make missed transient matches a finite smear penalty so preservation
  failures are ranked by severity instead of becoming inconclusive report rows
- [x] add an explicit offline loop-boundary smoother and apply it to loop-seam
  candidate evidence, clearing the 0.5x loop-click priority row
- [x] set transient-smear comparison tolerance to one sample frame so the
  synthetic priority report remains driven by actionable quality regressions
- [x] add synthetic report threshold policy that converts the empty-priority
  comparison report into accepted or rejected `StretchPromotionReceipt` evidence
- [x] promote OfflineHighQuality artifact readiness from prototype-blocked to
  product-facing `Ready` only when render/runtime/host plans carry accepted
  promotion evidence
- [x] materialize the first ready OfflineHighQuality stereo PCM artifact for
  render-cache/freeze/export consumers through the existing `RenderSource::Samples`
  path, with unsupported pitch/dynamic-ratio combinations rejected explicitly
- [x] add render-plane materialization receipts and close the static pitch plus
  dynamic-ratio composition gap while keeping pitch automation explicitly
  unsupported
- [x] surface materialized artifact receipts through Signal runtime/host
  observation so planned and produced OfflineHighQuality artifact state report
  together
- [x] consume render-plane materialization receipts from the first Signal host
  edge freeze/export/cache bridge so runtime observation reports the receipt
  produced by real OfflineHighQuality PCM materialization
- [x] replace public-boundary accepted-promotion fixtures with the current
  synthetic comparison-policy receipt so readiness and materialization evidence
  come from the Signal report gate
- [x] add a reusable Signal artifact-builder gate that plans and materializes
  OfflineHighQuality render/export/freeze artifacts only from policy-derived
  promotion evidence, including a rejected-policy path that produces no
  product-facing buffer
- [x] route the public host-edge artifact consumer through a typed builder
  request, leaving direct receipt-based materialization as the lower-level
  render-plane test seam
- [x] add a render-plane helper that packages a policy-gated
  OfflineHighQuality artifact as `RenderSource::Samples`, with rejected policy
  still producing no product-facing source
- [x] route the public host-edge receipt path through the packaged
  `RenderSource::Samples` helper so render consumption and runtime
  materialization evidence share one policy-gated artifact object
- [x] wire a full render-plane export/freeze spec fixture through the packaged
  artifact source helper, including a rejected-policy fixture that cannot
  create a renderable clip source
- [x] add cache/source reuse evidence around the packaged artifact source path,
  proving identical policy-gated artifacts share stable cache identity while
  projection or curve changes produce distinct render sources
- [x] add a first render-cache handoff surface that returns cache identity,
  render source, and materialization receipt together for cache lookup/write
  decisions, with rejected policy producing no cacheable source
- [x] route the first Signal render-cache bridge through the handoff surface
  and add cache lookup/write decision evidence for accepted, rejected, hit,
  and invalidated identities
- [x] surface render-cache bridge decisions through Signal-owned
  receipt/observation edges so cache hits, writes, and invalidations can be
  audited alongside materialized artifact receipts
- [ ] wire OfflineHighQuality artifacts into render/export/freeze only after
  corpus evidence beats the draft baseline
- [ ] add Pulse/Aura contract changes only for product-visible mode, ratio,
  pitch, marker, and cache behavior

## Integration Plan

- Keep Signal as DSP owner. Pulse/Aura only need mode, ratio/pitch, marker, and
  render-cache contract additions when product workflow requires them.
- Keep render-plane realtime safety: preview stretch runs only through a proven
  bounded state object or through anticipative pre-rendered buffers.
- Offline export uses render plans plus cacheable post-warp artifacts. It must
  not fork an unrelated render path.
- Warp markers resolve through ADR-001 tick/sample projection. Engine inputs
  receive sample-domain source spans plus deterministic ratio/pitch curves
  derived from canonical ticks.
- Cache invalidation is owned by media identity, engine tier/version, marker
  map, ratio/pitch curves, source tempo, project tempo projection, and channel
  layout.

## Success Criteria

- Signal exposes `Repitch`, `RealtimePreview`, and `OfflineHighQuality` as
  first-party tiers with documented readiness and degradation.
- OfflineHighQuality beats the current draft phase vocoder across the corpus
  before any product-facing promotion.
- Rubber Band CLI/output can be used in the comparison harness as an external
  benchmark, with no source dependency or copied implementation.
- Exports, freezes, previews, and warp-marker renders share one engine
  vocabulary and cache identity model.
- Loophole-facing integration planning is recorded in Chorus only when a
  product workflow consumes the Signal-owned contract.

## Next Task

Continue the Signal DSP quality batch by turning the synthetic comparison
report into a promotion-readiness loop: add a render/export/freeze cache
decision fixture that records artifact plan, cache decision, materialization,
and render consumption in one observable Signal path. Keep Pulse/Aura contract
planning deferred until a Loophole product workflow consumes the Signal-owned
contract.
