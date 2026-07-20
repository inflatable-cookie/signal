# 085 Creative Time-Stretch Product And Routing Contract

Status: active PaulX-like `Dream`; source-relative candidate rejected at vector proof
Owner: core-product
Updated: 2026-07-20
Related contracts: `046`, `048`, `084`
Related architecture: `docs/architecture/offline-creative-time-stretch-study.md`,
`docs/architecture/offline-creative-source-relative-renewal-spectral-brief.md`,
`docs/architecture/offline-creative-audited-variance-compensated-renewal-spectral-brief.md`,
`docs/architecture/offline-creative-variance-compensated-renewal-spectral-brief.md`,
`docs/architecture/offline-creative-compensated-renewal-spectral-brief.md`,
`docs/architecture/offline-creative-renewal-spectral-brief.md`,
`docs/architecture/offline-creative-similarity-aligned-cyclic-brief.md`,
`docs/architecture/offline-creative-cyclic-grain-brief.md`
Related research:
`docs/research/specimen-dossiers/creative-stretch-source-triangulation.md`
Roadmap: `g10.031`

## Purpose

Freeze one reusable Signal boundary for intentional long-form creative stretch
without weakening the transparent `OfflineHighQuality` contract or forcing a
consumer to expose renderer-specific controls.

## Authority

- Contract `046` remains authoritative for `Repitch`, `RealtimePreview`, and
  `OfflineHighQuality`.
- Contract `084` remains closed for transparent successor work. Its rejected
  candidate families do not become active through this contract.
- Contract `085` owns creative intent, range routing, transition behavior,
  deterministic variation, and creative admission.
- Signal owns the engine and its semantic parameter vocabulary. A consumer
  owns layout, labels, percent-versus-duration display, and workflow placement.
- External software is comparator evidence only. No external production
  dependency or copied implementation expression enters Signal.

## Product Vocabulary

`CreativeStretch` means offline pitch-preserving expansion whose goal is useful
dreamy, smeared, cyclic, or cloud-like synthesis rather than transparent event
reconstruction.

The stable semantic request contains:

- exact target frame count
- output/input duration ratio
- `character`: `Dream`, `Spectral`, `Rough`, `Cloud`, or `Cyclic`
- normalized `motion`
- normalized `detail`
- normalized `space`
- deterministic seed or request for the identity-derived default

Target frames are authoritative. Ratio is derived or validated against that
target; inconsistent values are rejected rather than rounded into different
routing decisions.

`Dream` remains the intended default if the automatic router reopens. It means
smooth, fused, musical spectral smear.
`Spectral` intentionally exposes vocoder-like separation and decoherence.
`Rough` intentionally exposes a less smoothed polyphase texture. `Cloud` means
dispersed upper-range evolution. `Cyclic` means commanded Akai-style
repetition. Both complete cyclic candidates are rejected and deleted. Final
ownership reassessment found no third materially different, source-backed
whole-renderer path. The character is closed and unavailable.

No character is public today. A cyclic-only admission must not expose
unimplemented `Dream`, `Spectral`, `Rough`, or `Cloud` values, or imply that an
automatic range router exists.

These fields are intent, not transform controls. Public consumers must not
select FFT size, window, grain size, overlap, phase mode, internal renderer,
or transition weight.

`Transparent` and `CreativeStretch` are separate product choices. Creative
admission does not upgrade `OfflineHighQuality`, and transparent admission does
not authorize creative output.

## Rules

### Rule 1: one source/output map

Every internal owner uses the same monotonic source/output map and exact target
output lattice. Range selection may change synthesis character, not duration or
event-order truth.

A waveform owner may realize one bounded alignment displacement around that
ideal map at each synthesis launch only when the chosen source anchors remain
strictly increasing, the displacement is re-anchored to the ideal map rather
than accumulated, and the realized path is shared across linked channels. A
free-running adaptive cursor, backward anchor, or second event timeline
violates this rule.

### Rule 2: routing is versioned and deterministic

The automatic routed bands are paused:

- coherent: `1x` through `2x`
- coherent/diffusive overlap: `2x` through `4x`
- diffusive: `4x` through `16x`
- diffusive/cloud overlap: `16x` through `32x`
- cloud: `32x` through `100x`

These bands remain future product intent, not implementation authority. The
closed explicit `Cyclic` character bypassed them and targeted fixed expansion
above `1x` through `8x`. Any future reopening must preserve `2x`, `4x`, and
`8x` as mandatory admission points.

If automatic routing reopens, overlap weights use smoothstep interpolation over
`log2(ratio)`. A fixed-ratio request uses one constant channel-shared weight for
the whole render.

Changing the band map or blend law changes the creative routing version and
cache identity.

### Rule 3: the UI vocabulary stays stable across renderer changes

`duration`, `character`, `motion`, `detail`, and `space` retain their audible
direction across every range. An internal renderer may implement the macro
differently, but increasing a control must not reverse its semantic meaning at
a routing boundary.

Character values are semantic anchor regions, not external algorithm names.
The required initial anchors are:

- `Dream`: PaulXStretch-like smoothness and musical usefulness
- `Spectral`: CDP-like vocoder/decoherence character
- `Rough`: `Rrreeeaaa`-like conspicuous polyphase texture
- `Cyclic`: `ReaReaRea`-like repetition through `8x`

Signal may use different internal owners or blends to reach those regions.
Until more than one character is admitted, a consumer receives only the
available character and the shared macros, not disabled or fictional choices.

`seed` is advanced variation identity, not a continuous quality knob.

### Rule 4: seamless means measured continuity

A transition is not seamless merely because it crossfades. Both owners must
share target length, source cursor, boundary alignment, linked-channel weight,
and deterministic state.

Boundary probes must cover values immediately below, inside, and above each
overlap. Reject audible level steps, image jumps, timing discontinuity, new
clicks, or abrupt changes in motion density.

Dynamic-ratio routing remains unsupported until fixed-ratio owners and overlap
bands pass. A later dynamic path must carry state and slew weights; independent
segment concatenation is forbidden.

### Rule 5: stereo variation stays linked

Analysis decisions, source-position variation, routing weights, and
normalization are shared across linked channels. Per-channel synthesis may
preserve source-relative detail, but left and right must not draw unrelated
random trajectories.

Neutral `space` preserves mono. Duplicate stereo, swap, and polarity mechanics
must remain explicit structural gates. Independent linked-stereo listening is
required before promotion.

A candidate brief must state which of those gates are samplewise invariants
and which are relationship or listening invariants. It may not inherit an
exact algebraic tolerance from a rejected representation without showing that
the selected representation can express it and the product requires it.

### Rule 6: variation is reproducible

The same complete request and engine version produce byte-identical output on
the supported deterministic platform contract. Default variation derives from
the artifact identity. Rerolling produces a new explicit seed and artifact.

### Rule 7: exact boundaries and bounded state remain mandatory

Creative intent does not waive:

- exact target frame count
- finite output
- deterministic exterior padding and cropping
- bounded duration-independent working state, excluding source and output
- explicit chunk or artifact-writer bounds for long renders
- no audio-thread allocation, blocking, I/O, or execution

### Rule 8: cache identity includes creative intent

Before product-facing caching, identity includes at least:

- creative engine version
- routing version
- source content and channel layout
- exact target frames and ratio/map identity
- character, motion, detail, and space
- deterministic seed
- projection epoch and any pitch/warp inputs that affect output

Creative and transparent artifacts cannot collide.

### Rule 9: listening defines creative quality

Objective controls reject integrity and continuity failures. They do not decide
whether output is dreamy, evolving, musical, or useful.

The current cyclic lane requires concealed long-form listening at `2x`, `4x`,
and `8x`, with `4x` and `8x` primary. The pack covers percussion, bass, vocals,
pads/sustains, and full mix. `16x` remains a rejection-boundary probe, not a
supported target. Independent stereo review remains mandatory.

If the automatic router later reopens, its `Dream`, `Spectral`, and `Rough`
lane still requires `4x`, `8x`, and `16x`. `Dream` must remain the smoothest
and most generally musical centre. Exposed vocoder colour, rough periodicity,
or cyclic repetition in neutral `Dream` is rejection. `Spectral` and `Rough`
must remain deliberate, recognizable, stable destinations rather than one
degraded compromise.

Transparent transient-placement, replica, and tonal-fidelity limits are not
silently reused. Creative gates instead reject uncontrolled clicks, dropouts,
level changes, periodic flutter, metallic repetition outside `Cyclic`, static
freeze, stereo instability, and failure to map the semantic controls
consistently.

Every numeric creative-character gate must be calibrated against the retained
comparator row or identify a hard integrity boundary. Comparator metrics still
diagnose and reject; concealed long-form listening remains promotion authority.

### Rule 10: one complete candidate at a time

The independent-bin candidate was rejected for crest growth. Its first
continuous-excitation replacement was rejected at linked-relation admission.
The final direct-complex replacement then stopped at coefficient proof because
its exact anti-phase test required incompatible negation and swap outcomes.
Every branch and its scaffolding was deleted. No distribution, window,
coefficient, phase, smoothing, seed, assertion repair, or scalar sweep follows
these rejections. The current diffusive owner is closed.

Batch 31.9 range-owner reassessment rejected the retained coherent renderer as
a substitute for the PaulX-centred core and found no new complete source-backed
spectral family. At that stop, `Dream`, `Spectral`, `Rough`, `Cloud`, and
automatic routing stayed closed.

The separate cyclic reserve has operator value, a retained ReaReaRea target,
and public two-grain Akai-style architecture evidence. It became the next
owner study. Its first `CyclicGrain` candidate passed structural admission
but failed the first synthetic pitch row: `110 Hz` at `2x` measured
`111.328 Hz`, or `20.778` cents against the frozen `15`-cent ceiling. It was
deleted without correction or rerun. GPL source informs clean-room architecture
only; no expression, constant, or control flow enters Signal. Do not build the
full router as simultaneous experiments.

Cyclic ownership reassessment selected `SimilarityAlignedCyclic` for one new
complete brief. Unlike the rejected fixed lattice, it chooses each source
segment inside a bounded ideal-map neighbourhood by waveform similarity to the
prior segment's natural continuation. One strictly increasing selected path
and one channel-symmetric linked score own the render. This is source-backed by
the WSOLA paper and maintained SoundTouch architecture, but it is not a claim
of transparent or arbitrary-polyphonic quality. Echoing, drift, one-lag
polyphonic compromise, event replicas, and linked-image motion remain explicit
rejection risks. No source expression, constants, search schedule, or tuning
transfers to Signal.

The frozen brief now owns one exact rational nominal map, one regular output
lattice, one bounded two-stage zero-mean correlation search, one strictly
increasing source-anchor path, one low-confidence fallback, unit-rate native
reads, complementary overlap synthesis, shared linked-channel decisions,
exact length, `4 MiB` state, deterministic cost, fixed gates, and complete
cleanup. Candidate implementation may not choose another WSOLA variant or
alter those laws.

That candidate passed compile-only validation but failed structural
known-offset recovery. Its coarse shortlist excluded the exact continuation at
source frame `6352`; full refinement selected frame `6432`. It was deleted
without correction or rerun.

Final ownership reassessment closes explicit `Cyclic`:

- fixed or unaligned overlap-add is the rejected `CyclicGrain` owner
- SOLA/WSOLA search changes are repairs to the rejected similarity-aligned owner
- pitch- and epoch-synchronous methods lack one full-mix, linked-channel period
  owner and retained `8x` evidence
- transient-managed, component, spectral, sinusoidal, and learned hybrids
  reopen a closed seam or require a separate research program

No third cyclic implementation is authorized. New complete-system evidence or
an explicit operator decision is required before any creative owner reopens.

Batch 31.16 is that explicit research reopening for neutral `Dream`. Pinned
PaulXStretch source shows one complete family not directly tested by the
rejected briefs: long-window magnitude analysis, deliberate source-phase loss,
per-frame stochastic phase renewal, and output-frame crossfade. The rejected
`DiffuseSpectral` brief instead added an instantaneous-frequency carrier,
correlated diffusion, log-magnitude evolution, and a different overlap law.
The later continuous-excitation candidates added recurrence rather than
testing phase renewal.

This new evidence authorizes one docs-only `RenewalSpectral` brief. It does not
restore a rejected branch, authorize candidate DSP, or reopen `Spectral`,
`Rough`, `Cyclic`, `Cloud`, routing, cache, or product exposure. The brief must
freeze exact length, bounded state, deterministic phase renewal, crest-owning
frame synthesis, and one Signal-owned linked-channel rule before implementation
can be considered.

Batch 31.17 freezes that complete brief. It selects one sample-rate-normalized
long transform, exact sample-centred map, magnitude-only mono or mid/side
analysis, counter-addressed per-frame phase renewal, pairwise equal-power frame
crossfade, fixed energy calibration, linked `space`, bounded exterior envelope,
and exact crop. `motion` and `detail` are explicitly absent from the private
candidate request. Every character metric is tied to PaulXStretch or a named
hard integrity boundary. No implementation is admitted by the brief.

Batch 31.18 implemented the frozen brief once. Compile-only validation and the
complete structural gate passed. The mandated first neutral-`Dream` crest row
then measured `8.263162 dB` of crest-factor growth against the frozen `6 dB`
ceiling. Complete independent phase renewal still leaves cross-bin waveform
summation uncontrolled. The candidate stopped without correction or rerun and
was deleted before later synthetic or listening gates.

Batch 31.19 closed neutral `Dream`. Its two independent-phase candidates both
failed the same crest boundary despite materially different carriers, envelope
evolution, and synthesis overlap. Public low-crest multisine and IAAFT methods
jointly optimize synthetic phase but do not provide one nonstationary musical
time map, linked-stereo law, fixed bounded cost, and retained long-form target.
STN noise morphing is a residual component, not a complete first-party owner.
Signal's complete continuous-excitation translation already failed linked
relation ownership.

Batch 31.19 authorized no neutral-`Dream` implementation and prohibited
reopening through a limiter, post-gain stage, phase or window substitution,
scalar sweep, or fusion of rejected owners. The following operator correction
and matching-reference evidence supersede that target closure without
restoring either rejected candidate.

The operator superseded that target closure. The `3.88 dB` PaulXStretch
calibration came from retained musical rows, while `RenewalSpectral` stopped on
synthetic uniform noise before the matching PaulXStretch synthetic suite ran.
Candidate rejection remains valid under its frozen brief; family closure does
not follow from that unmatched row.

PaulXStretch's whole path also includes position-dependent compensation around
its adjacent-frame blend. Signal's rejected translation substituted an
equal-power blend and fixed gain. A new clean-room brief may derive a bounded
overlap-statistics compensation law and matching-reference gates. It may not
copy upstream constants, expressions, or control flow.

Neutral `Dream` remains an active product goal. Work stays one complete
candidate at a time. Candidate failure requires diagnosis and a new complete
brief, but does not close the product target unless the operator explicitly
does so. Hard integrity can stop a render. Character promotion or rejection
must include reference-matched evidence and long-form listening.

Batch 31.20 rendered the same frozen synthetics through the pinned
PaulXStretch `1.6.0` core. Its worst-channel uniform-noise crest growth is
`9.932`, `11.899`, and `10.432 dB` at `4x`, `8x`, and `16x`. The old `6 dB`
ceiling is not reference-calibrated; the rejected Signal `8.263162 dB` row is
below the matching PaulX `4x` row.

The rejected-at-compile `CompensatedRenewalSpectral` brief froze the clean-room
topology. Fresh `VarianceCompensatedRenewalSpectral` authority retains Signal's
exact map, long magnitude analysis, phase renewal, and linked mid/side law. It
uses one raised-cosine adjacent-frame blend and compensation derived from
`1/sqrt(a^2+b^2)`, the variance of two equal-energy uncorrelated frames. The
law is bounded from `1` through `sqrt(2)` and copies no upstream coefficient or
control flow.

Hard integrity remains absolute. Crest, pitch, replica, modulation, and gap
character are compared row-for-row with the matching PaulX synthetic.
Concealed long-form listening remains promotion authority.

Batch 31.21 implemented the frozen brief once, but compile-only validation
failed on an unconstrained structural-test `Option` accumulator before the
renderer executed. The candidate was deleted without correction or rerun. The
result is implementation evidence only: the compensated-renewal DSP remains
untested. No renderer is admitted.

Batch 31.22 freezes `VarianceCompensatedRenewalSpectral` as the fresh complete
candidate identity. The DSP topology and all acoustic gates are unchanged.
Construction now ends with one clean `effigy test compile` receipt and a local
isolated checkpoint. Compiler-only type, import, visibility, ownership, and
test-assembly repairs may occur before that receipt only when they do not
change DSP or evidence semantics. Structural admission and every later gate
remain one-shot and terminal from the recorded checkpoint.

Batch 31.23 compiled that isolated implementation and passed seven structural
tests. Its synthetic command returned green, but pre-listening review found
that required impulse-train crest, secondary-region, exact-lag
autocorrelation, and full discontinuity assertions were absent. Impulse width
and silence-gap measurement expression were also not frozen tightly enough to
guarantee reference parity. The receipt is invalid. The unopened pack and
candidate were removed without repair or rerun. No DSP or listening decision
exists for the topology.

Batch 31.24 retains the still-untested topology under fresh identity
`AuditedVarianceCompensatedRenewalSpectral`. Its complete brief freezes one
compile-linked owner for every gate, actual allocation accounting including
FFT plans, exact source and metric definitions, corrected shortest-interval
impulse references, exact sample-centred event placement, full-lag
autocorrelation, explicit replica-region semantics, and a PaulX-relative
first-difference crest control. Candidate source may not be recovered from the
deleted implementation.

Batch 31.25 produced the first valid end-to-end evidence for the compensated
renewal topology. Compile, construction `1/1`, structural `13/13`, and
synthetic `9/9` passed from immutable checkpoint `97ee7056`. Concealed mono
listening passed as `15/15` ties against PaulXStretch with no unusable row or
family loss.

Linked-stereo speaker review then exposed a persistent source-relative channel
imbalance. A retained full-mix source measured `-0.4516 dB` right-minus-left;
candidate `8x` output measured `+4.2147 dB`, `+3.3660 dB`, and `+1.9453 dB`
at `space=0`, `0.5`, and `1`. The first-exactly-non-zero orientation law used
mid and side samples more than `141 dB` below component peak to choose a
render-wide polarity after discarding their source phase relationship. This
violates source-image and centre-stability ownership. Objective evidence and
operator listening reject the candidate before independent promotion review.

The rejection does not invalidate the mono renewal topology or close neutral
`Dream`. A successor may retain its map, transform, magnitude renewal, frame
blend, variance compensation, boundaries, and passed gates. It must replace
the failed stereo orientation law with one explicit source-relative relation
owner and add source-relative channel-balance admission. It may not apply
post-render channel gain, threshold the first sample, or patch this checkpoint.

Batch 31.26 freezes that successor as `SourceRelativeRenewalSpectral`. It keeps
the passed mono path, but analyzes native left/right complex spectra. One
counter phase renews each linked pair while per-channel magnitudes remain
source-owned. Neutral `space` preserves the analyzed interchannel relation;
increasing `space` widens only non-zero coherent relations toward quadrature
above the protected low band. Duplicate mono, anti-phase, common polarity,
swap relationships, whole/band/window balance, exact boundaries, bounded
state, determinism, and independent stereo listening are explicit gates. No
candidate DSP entered `main`.

Batch 31.27 implemented that authority once from fresh source. Compile and
construction `1/1` passed at checkpoint `1f05cc33`. Structural admission
selected `15` owners; `14` passed. The mono-renewal owner then exposed a frozen
evidence-vector defect: the normative `mix64(1)` formula returned
`0x5692161d100b05e5`, while the assertion expected the transposed value
`0x569216d1009b05e5`. No assertion repair or rerun is permitted after the
checkpoint. The candidate was rejected and deleted before synthetic or
listening gates. This is no acoustic or stereo result.

## Initial Promotion Sequence

1. Comparator capture and target-character freeze. Complete.
2. Rejected independent-bin brief and architecture reassessment. Complete.
3. Complete `ContinuousExcitationSpectral` replacement brief. Complete.
4. Isolated fixed-ratio structural candidate. Rejected on common-polarity
   covariance before creative synthetic controls.
5. Linked-relation architecture reassessment. Complete.
6. Final complete brief and isolated candidate. Rejected at relation proof.
7. Creative range-owner reassessment. Complete; automatic router paused and
   cyclic-first promise selected.
8. Freeze one complete clean-room cyclic-owner brief. Complete.
9. Implement one isolated cyclic candidate. Rejected at the first creative
   synthetic pitch row after structural admission.
10. Reassess cyclic ownership at architecture level or close the character.
    Complete; similarity-aligned waveform overlap selected.
11. Freeze one complete `SimilarityAlignedCyclic` brief without candidate DSP.
    Complete.
12. Implement one isolated candidate and stop at the first failed structural
    or synthetic gate. Complete; rejected at structural known-offset recovery.
13. Reassess cyclic ownership and select a genuinely different complete path
    or close the character. Complete; no third path found, character closed.
14. Retained mono and independent stereo listening. Closed without admission.
15. Minimal cyclic admission, product exposure, and cache review. Closed
    without admission.
16. Pinned PaulXStretch, CDP, and Potenza whole-path source triangulation.
    Complete; one materially different neutral `Dream` family selected.
17. Freeze one complete `RenewalSpectral` brief without candidate DSP.
    Complete.
18. Implement the brief once in the named disposable worktree and stop at the
    first failed gate. Complete; rejected on the first crest row after
    structural admission.
19. Reassess neutral-`Dream` crest ownership at architecture level or close the
    owner. Superseded; closure used unmatched musical-versus-synthetic crest
    evidence and exceeded the operator decision.
20. Capture matching PaulXStretch synthetic rows, repair the gate, and freeze
    one complete clean-room frame-blend-compensated brief. Complete; no
    candidate DSP entered `main`.
21. Implement `CompensatedRenewalSpectral` once in the frozen disposable
    worktree and run gates in order. Complete; rejected at compile-only
    validation before renderer execution.
22. Freeze fresh complete candidate authority for the still-untested
    compensated-renewal topology. Complete; no candidate DSP entered `main`.
23. Implement `VarianceCompensatedRenewalSpectral` once, complete construction
    and compile validation, freeze one isolated checkpoint, then run admission
    gates in order. Complete; synthetic receipt invalidated before listening
    because required assertions were absent.
24. Reconcile evidence ownership and freeze one fresh complete brief for the
    still-untested topology. Complete; no candidate DSP entered `main`.
25. Implement `AuditedVarianceCompensatedRenewalSpectral` once, pass compile
    and construction-manifest validation, freeze one checkpoint, then run
    structural and synthetic gates in order. Complete; construction passed
    `1/1`, structural passed `13/13`, synthetic passed `9/9`, and concealed
    mono listening passed `15/15`. Rejected at linked-stereo image preservation
    after a persistent rightward balance inversion.
26. Freeze one complete source-relative stereo renewal successor brief without
    candidate DSP. Complete; `SourceRelativeRenewalSpectral` frozen.
27. Implement the brief once in its named disposable worktree, complete
    construction `1/1`, freeze one checkpoint, and run `15` structural then
    `9` synthetic owners. Complete; rejected at structural exact-vector proof
    after `14/15` passes.
28. Reconcile executable vector ownership, audit every exact construction
    vector, and either freeze fresh complete candidate authority or close the
    topology. Ready; docs and architecture only.

`Spectral`/`Rough`, coherent overlap, `LayeredCloud`, the upper overlap, dynamic
ratios, and automatic routing still require separate reopening decisions backed
by new complete-system evidence.

## Current State

Six isolated spectral candidates and both cyclic candidates are rejected and
deleted. Explicit `Cyclic` and the automatic router remain closed or paused.
Neutral `Dream` is active. Matching PaulX synthetics invalidate the old crest
calibration. The first complete `CompensatedRenewalSpectral` implementation
failed compile-only validation before DSP execution and was deleted.
`VarianceCompensatedRenewalSpectral` later produced an invalid synthetic
receipt and was deleted before listening. The fresh
`AuditedVarianceCompensatedRenewalSpectral` checkpoint passed compile,
construction, structural, synthetic, and concealed mono gates without repair
or rerun. It was rejected at linked-stereo image preservation because its
first-sample component orientation inverted source-relative channel balance.
The passed mono renewal core remains active architectural evidence; the failed
stereo law does not. The first `SourceRelativeRenewalSpectral` checkpoint is
also rejected and deleted after a frozen `mix64` vector typo stopped structural
admission at `14/15`. Its renderer never reached synthetic or listening
evidence. Native left/right relation ownership remains untested architecture,
not candidate authority.

No public Rust enum, renderer, harness mode, fixture, artifact schema, runtime
route, or product-facing claim entered `main`. `OfflineHighQuality` remains
byte-exact and Contract `084` remains closed. No creative renderer is admitted.

## Next Task

Run Batch 31.28 only. Reconcile the incorrect executable vector with the
normative formula, audit every exact construction vector, and either freeze
fresh complete authority under a new candidate identity or close the topology.
Do not implement candidate DSP in the same batch.
