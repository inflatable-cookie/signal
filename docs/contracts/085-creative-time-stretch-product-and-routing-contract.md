# 085 Creative Time-Stretch Product And Routing Contract

Status: active PaulX-like `Dream`; linked STN transform-error resume ready
Owner: core-product
Updated: 2026-07-22
Related contracts: `046`, `048`, `084`
Related architecture: `docs/architecture/offline-creative-time-stretch-study.md`,
`docs/architecture/offline-creative-linked-stn-noise-morph-brief.md`,
`docs/architecture/offline-creative-comparator-audited-renewal-spectral-brief.md`,
`docs/architecture/offline-creative-verified-source-relative-renewal-spectral-brief.md`,
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

For creative `Dream`, finite output, exact length, deterministic repeat,
duplicate/mono mechanics, common polarity, anti-phase, swap relationship,
per-bin channel magnitude, whole-render balance, three-band balance, and
declared `space` direction are hard stereo controls. Local mapped-window
source-relative balance and dominance are mandatory diagnostics. They do not
carry a terminal numeric threshold because the preferred PaulX target uses
separate per-channel phase renewal and does not promise local source-relative
waveform balance. Comparator-relative review by an eligible independent
stereo listener is terminal and cannot be waived by objective metrics.

A candidate brief must state which of those gates are samplewise invariants
and which are relationship or listening invariants. It may not inherit an
exact algebraic tolerance from a rejected representation without showing that
the selected representation can express it and the product requires it.

### Rule 6: variation is reproducible

The same complete request and engine version produce byte-identical output on
the supported deterministic platform contract. Default variation derives from
the artifact identity. Rerolling produces a new explicit seed and artifact.

Every stochastic candidate brief must freeze one exact `ADMISSION_SEED` before
implementation. Synthetic and listening helpers may not choose an implicit or
local seed. A fixed-seed candidate pass does not admit product seed/reroll
exposure; that requires a later frozen multi-seed character review.

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
comparator row or identify a hard integrity boundary. Hard integrity,
replica-region, level, discontinuity, dropout, deterministic-state, boundary,
and the explicitly named hard linked-stereo gates remain terminal.

For neutral `Dream`, whole-render and three-band candidate-source balance
remain hard at `0.75 dB`, balance spread across `space=0`, `0.5`, and `1`
remains hard at `0.50 dB`, and whole/band dominance reversal remains terminal
when source balance magnitude is at least `0.50 dB`. Four-second mapped-window
balance with two-second hops must record complete candidate-source and
PaulX-source error and dominance. Missing or non-finite evidence rejects the
receipt; finite local error and reversal are diagnostic for independent
listening rather than numeric rejection.

For neutral `Dream`, sustained pitch error versus the matching PaulX row is a
mandatory diagnostic, not a terminal comparator threshold. The frozen
estimator must complete every tone and chord row and record both errors and
their delta. Concealed long-form listening decides whether finite measured
tonal deviation is objectionable. A missing or non-finite measurement remains
an evidence failure. This exception does not weaken transparent Contract `046`
or `084` tonal admission.

`Y08` owns separate discontinuity and dropout ranges. First-difference crest
uses the frozen comparator range, including complete output for the isolated
impulse. Dropout uses the mapped authored-support hull only. With source length
`L`, target length `T`, and source half-open support `[a,b)`, map that hull to
`[floor(a*T/L),ceil(b*T/L))` with checked `u128` arithmetic and clip it to
endpoint domain `[0,T]`. The frozen synthetic supports are:

| Source | Source support `[a,b)` |
| --- | --- |
| low tone, mid tone, chord, harmonic pad | `[24000,72000)` |
| silence gap | `[24000,72000)` including its intentional interior gap |
| uniform, Rademacher, amplitude-modulated noise | `[24000,72000)` |
| impulse | `[48000,48001)` |
| impulse train | `[19200,77798)` |

At exact `4x`, `8x`, and `16x`, those endpoints are integer multiples of the
source endpoints. Dropout examines only complete `H`-sample windows wholly
contained in the mapped hull. A hull shorter than `H` has no eligible dropout
window; the assertion passes vacuously. `Y03` owns isolated-impulse spread and
placement, while `Y04` owns impulse and impulse-train replicas. Expanding the
dropout scan to complete impulse output changes the gate and invalidates that
receipt; it is not renderer evidence.

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

Batch 31.28 independently derived the counter table in Python and Ruby. Every
tag, both `mix64(1)` intermediate rounds, finalizer control, full address
stage, and high-53-bit numerator agreed. The rejected candidate had only one
handwritten exact counter assertion; no second mismatch exists to reconcile.

Fresh authority is `VerifiedSourceRelativeRenewalSpectral`. Its DSP,
structural, synthetic, mono, stereo, cleanup, and admission boundaries are
unchanged. Candidate tests must own all exact counter literals once in a named
table; construction validates that table before checkpointing, and later tests
may not carry duplicate handwritten values. No candidate DSP entered `main`.

Batch 31.29 implemented that authority fresh. Checkpoint `d94612dd` passed
compile, construction `1/1`, and structural `15/15`, then completed synthetic
admission at `7/9`. One `16x` replica row and two `4x` pitch rows failed. The
candidate was deleted before listening without correction or rerun.

Batch 31.30 withdraws the closeout's fixed-resolution range diagnosis. Batch
31.25 and Batch 31.29 own the same normative mono DSP, sources, and metrics,
but their briefs froze no synthetic candidate seed. Batch 31.29 used seed `17`;
the earlier passing receipt did not record its seed. Pinned PaulX also uses one
renewal path across all retained ratios. The failed checkpoint remains
rejected, but its stochastic rows cannot select a range switch or reject the
source-relative topology.

Batch 31.30 froze `SeedAuditedSourceRelativeRenewalSpectral`. It retained the
complete verified topology and gates under a new worktree, branch, module, and
prefix identity. The existing audited address vector's seed was the sole named
`ADMISSION_SEED` for every synthetic and listening candidate render. Candidate
DSP, routing, public seed control, and product exposure remained absent.

Batch 31.31 implemented that authority once. Checkpoint `790119b7` passed
compile, construction `1/1`, and structural `15/15`. Synthetic admission
selected all nine owners. Six passed before `Y02` failed the `8x` chord at
`13.351828347` cents against its `11.331375778`-cent ceiling; `Y08` and `Y09`
were cancelled. `Y04` passed at every ratio. The candidate was deleted before
listening without correction or rerun.

The audited seed resolves the predecessor's replica miss but not tonal pitch.
Batch 31.29 and Batch 31.31 now fail the same pitch class across different
seeds, material, and ratios. Contract `084` Rule 7 requires an architecture
reassessment. No source-relative renewal candidate remains active.

Batch 31.32 completes that reassessment and closes the renewal family. Its
magnitude-only stochastic frames have no phase-continuous tonal state, so a
seed, transform, hop, window, blend, threshold, or scalar change cannot own the
repeated finite-render pitch failure. Adding recurrence, tracked peaks,
oscillators, material separation, or learned synthesis would select a different
family and may not be smuggled in as renewal repair.

No retained or newly checked public family qualifies as the next complete
creative owner. Signalsmith's coherent predictor explicitly falls back to
randomized observations above `2x`; Bungee and Rubber Band reopen prohibited
peak or material-state work; pinned SBSMS already failed whole-renderer
feasibility; STN and public toolboxes separate independently rendered
components; and TSM-Net depends on pretrained model state without a released
training path, usable repository licence, or intrinsic linked pitch law.

This closes an implementation family, not the operator's PaulX-like `Dream`
goal. Reopening requires new complete-system evidence that jointly owns tonal
coherence, creative diffusion at `4x`, `8x`, and `16x`, linked channels, exact
length, determinism, and bounded state, or an explicit operator change to a
governing product boundary.

The operator made that boundary change after Batch 31.32. Prior renewal
checkpoints remain rejected under their frozen gates, but the repeated finite
PaulX-relative pitch delta no longer defines a terminal creative failure class.
Batch 31.25 already passed concealed mono as `15/15` ties, and operator speaker
review found solid stereo character apart from the balance inversion that the
later native-channel law was designed to remove.

Batch 31.33 authority was `ListeningLedSourceRelativeRenewalSpectral`. It
retains the seed-audited source-relative renderer and every terminal gate.
`Y02` must render and report the complete pitch matrix but cannot reject a
finite row for exceeding PaulX error plus `2` cents. This authorized one fresh
candidate only; it did not revive deleted source, waive listening, admit
product controls, or claim tonal parity.

Batch 31.34 implemented that authority once. Checkpoint `f76d5bb7` passed
compile, construction `1/1`, and structural `15/15`; synthetic admission
finished `8/9`. `Y02` completed its diagnostic. `Y08` failed because the test
scanned complete impulse output for dropout. The candidate was rejected and
deleted before listening.

Batch 31.35 reconciles that receipt against the audited brief and Batch
31.25's passed `Y08`. Complete impulse output belongs only to discontinuity
crest. Dropout belongs to the mapped authored-support hull. The isolated
impulse hull is shorter than `H` at every admitted ratio, so no dropout window
exists. The failed executable broadened the gate; it did not demonstrate
renderer dropout. Fresh support-audited authority is frozen without changing
DSP or any terminal threshold.

Batch 31.36 implemented that authority once. Checkpoint `5d8eaf45` passed
compile, construction `1/1`, structural `15/15`, synthetic `9/9`, and
concealed mono as `15/15` ties. The valid exact-source stereo gate rejected
all three `16x` full-mix rows at `9.37..9.42 dB` mapped-window balance error
with local channel-dominance reversal; `16x` bass also missed at about
`2.00 dB`. Whole-render and band balance remained close. The candidate was
deleted before speaker or independent stereo listening.

Batch 31.37 closes renewal under the current stereo contract. The failed
native-channel law already preserves current-frame magnitudes and exact complex
relation at `space=0`; independent frame renewal and synthesis blending leave
successive-waveform interference unowned. Every reviewed source-backed
temporal correction selects another family: coherent phase prediction,
predecessor peak trajectories, or paired oscillators. Those paths are already
incomplete, closed, or source-feasibility rejected. Post-hoc gain, covariance,
consistency, relation smoothing, and phase variants remain unauthorized.

This closes an implementation family, not the PaulX-like target. PaulX's own
separate channel engines do not satisfy Signal's hard local source-relative
invariant. Changing that invariant to a diagnostic under comparator-relative
independent listening is an explicit operator product decision. It cannot
reinterpret or recover a rejected checkpoint.

The operator made that product decision after Batch 31.37. For neutral
creative `Dream`, mapped local source-relative balance and dominance are now
diagnostic. Hard integrity, structural stereo relations, whole-render and
three-band balance, `space` consistency, and complete evidence remain
terminal. Concealed comparator-relative stereo review by an eligible
independent listener is promotion authority. The operator may still reject a
speaker pre-screen but cannot supply the required independent pass.

Batch 31.38's fresh authority was `ComparatorAuditedRenewalSpectral`. It
retained every Batch 31.36 renderer formula and hard control, changed only the
explicit stereo gate classification, and added PaulX-source values to the
mapped-window diagnostic. The deleted checkpoint remained rejected.
Implementation had to start from the brief in a new isolated worktree without
recovering prior code.

Batch 31.39 completed that implementation and rejected it at synthetic
admission. `Y04` failed one `16x` replica row and `Y09` failed linked-stereo
swap at `4x` and `8x`; the full result was `7/9`. The candidate was deleted
before listening. Batch 31.36 passed both owners under the nominally same
renderer and admission seed, so no further implementation is authorized until
the contradictory receipts are reconciled as authority and evidence.

Batch 31.40 found no retained executable identity capable of that
reconciliation. Construction proved owner inventory, counter vectors, seed,
and support tables, not helper or assertion equivalence. `Y04`'s `-30 dB`
value is an active-window threshold and its frozen result is one region with
no secondary. `Y09` never froze one exact source-relative long-form swap
assertion after exact time-domain swap was explicitly disclaimed. Deleted
candidate source, helper bodies, per-row results, and output digests are not
available and must not be recovered.

Each receipt remains terminal only for its own checkpoint. Neither proves the
other checkpoint or the complete renewal topology. A new executable authority
would be a third renewal candidate program, not evidence reconciliation. With
no materially different source-backed renewal owner, that path is closed.

### Rule 11: conformance precedes acoustic identity

This rule applies prospectively to work authorized after Batch 31.55. It does
not change any historical checkpoint result or restore deleted source.

Creative candidate work has three distinct states:

1. **Working implementation.** One isolated worktree implements one frozen
   complete architecture. Compile, manifest/construction, and structural
   conformance may run repeatedly. This state is not an acoustic candidate
   receipt.
2. **Conformance-complete tree.** Compile, construction, and the complete
   structural suite pass together. The worktree is clean and every candidate,
   test, helper, source-table, dependency, toolchain, and platform identity is
   recorded.
3. **Acoustic checkpoint.** The conformance-complete tree is committed and
   referenced immutably before any synthetic acoustic gate, rendered
   comparator review, or listening output runs. This is the sole candidate
   identity used by one-candidate and repeated-failure rules.

Before implementation starts, the canonical brief must freeze the complete
renderer plus every structural and acoustic source, seed, helper algorithm,
metric, threshold, assertion, comparator row, listening pack, and gate order.
Executable acoustic owners must compile during conformance but may not run or
produce inspectable renders before the acoustic checkpoint.

Before that checkpoint, corrections are allowed only when they make code or
tests conform to existing canonical authority. Compiler, type, visibility,
ownership, allocation, tie, boundary, state-machine, and exact-vector defects
may be corrected and conformance rerun. Every round records failed owners and
the corrective diff. A correction that requires choosing or changing a DSP
formula, source, seed, helper algorithm, metric, threshold, assertion,
comparator, or listening policy stops for docs-level reassessment. No acoustic
output may guide conformance work. No parameter, coefficient, window, phase,
seed, or scalar sweep is allowed.

When such a stop occurs before any acoustic checkpoint, a docs-only
reconciliation may resume the retained working implementation without creating
a new candidate identity only when all of these are true:

- the stop commit, tree, failed owner, and complete conformance ledger remain
  retained and immutable
- no synthetic gate, rendered comparator, listening output, or acoustic ref
  exists
- canonical authority freezes one exact correction before further execution
- the docs closeout commit is applied to the isolated worktree
- the resumed tree starts clean and reruns the complete compile, construction,
  and structural sequence twice before creating an acoustic ref

The resumed implementation may change only the newly frozen authority and its
direct conformance owners. It may not use prior partial passes as admission,
inspect acoustic output, sweep the correction, or alter any unrelated renderer
or gate rule. If executable state was deleted or cannot be proved identical,
the resume exception does not apply.

After the acoustic checkpoint, the order is fixed:

1. synthetic acoustic and integrity admission
2. concealed long-form mono comparison
3. independent comparator-relative stereo admission
4. fixed-ratio promotion decision
5. dynamic-ratio, routing, cache, and product review only after promotion

Each stage runs from the same checkpoint and stops later stages on failure.
No source, renderer, test, helper, assertion, or threshold changes are allowed.
Listening remains promotion authority.

Create a local immutable ref under
`refs/signal-evidence/creative/<family>/<checkpoint>` when acoustic identity is
frozen. On failure, delete the worktree, branch, build state, and generated
renders after the receipt is recorded, but retain that ref through the required
reassessment so exact source and test bodies remain comparable. Delete the ref
when reassessment closes the evidence question. Evidence refs are local-only,
never production branches, releases, or push authority. Rejected source never
enters `main`.

A family closed only on pre-acoustic compile, construction, structural, or
evidence-assembly failure may be considered once under this protocol when:

- no synthetic or listening gate executed
- its canonical architecture remains complete and source-backed
- a docs-only owner-selection batch explicitly chooses it
- implementation starts fresh without recovering deleted source
- one new acoustic checkpoint identity owns the entire future receipt

This does not revive a rejected checkpoint. A family with a synthetic,
long-form, stereo, or listening failure remains closed until new complete
architecture or an explicit evidence-backed product-gate change addresses that
failure class. Two acoustic checkpoints failing the same dominant cause still
trigger architecture reassessment under Contract `084` Rule 7.

#### Batch 31.56 owner selection

Materially different renderer owners have separate rows. Identities that
changed only evidence construction, seed ownership, gate classification, or
candidate isolation stay with their parent DSP family. A later
conformance-only identity does not erase an earlier acoustic result from that
family.

| Closed family | Highest valid stage | Rule 11 class | Decision |
| --- | --- | --- | --- |
| `DiffuseSpectral` | structural pass, then synthetic crest rejection | acoustic rejection | closed |
| `ContinuousExcitationSpectral` | `12/13` structural; linked relation failed | conformance-only, superseded | ineligible; current brief does not own the failed relation |
| `ContinuousExcitationComplexRelation` | coefficient relation proof exposed an impossible anti-phase negation/swap expectation | conformance-only, contradictory | ineligible; correction requires new relation authority |
| `CyclicGrain` | structural pass, then synthetic pitch rejection | acoustic rejection | closed |
| `SimilarityAlignedCyclic` | `6/7` structural; frozen shortlist could not reach the exact natural continuation | conformance-only, architecture miss | ineligible; correction changes the search owner |
| `RenewalSpectral` | structural pass, then synthetic crest rejection under the superseded unmatched gate | acoustic receipt, superseded architecture | ineligible; corrected source-backed blend belongs to the compensated family |
| compensated/variance/audited renewal | structural and synthetic pass plus concealed-mono pass, then stereo image rejection | acoustic and stereo rejection | closed |
| source-relative/listening-led/support-audited/comparator-audited renewal | support-audited identity passed structural, synthetic, and concealed mono, then failed stereo; after stereo policy changed, comparator-audited identity failed synthetic admission | acoustic and stereo rejection | closed |
| linked STN: `LinkedStnNoiseMorph` through construction-bound v6 | compile and construction passed; structural conformance reached `17/18` and later `16/18`; no synthetic render, comparator pack, or listening gate ran | conformance-only | selected once for protocol binding |

The conformance-only continuous-excitation and similarity-aligned cyclic owners
do not qualify: their frozen architectures contain the failed choice, so a
plausible correction would change renderer authority. The compile-only
`CompensatedRenewalSpectral` identity does not qualify separately because its
same compensated DSP lineage later reached acoustic and stereo rejection under
fresh construction and evidence identities.

`LinkedStnNoiseMorph` is the only eligible family. Its canonical brief owns a
complete renderer, exact map, material decomposition, tonal oscillators,
single-claim transient events, residual noise morphing, linked-channel law,
boundaries, bounded state, determinism, and the full gate sequence. Pinned
SiTraNoStar plus the retained STN and noise-morphing papers provide clean-room
source backing. The `S06` plateau-tie and `S18` private-surface misses are
implementation conformance defects against already frozen authority, not
missing architecture choices.

Eligibility is not a quality claim. The renderer is large, its complete
structural surface has never passed together, and no linked-STN acoustic output
has been heard. Residual noise, entry/tail energy, transient replicas, tonal
coherence, and stereo image remain live terminal risks.

Selection was not implementation authority. Batch 31.57 bound the existing
complete renderer and every executable gate to Rule 11 in the same canonical
brief. It changed protocol ownership and candidate identity only. The audit
found no missing authority requiring a DSP, source, seed, metric, threshold,
assertion, comparator, or listening-policy choice. Batch 31.58 started from
fresh source and later exposed the transient-refinement authority gap recorded
below. Batch 31.59's first correction proved incomplete in Batch 31.60. Batch
31.61 owns the replacement authority before retained pre-acoustic execution
may resume.

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
    topology. Complete; `VerifiedSourceRelativeRenewalSpectral` frozen.
29. Implement the verified brief once in its named disposable worktree,
    complete construction `1/1`, freeze one checkpoint, then run `15`
    structural and `9` synthetic owners. Complete; construction and structural
    admission passed, synthetic finished `7/9`, and the candidate was deleted
    before listening.
30. Reassess the paired ratio-end failures against pinned source and prior
    Signal evidence. Complete; candidate seed was unfrozen, no range switch is
    source-backed, and fresh seed-audited authority is frozen.
31. Implement the seed-audited brief once, freeze one checkpoint after
    construction `1/1`, then run structural and synthetic owners in order.
    Complete; construction and structural admission passed, `Y02` failed one
    `8x` chord row, and the candidate was deleted before listening.
32. Reassess the renewal family against the repeated tonal-pitch failure.
    Either select one materially different, source-backed complete renderer or
    close the family. Complete; no eligible replacement found, renewal closed.
33. Apply the operator-authorized listening-led gate correction and freeze one
    fresh complete source-relative authority. Complete; docs and architecture
    only.
34. Implement `ListeningLedSourceRelativeRenewalSpectral` once from fresh
    source. Complete; checkpoint `f76d5bb7` passed construction `1/1` and
    structural `15/15`, then synthetic `Y08` rejected exact-zero impulse hops
    at every ratio. The candidate was deleted before listening.
35. Reconcile the `Y08` impulse measurement range. Complete; the
    complete-output dropout assertion over-broadened the frozen gate and is an
    executable-evidence construction failure.
36. Implement `SupportAuditedListeningLedSourceRelativeRenewalSpectral` once
    from fresh source under the frozen support table. Complete; all objective
    and concealed mono gates passed, then source-relative stereo admission
    rejected local image stability at `16x`.
37. Reassess renewal linked-stereo ownership across the two complete stereo
    failures. Complete; no materially different source-backed complete owner
    remains. Renewal closes without closing the PaulX-like product target.
38. Apply the operator-authorized creative-stereo policy and freeze one fresh
    complete comparator-audited renewal brief. Complete; docs and architecture
    only. No rejected checkpoint is revived.
39. Implement `ComparatorAuditedRenewalSpectral` once from fresh source and run
    its frozen construction, objective, mono, and stereo admission sequence.
    Complete; compile, construction `1/1`, and structural `15/15` passed,
    synthetic admission failed `Y04` and `Y09`, and the candidate was deleted
    before listening.
40. Reconcile the contradictory Batch 31.36 and Batch 31.39 synthetic receipts.
    Complete; exact executable identity was not retained, `Y04` was
    misdescribed in the Batch 31.39 closeout, and `Y09` lacks one canonical
    source-relative swap assertion. Further renewal implementation is closed.
41. Study one materially different, source-backed complete creative owner.
    Complete; `LinkedStnNoiseMorph` is selected for a clean-room whole-renderer
    brief. No candidate implementation is authorized.
42. Freeze one self-contained clean-room `LinkedStnNoiseMorph` renderer and
    evidence brief. Complete; one isolated candidate is ready and no DSP
    entered `main`.
43. Implement that brief once in its named disposable worktree, complete
    construction, freeze one checkpoint, then run structural and synthetic
    admission in order. Complete; the first implementation failed structural
    bounded state and was deleted.
44. Freeze a bounded two-pass schedule, implement it once, and reconcile its
    capacity authority. Complete; bounded v2 failed contradictory construction
    authority and capacity-audited v3 was frozen docs-only.
45. Implement capacity-audited v3 once. Complete; pre-checkpoint geometry
    evaluation found `R_v=59` at `F=8000` against the frozen `R_v<=57` bound.
    The candidate was deleted before compile or gate execution.
46. Reconcile every geometry-derived median extent and affected capacity row.
    Complete; `R_v=59`, shared median scratch remains `97`, and fresh
    geometry-audited v4 authority is frozen docs-only.
47. Implement geometry-audited v4 once, checkpoint after construction, then
    run structural and synthetic admission in order. Complete; construction
    passed `1/1`, structural stopped at `17/18` on `S15` exact silence, and the
    candidate was deleted before synthetic or listening.
48. Reconcile zero-power residual interpolation with bit-exact silence across
    the complete linked-STN authority. Complete; zero-preserving v5 is frozen
    docs-only under fresh identity.
49. Implement zero-preserving v5 once, checkpoint after construction, then run
    structural and synthetic admission in order. Complete; construction passed
    `1/1`, structural stopped at `17/18` on an incorrect `S02` geometry vector,
    and the candidate was deleted before synthetic or listening.
50. Independently audit every exact geometry vector and bind the structural
    table into construction authority. Complete; construction-bound v6 is
    frozen docs-only.
51. Implement construction-bound v6 once, checkpoint after construction, then
    run structural and synthetic admission in order. Complete; construction
    passed `1/1`, structural stopped at `16/18` on peak-plateau ownership and
    private-surface containment, and the candidate was deleted before
    synthetic or listening.
52. Reassess executable structural authority. Complete; most structural owners
    require renderer execution, construction-owning all of them would duplicate
    or move the gate, and `LinkedStnNoiseMorph` is closed without promotion.
53. Reassess creative candidate evidence protocol. Complete; iterative
    conformance now precedes one immutable acoustic checkpoint, exact
    executable identity survives through reassessment, and acoustic failures
    remain terminal.
54. Classify closed creative families under the new protocol and select at
    most one eligible complete owner. Complete; linked STN is the sole
    conformance-only family and is selected once for protocol binding.
55. Bind the complete linked-STN brief to Rule 11 without changing renderer or
    acoustic authority. Complete; fresh conformance and acoustic identity,
    evidence corpus, cleanup, and pass behavior are frozen docs-only.
56. Implement one fresh protocol-bound linked-STN candidate. Ready in the
    exact isolated worktree under the canonical brief.

`Spectral`/`Rough`, coherent overlap, `LayeredCloud`, the upper overlap, dynamic
ratios, and automatic routing still require separate reopening decisions backed
by new complete-system evidence.

## Current State

All isolated spectral candidates and both cyclic candidates are rejected and
deleted. Explicit `Cyclic` and the automatic router remain closed or paused.
Neutral `Dream` remains active product intent. Matching PaulX synthetics
invalidate the old crest calibration. The first complete
`CompensatedRenewalSpectral` implementation failed compile-only validation
before DSP execution and was deleted.
`VarianceCompensatedRenewalSpectral` later produced an invalid synthetic
receipt and was deleted before listening. The fresh
`AuditedVarianceCompensatedRenewalSpectral` checkpoint passed compile,
construction, structural, synthetic, and concealed mono gates without repair
or rerun. It was rejected at linked-stereo image preservation because its
first-sample component orientation inverted source-relative channel balance.
The passed mono renewal core remains historical evidence; the failed stereo
law does not. The first `SourceRelativeRenewalSpectral` checkpoint is
also rejected and deleted after a frozen `mix64` vector typo stopped structural
admission at `14/15`. The verified replacement passed native-left/right
structural admission, then failed two stochastic synthetic owners under a
locally selected seed. That checkpoint remains rejected, but its receipt
cannot support the discarded fixed-resolution range diagnosis because
candidate seed was not frozen. The seed-audited replacement passed construction
and structural admission and cleared the previous replica failure, but failed
`Y02` on the `8x` chord. Two complete checkpoints fail the same tonal-pitch
class. Batch 31.32 found no eligible complete replacement and closed renewal.
Batch 31.33 superseded that future-execution closure under the explicit
operator gate change. Batch 31.34 then passed compile, construction `1/1`, and
structural `15/15`. Synthetic admission finished `8/9`: `Y02` passed its
complete diagnostic, while `Y08` found an exact-zero `H` block in the impulse
row at `4x`, `8x`, and `16x`. Its executable assertion used complete impulse
output for dropout, contrary to the frozen mapped-support boundary. The
checkpoint remains rejected and deleted.

Batch 31.35 classifies that result as executable-evidence construction
failure. The one-sample impulse support maps to `4`, `8`, or `16` output
samples, all shorter than `H=16384`, so it has no eligible dropout window.
Batch 31.25's otherwise matching mono renderer passed the intended `Y08`.
This does not reinterpret or revive Batch 31.34. Fresh
`SupportAuditedListeningLedSourceRelativeRenewalSpectral` authority now owns
one immutable support table, distinct discontinuity and dropout ranges, and
the unchanged renderer and admission system.

Batch 31.36 passed every construction, structural, synthetic, and concealed
mono gate. The operator heard only minor extra low-end noise and opposite
exterior energy weighting versus PaulX. Valid same-source stereo admission
then exposed a different terminal defect: global and band balance stayed
close, but mapped local windows drifted by up to `9.418990 dB` and reversed
channel dominance on the `16x` full mix. The candidate is rejected and
deleted. Alongside Batch 31.25's global balance inversion, this is a second
complete renewal linked-stereo failure and requires architecture reassessment,
not another relation-law adjustment.

Batch 31.37 completed that reassessment. Current-frame common rotation is
already present and insufficient after inverse synthesis and frame blending.
Temporal recurrence, peak trajectories, and paired oscillators are different
families with closed or failed complete-system evidence. Renewal is closed.
No creative renderer was ready under the old stereo policy.

Batch 31.38 records the explicit operator change. Local mapped-window
source-relative balance and dominance are diagnostic for neutral `Dream`.
Whole-render and band balance, structural relationships, exact length,
finiteness, determinism, bounded state, and every other hard gate remain
terminal. Eligible independent stereo listening now owns the final creative
image decision. Batch 31.39's fresh `ComparatorAuditedRenewalSpectral`
checkpoint passed construction and structural admission, then failed
synthetic `Y04` and `Y09` and was deleted before listening. Its `7/9` receipt
conflicts with Batch 31.36's nominally equivalent `9/9` receipt. Candidate
implementation is closed.

Batch 31.40 found the receipts share exact seed, counter, support, and owner
inventory, but not a retained executable identity. The multi-hop briefs do not
freeze helper bodies or a complete `Y09` swap assertion, and cleanup removed
the only candidate source and outputs that could have been compared. Both
receipts remain historical checkpoint decisions. Renewal is closed; the
PaulX-like target remains active without a renderer owner.

Batch 31.41 selects `LinkedStnNoiseMorph` as the next whole-renderer family.
Its source basis is the complete mono STN/noise-morphing path demonstrated by
the pinned SiTraNoStar application plus the published STN decomposition, noise
morphing, transient relocation, envelope, and stereo studies. This is
architecture evidence only. GPL implementation expression, constants,
thresholds, masks, and control flow may not enter Signal.

The complete brief must jointly own:

- one exact monotonic map shared by tonal, transient, and residual lanes
- channel-symmetric two-stage material separation with native-channel
  reconstruction
- persistent linked tonal peak/phase state, including dormancy and
  reactivation
- shared transient classification, segmentation, exact placement, collision,
  seam, and anti-replica behavior
- continuous deterministic residual excitation, spectral-envelope morphing,
  and explicit linked-channel spatial statistics
- mapped source-envelope treatment, component recombination, windowing,
  normalization, exterior continuity, and exact target crop
- bounded duration-independent memory, deterministic offline execution, fixed
  computational shape, terminal gates, listening order, cleanup, and minimal
  admission

No item may be deferred to candidate implementation. Source evidence does not
establish `16x`, long-form musical quality, linked residual stereo, exact
length, or bounded deterministic execution. Those remain terminal Signal
risks. Comparator-relative independent stereo listening remains promotion
authority after objective and concealed mono admission.

No public Rust enum, renderer, harness mode, fixture, artifact schema, runtime
route, or product-facing claim entered `main`. `OfflineHighQuality` remains
byte-exact and Contract `084` remains closed. No creative renderer is admitted.

Batch 31.42 freezes the complete candidate authority in
`offline-creative-linked-stn-noise-morph-brief.md`. It selects one
sample-rate-normalized two-stage separation, exact signed-rational map,
persistent linked tonal owner, one-shot native transient owner, deterministic
covariance-shaped residual owner, mapped envelope, normalized synthesis,
bounded state, exact evidence specification, receipt identity, cleanup, and
minimal admission boundary. No component choice remains open to the isolated
implementation.

The brief deliberately does not reuse renewal's complete-mixture impulse-smear
assumptions. Hard event gates instead require exact mapped anchors, one ledger
commit and emission, bounded crest, comparator-relative maximum spread, and no
secondary active region. Tonal pitch remains a complete finite diagnostic;
concealed long-form listening remains creative authority. Hard linked-stereo
mechanics, whole/band balance, mapped diagnostics, and eligible independent
stereo listening retain this contract's current policy.

Batch 31.42 changed documentation only. The frozen renderer is ready for one
isolated implementation, not production admission. `16x`, component leakage,
long-form musical quality, linked residual image, cost, and exterior character
remain terminal candidate risks.

Batch 31.43 implemented that authority once in isolation. Compile and
construction `1/1` passed. Structural admission completed `17/18`; `S17`
rejected duration-derived component arrays because working state must be
bounded independently of source and output duration. The checkpoint was not
repaired or rerun. Synthetic and listening gates remained closed, and the
candidate worktree, branch, checkpoint reference, source, tests, and build
state were deleted. No creative renderer is admitted.

Batch 31.44 proves a bounded schedule without changing the renderer's audible
owners. Residual orientation requires one deterministic decomposition/event
prepass because its first non-zero augmented-residual signs are non-causal.
The real render then resets all state and advances fixed spectral, component,
event, covariance, envelope, and output rings by monotonic last-consumer
frontiers. Only the required returned `Vec<f32>` may derive capacity from
duration.

The revised canonical brief freezes an `89 MiB` owned-state design ceiling,
retains the terminal `96 MiB` actual allocation ceiling, and makes every
capacity and duration-independence assertion compile-linked `MEMORY_SPEC`
authority. `BoundedLinkedStnNoiseMorph` is a fresh candidate identity. It is
implementation authority only; no creative renderer is admitted.

Batch 31.45 passed compile, then failed construction `0/1` because the frozen
first-residual formula exhaustively reaches `53248` while its asserted maximum
was `59392`. The candidate was deleted before checkpoint. Batch 31.46 retained
the per-geometry formula, corrected that row and its conservative packed model,
and froze fresh capacity-audited v3 authority.

Batch 31.47 stopped that authority before compile or construction execution.
At `F=8000`, the frozen geometry gives `N_s=256` and `A_s=64`; the short
vertical rule evaluates to `round(57.6)=58`, and the frozen upward odd tie
produces `R_v=59`. The same brief requires exhaustive proof that `R_v<=57`.
No allowed assembly repair can make both statements true. No checkpoint or
quality evidence exists, and the disposable candidate surface was deleted.
No creative renderer is admitted.

Batch 31.48 exhaustively recomputed all four median extents with two
independent exact-integer evaluators. The maxima are `Q_h=17`, `Q_v=97`,
`R_h=19`, and `R_v=59`. `Q_v` already owns the shared `97`-scalar median
scratch, so correcting `R_v` changes no ring capacity, packed model, category
ceiling, cost class, or audible rule. The `89 MiB` design sum and `96 MiB`
actual gate remain unchanged. Positive rational half-rounding is frozen to the
larger integer. Fresh `GeometryAuditedBoundedLinkedStnNoiseMorph` authority is
ready for one isolated implementation. No creative renderer is admitted.

Batch 31.49 implemented that authority once from authorized `main` head
`feeb76fe`. Compile and construction `1/1` passed, freezing checkpoint
`e2ef62f8` and tree `85dc0e45`. Structural admission completed `17/18`.
`S15` rejected exact-silence input because the residual lane emitted
deterministic samples around `1e-14` instead of bit-exact zero. All other
structural owners passed.

The dominant cause is contradictory authority. Residual diagonal power uses
`ln(power+eps)` interpolation even when both endpoints are zero, while the
boundary owner requires bit-exact silence. No checkpoint repair or rerun is
permitted. Synthetic and listening stayed closed. The worktree, branch,
checkpoint reference, source, tests, build state, receipt, and outputs were
deleted. No creative renderer is admitted.

Batch 31.50 resolves the contradiction across the complete residual and
boundary path. The sole `zlog` rule returns canonical positive zero only when
both power endpoints are exact zero; every one-zero/one-positive and
two-positive row retains the v4 formula. Zero-power coherence, cross-power,
mono and mid/side excitation, mapped-envelope contribution, and final `f32`
encoding are also canonical positive zero.

The rule is exact, not thresholded. It adds no denoiser, fast path, mask,
duration state, allocation, variable traversal, stochastic change, or
post-render repair. Duplicate, common-negation, anti-phase, swap, signed-zero,
local-envelope, and exact-crop evidence now include zero states. Memory,
determinism, fixed cost, synthetic thresholds, comparator rows, listening
packs, and cleanup remain unchanged.

Fresh `ZeroPreservingGeometryAuditedBoundedLinkedStnNoiseMorph` authority is
ready for one isolated implementation. It does not revive checkpoint
`e2ef62f8` or any deleted source. No creative renderer is admitted.

Batch 31.51 implemented that authority once from `570da160`. Compile and
construction `1/1` passed, freezing checkpoint `95909451` and tree
`080bea36`. Structural admission completed `17/18`. `S02` asserted `Q_h=5`
at `F=8000`; the frozen formula and renderer both produce
`odd(round(0.240*8000/256))=9`.

The dominant cause is incomplete executable geometry authority. Construction
checked exhaustive maxima but not the exact per-rate vector consumed by
structural admission. No checkpoint repair or rerun is permitted. Synthetic
and listening stayed closed. The worktree, branch, checkpoint reference,
source, tests, build state, receipt, and outputs were deleted. No creative
renderer is admitted.

Batch 31.52 independently evaluated all `184001` supported integer-rate
geometry rows twice. Both evaluators agree on table SHA-256
`22d14913f01143007a114fad7a97d44a7e2b07cf5b254b92bc59c7f805e73697`,
FNV-1a-64 `7ffb5aa02900893e`, every transform transition and tie class, maxima
`17,97,19,59`, their first witnesses `16534,8000,17500,8000`, and every
geometry-derived capacity maximum and witness.

Construction-bound v6 freezes one compile-linked `GEOMETRY_SPEC` as the sole
literal geometry table. A separately coded integer oracle must equal the
renderer for every supported rate. One shared authority assertion runs during
construction before checkpoint and again in structural `S02`; `S02` may not
carry another geometry row. The exact renderer, zero-preserving behavior,
memory ceilings, quality gates, and cleanup policy do not change. Fresh
`ConstructionBoundZeroPreservingLinkedStnNoiseMorph` authority is ready for
one isolated implementation. No creative renderer is admitted.

Batch 31.53 implemented that authority once from exact `main` head
`fdad84326d1d2b576f6a73e96499b77be76dcd4e`. A permitted pre-checkpoint Rust
ownership repair followed the first compile attempt. Compile and construction
`1/1` then passed, freezing checkpoint `366ac24b` and tree `68da7e43`.

Structural admission completed `16/18`. `S06` returned peak bins `[1,3,4]`
where the frozen equal-plateau tie law requires `[1,3]`. `S18` found the
forbidden `pub fn` token in the private candidate source. All other structural
owners passed. Synthetic and listening did not open.

The dominant cause is incomplete construction ownership of structural
semantics: construction proved geometry but not peak-plateau tie ownership or
the private-surface token boundary. The checkpoint was not repaired or rerun.
The worktree, branch, checkpoint reference, private source, tests, and build
state were deleted. No creative renderer is admitted and no linked-STN
candidate is ready.

Batch 31.54 audited every structural owner against construction. Geometry,
fixed counter vectors, zero-state primitives, and memory formulas have honest
construction subsets. Request behavior, WOLA, reconstruction, streaming,
tracks, transients, rendered relations, boundaries, allocation, and full
containment need executable structural proof. `S18` should have run before
checkpoint; making `S06` construction-owned requires executing it there.
Generalizing that correction duplicates structural admission or moves the
checkpoint. Neither creates a different renderer.

Six linked-STN implementation attempts produced no synthetic or listening
evidence. The last two failed the same incomplete-executable-authority class.
Contract `084` Rule 7 blocks a locally corrected seventh identity.
`LinkedStnNoiseMorph` closes without promotion. Its brief and receipts remain
historical evidence only. The PaulX-like neutral `Dream` target remains active,
but no creative renderer or implementation batch is ready.

Batch 31.55 freezes the reusable creative candidate protocol in Rule 11.
Compile, construction, and structural checks are iterative conformance against
already frozen authority. They must pass before one immutable acoustic
checkpoint exists. Synthetic, mono, stereo, and promotion stages then run once
from that exact identity and remain terminal.

The policy keeps rejected code off `main` while retaining a bounded local
evidence ref through reassessment, avoiding the lost-helper and incomparable-
receipt failures seen earlier in this roadmap. It does not reopen any renderer.
Conformance-only families become eligible for one explicit docs-only owner
selection; acoustically rejected families remain closed absent new complete
architecture or an evidence-backed product decision.

Batch 31.56 classifies every closed owner across four lineages. Diffusive
spectral and cyclic each reached synthetic rejection; their conformance-only
successors are superseded or contain architecture-level misses. Renewal reached
valid synthetic and concealed-mono admission before stereo rejection, then a
later checkpoint also failed synthetic admission. Those families remain closed.

Linked STN is the sole conformance-only lineage. Its six attempts never ran a
synthetic, comparator, or listening gate. The complete material-separated
architecture remains source-backed and plausibly owns the current pitch,
replica, crest, tonal, transient, residual, stereo, boundary, memory, and
determinism gates. It is selected once for a fresh Rule 11 binding. This does
not revive its historical brief or authorize implementation.

Batch 31.57 completed that binding in the canonical linked-STN brief. Fresh
identity `ConformanceBoundLinkedStnNoiseMorph` owns exact isolated paths, one
tracked conformance ledger, compile plus construction `1/1` plus structural
`18/18` passage twice from one clean tree, and one later immutable acoustic
ref. Synthetic, concealed mono, speaker, and independent stereo gates remain
one-shot terminal stages from that ref.

The audit made retained source bytes, long-form pack hashes, estimator rules,
structural vectors, cleanup, and pass disposition explicit. It corrected old
prose that called retained half-cosine edges linear and described `Y07` with an
adjacent-region denominator; retained artifacts prove half-cosine edges and a
full-active-support denominator. No DSP formula, source bytes, comparator
number, threshold, assertion, or listening policy changed. Batch 31.58 was
therefore opened; no creative renderer was admitted.

Batch 31.58 stopped during pre-acoustic conformance at retained commit
`ae618c90827ddd748dc224632920ee32f785cc65`, tree
`de551fc6fa458d500239ac603ed26dee1a4458d6`. Compile, construction `1/1`,
independent full-buffer `S05`, and bounded-allocation `S17` had passed focused
controls. `S09` then proved a missing authority rule: the reconstructed
isolated impulse's adjacent derivative powers differ by two non-negative
`f64` encodings, so exact comparison chooses source sample `p+1` while frozen
`Y03` requires authored `p`. No formal clean pass, synthetic gate, rendered
audio, listening, or acoustic ref ran. Candidate code did not enter `main`.

Batch 31.59 froze a four-ULP correction from the isolated impulse evidence.
Batch 31.60 applied it and compiled the mapped target ledger, then `S09` failed
on the already-frozen `0.65` impulse-train event. Its reconstructed rise/fall
powers were `0x3fdb0a3d4f5c2900` and `0x3fdb0a3d4f5c290a`, distance `10`.
The retained stop is commit `4cb82a2ef7731aeaf306d3955766c75c9863aa89`,
tree `6083e84604bb95f561fd6b7c25aef55b9a49b12a`. No complete structural round,
synthetic gate, rendered audio, listening, or acoustic ref ran.

Batch 31.61 replaces representation-local ULP counting with one
scale-relative forward-error rule. For non-negative finite current score `a`
and later challenger `b`, `tau=64*f64::EPSILON*max(a,b)`; the challenger wins
only when `b>a` and `b-a>tau`. Otherwise earliest owns the numerical tie.
There is no absolute floor and scores remain unchanged. `S09` owns `64`/`65`
ULP boundary vectors at `1.0` plus both observed impulse pairs. `S10` owns exact
mapped target-ledger anchors. Compiled `Y03` retains authored source anchors
and exact mapped target anchors.

The retained worktree again satisfies the pre-acoustic resume rule. Batch
31.62 may apply the docs closeout, replace only the comparison and direct
owners, then restart full conformance twice. The identity remains
`ConformanceBoundLinkedStnNoiseMorph`. No creative renderer is admitted.

## Next Task

Apply the Batch 31.61 docs closeout commit to retained isolated worktree
`signal-candidate-31-58`. Implement only the frozen scale-relative transient
comparison and direct `S09`, `S10`, and compiled `Y03` ownership. Commit a clean
tree, then restart complete compile, construction `1/1`, and structural `18/18`
conformance twice before creating any acoustic ref. Do not run acoustic gates,
alter `main`, open routing or product exposure, touch Loophole or Chorus, merge,
or push.
