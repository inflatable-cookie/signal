# Offline Creative CyclicGrain Renderer Brief

Status: frozen historical brief; candidate rejected
Owner: dsp
Updated: 2026-07-19
Contract: `085`
Roadmap: `g10.031`, Batch 31.10

## Decision

Build one Signal-owned `CyclicGrain` renderer for explicit fixed-ratio
`Cyclic` expansion above `1x` through `8x`. It is an offline, sample-domain
renderer with:

- one sample-centred source/output map
- one deterministic grain lattice
- at most two overlapping unit-rate source reads
- normalized raised-cosine crossfades
- channel-shared scheduling and native-channel samples
- exact target length and bounded rolling state

Unit-rate reads preserve local pitch. Slow source-anchor motion owns duration.
The source offset between overlapping unit-rate reads creates the commanded
Akai-style cycle under a continuous normalized crossfade. There is no phase
vocoder, transient detector, pitch shifter, random grain cloud, spectral layer,
feedback path, or automatic owner router.

This is a clean-room Signal architecture. Public Potenza source supports only
the high-level feasibility of overlapping moving grains with crossfaded
output. Signal does not copy GPL expression, constants, thresholds, tables,
or control flow. ReaReaRea and Akaizer remain behavioral references only.

## Candidate Outcome

Batch 31.11 implemented this brief once in the disposable
`signal-candidate-31-11` worktree. All seven structural tests passed: request
validation, byte-exact identity, exact length, finiteness, silence,
determinism, monotonic mapping, bounded two-grain scheduling, scheduled-only
impulse energy, linked-stereo covariance, peak bounds, and duration-independent
state.

Creative synthetic admission then stopped on its first neutral row. A `110 Hz`
tone at `2x` measured `111.328 Hz`: `20.778` cents of error against the frozen
`15`-cent ceiling. The stop rule forbade correction or rerun. No later
synthetic row, comparator capture, long-form mono render, `16x` probe, or
stereo listening followed. The worktree, branch, module, tests, and build state
were deleted. No candidate code entered `main`.

This brief remains the exact record of the rejected topology. It is not
authority for a grain-length, hop, window, interpolation, seed, threshold, or
scalar sweep.

## Supported Request

The isolated candidate accepts:

- mono or linked-stereo interleaved finite `f32`
- sample rate `F` from `8,000` through `192,000` Hz
- non-empty input length `L`
- exact target frame count `T`
- derived ratio `q=T/L`, where `q=1` or `1<q<=8`
- finite normalized `motion`, `detail`, and `space` in `[0,1]`
- one explicit `u64` seed

Target frames own duration. The candidate derives `q`; it has no independent
ratio authority. Non-empty `L` and `T` may not exceed `2^53-1`; use checked
integer comparisons `T==L` or `L<T<=8*L` before forming the floating map.
Empty input with `T=0` returns empty output. Every other
empty, zero-target, non-finite, overflowed, unsupported-channel,
unsupported-sample-rate, control, compression, or ratio-above-`8x` request
fails before output allocation. Values are rejected, not clamped.

`q=1` is an exact finite-input passthrough. It does not enter the grain path,
allocate renderer state, or respond to macros or seed. Dynamic ratio, reverse,
pitch composition, cache, artifact routing, RealtimePreview, audio-thread
execution, other creative characters, and public product integration are
unsupported.

The input is borrowed. The required output buffer is not working state. All
other allocations count against the frozen state cap.

## Source Map And Timeline

For output sample-centre coordinate `y`, freeze the sole map:

`a(y)=((y+0.5)*L/T)-0.5`.

It is strictly increasing for every supported non-empty request. Grain anchor
`y_k` reads source anchor `x_k=a(y_k)`. No later stage moves an anchor or owns
another event timeline.

Every read inside a grain is also forward and unit-rate. Cyclic replicas may
interleave across overlapping grains by design, but neither an individual
grain nor the anchor sequence reverses source order.

## Grain Geometry And Semantic Macros

Let `d=detail` and `m=motion`. Freeze a render-wide grain length divisible by
four:

`G=clamp(4*floor((0.160*F*2^(-3*d))/4+0.5),256,32768)`.

This maps `detail=0` toward `160 ms` support and `detail=1` toward `20 ms`,
subject only to the explicit sample bounds. Higher `detail` never increases
`G`: it produces shorter cycles and more source-readable articulation.

Freeze synthesis launch hop:

`H=clamp(floor(G*(0.75-0.25*m)+0.5),G/2,3*G/4)`.

Higher `motion` never increases `H`: it moves from broader, slower crossfades
toward denser overlap and faster cyclic movement. Because `G/2<=H<=3*G/4`,
no output sample has more than two active grains.

The seed selects only the render-wide lattice phase:

`phi=(seed mod H)`.

Grain centres are `y_k=phi+k*H` for every signed integer `k` whose half-open
support `[y_k-G/2,y_k+G/2)` intersects `[0,T)`. Signed indices are never
rebased by source or target duration. Different seeds may alias modulo `H`;
the seed is variation identity, not a claim of unique output for all `u64`
values. There is no per-grain jitter or channel-local random draw.

`space` does not alter grain geometry. Consumers receive only the semantic
macros. Grain length, hop, lattice phase, window, and overlap count remain
private engine details.

## Grain Read And Crossfade

For grain-local `n` from `0` through `G-1`, output index and source position
are:

`y=y_k-G/2+n`

`p=x_k-G/2+n`.

Read each native channel at `p` by linear interpolation between `floor(p)` and
`floor(p)+1`. Samples outside `[0,L)` are exactly zero. All channels use the
same `p` and interpolation fraction.

Freeze the periodic raised-cosine weight:

`v[n]=0.5-0.5*cos(2*pi*n/G)`.

Add `v[n]*sample_c(p)` to the channel numerator and `v[n]` to one shared
normalization accumulator. Once no later grain can touch output index `y`,
emit:

`o_c[y]=numerator_c[y]/normalization[y]`.

Normalization at or below `1e-12` inside `[0,T)` is structural failure. This
is a normalized crossfade, not a gain envelope: with `space=0`, each emitted
sample is a convex combination of linearly interpolated native samples.

There is no transient copy, residual mix, attack layer, tail layer, feedback,
limiter, compressor, loudness correction, or post-render normalization.

## Pitch, Transients, And Replica Ownership

Each grain reads at one source sample per output sample. This is the complete
pitch-compensation law; no separate resampling or pitch-shift stage follows.
Anchor advance `H/q` supplies the requested duration while output grains
advance by `H`.

The renderer has no transient detector or classification state. Every event is
treated by the same scheduled reads. Repetition is permitted only where a
scheduled grain crosses that source event.

For an isolated impulse at integer source sample `e`, define the permitted
output set:

`I_e={y in [0,T): exists active k with |x_k+(y-y_k)-e|<1 and v[y-y_k+G/2]>0}`.

Linear interpolation may distribute one crossing across two adjacent output
samples. Output outside `I_e` must be zero within the frozen numerical
tolerance. A lobe outside this set, a second event read, or a repair echo is an
uncommanded replica and rejects the candidate. This rule distinguishes the
selected cyclic character from arbitrary doubled attacks or feedback.

## Linked Stereo And Space

Scheduling, anchors, positions, interpolation fractions, weights,
normalization, and seed phase are shared across linked channels. Each channel
contributes only its native source samples. There is no dominant-channel
selection, channel sum used as synthesis content, or independent trajectory.

After normalized grain synthesis, linked stereo applies one sample-domain
mid/side law. For `s=space`:

`mid=(left+right)/2`

`side=(left-right)/2`

`side'=side*(1+0.5*s)`

`left'=mid+side'`, `right'=mid-side'`.

Mono ignores `space`. Duplicate stereo remains duplicate for every value.
Neutral `space` is exact identity. Channel swap and common polarity commute
with the operation. The maximum side gain is `1.5`; no channel-local state or
post-width level repair exists.

## Boundaries And Exact Length

Signed pre-roll and post-roll grains are scheduled only when their support
intersects the target crop. Source reads outside `[0,L)` use zero padding.
Output outside `[0,T)` is discarded, never wrapped or reflected.

Use a rolling numerator ring per channel and one rolling normalization ring.
Finalize samples in ascending output order only after the scheduling frontier
proves that no later grain can touch them. Write exactly indices `[0,T)` into
the required output buffer. Never resize-fill, append a synthesis tail, repair
an edge, or add an implicit fade.

A non-finite position, weight, accumulator, normalization value, or output
sample stops the render with failure.

## Memory, Determinism, And Cost

Allocate before grain processing:

- the `G`-sample weight table
- one numerator ring per channel
- one shared normalization ring
- bounded interpolation and scheduler state

Ring capacity may not exceed `G+H`. Checked renderer-owned allocation must stay
at or below `8 MiB` for `C<=2` and `G<=32768`, excluding only borrowed input
and required output. Capacity is identical for matched one-second and one-hour
requests. No grain list, source copy, event list, output-length normalization
array, or duration-growing cache is allowed. No allocation occurs after grain
processing begins.

Each output sample receives at most two grain contributions per channel. Cost
is `O(C*T)` with a fixed upper bound of two linear interpolations, two weighted
adds, and one final division per channel/sample, plus the linked-stereo
operation. There is no FFT or duration-dependent planning pass.

Fixed signed scheduling, explicit rounding, direct seed reduction, fixed
traversal order, and preallocated state require sample-bit-identical output for
the same complete request on the same supported deterministic target.

## Candidate Shape

The disposable candidate adds one private family only:

`crates/signal-dsp-stretch/src/creative_cyclic/`

Use `mod.rs`, `plan.rs`, `schedule.rs`, `grain.rs`, `synthesis.rs`, and
`tests.rs`. They own the private request, checked plan and macro mapping,
signed lattice, source read and weight law, rolling synthesis, and admission
tests. `lib.rs` may declare the module privately in the candidate worktree.

Do not alter production stretch modules, `StretchQuality`,
`StretchBackendTier`, `TimeStretcher`, `OfflineHighQualityStretcher`, cache
identity, artifact plans, runtime reports, binaries, or product routes.
Candidate-only comparator assembly and audio stay under ignored `target/`.
No hidden review API, report mode, fixture family, feature flag, experimental
module tree, or external production dependency is allowed.

## Fixed Admission

Failure stops the sequence.

### 1. Structural Gate

Exercise identity, `2x`, `4x`, and `8x` over empty, one-sample, sub-grain,
exact-grain, silence, impulses at crop and lattice boundaries, tones,
deterministic noise, duplicate stereo, swapped stereo, anti-phase stereo,
delayed stereo, and unequal-level stereo. Also probe ratios immediately above
`1x`, immediately below `8x`, and exactly `16x`. Cover macro values `0`,
`0.5`, and `1` without taking their Cartesian product.

Pass requires:

- byte-exact finite identity passthrough with no grain allocation
- exact `T` frames at every supported ratio without resize fill
- finite positions, state, normalization, and output
- exact-zero output for exact-zero input
- strict monotonicity of `a(y)` and exact formula agreement for every anchor
- exact shared source positions and normalization across linked channels
- complete crop normalization above `1e-12`
- byte-identical repeats for the same complete request
- two seeds with different `phi` change an active non-identity render
- `G` non-increasing with `detail`; `H` and `H/G` non-increasing with `motion`
- no more than two active grains at any crop sample
- state at or below `8 MiB`, with equal capacity for matched short and long
  renders and no processing allocation
- duplicate, swap, and common-polarity covariance within `1e-6`
- neutral `space` relation error within `1e-6`; side width non-decreasing at
  `space` values `0`, `0.5`, and `1`
- at `space=0`, output peak no greater than the largest native input peak plus
  `1e-6`; at `space=1`, no greater than `1.5` times that peak plus `1e-6`
- compression, dynamic ratio, invalid controls, unsupported layouts, and
  `16x` fail before output allocation

### 2. Creative Synthetic Gate

Run isolated tones, chords, harmonic pads, impulses, impulse trains,
amplitude-modulated noise, deterministic broadband noise, silence gaps, and
linked-stereo relations at `2x`, `4x`, and `8x`. Test neutral macros first,
then each macro endpoint independently.

Pass requires:

- dominant tone and chord-partial pitch error at most `15` cents
- zero non-finite findings
- no `5 ms` output window below `-80 dBFS` inside a continuously active tone,
  chord, or noise case whose corresponding source window is above `-40 dBFS`
- RMS-matched exterior-inclusive first-difference peak growth at most `6 dB`
- neutral-space active-support RMS drift from `-9 dB` through `+3 dB`
- RMS-matched crest-factor growth at most `6 dB`
- impulse energy outside the exact scheduled set `I_e` at or below `1e-7`
  absolute amplitude
- every new stable envelope-modulation peak above `-18 dB` on non-periodic
  noise lies at the planned launch cadence `F/H` or one of its harmonics
- higher `detail` never lengthens the measured isolated-event cycle
- higher `motion` never lowers measured grain-launch density
- seed changes lattice alignment without changing target length, dominant
  pitch, or active-support RMS by more than `1 dB`
- every stereo mechanics row passes before listening

The launch cadence and its harmonics are intentional cyclic behavior. Metrics
reject unscheduled replicas, hidden oscillators, dropouts, level collapse, and
semantic inversions; they do not decide whether the cycle is musical.

The first-difference sequence includes the transition from exterior zero to
the first sample and from the last sample back to exterior zero. The silence
case is governed by exact-zero structural admission rather than a decibel
ratio.

### 3. Long-Form Mono Listening

Use the retained percussion, bass, vocal, pad/sustain, and full-mix sources at
`2x`, `4x`, and `8x`. Capture the missing `2x` ReaReaRea references from the
same retained sources with REAPER `7.69` before concealment. Keep the retained
ReaReaRea `4x` and `8x` captures. Assemble everything only under ignored
`target/` using the retained common-RMS / `0.95` peak-ceiling policy.

Compare concealed neutral `Cyclic` (`motion=0.5`, `detail=0.5`, `space=0`)
against ReaReaRea. Review `8x` first, then `4x`, then `2x`. Review `motion`
and `detail` endpoints separately after the neutral decision.

Pass requires:

- recognizable Akai-style cyclic character on at least `12` of `15` rows
- musically useful output on at least `10` of `15` rows
- candidate preferred or tied with ReaReaRea on at least `10` of `15` rows,
  including at least `7` of the `10` primary `4x` and `8x` rows
- no unusable row and no source family losing all three ratios
- no uncommanded click, dropout, arbitrary level step, doubled cycle, or
  source-obscuring buzz at the neutral point
- independently recognizable, useful directions for both `motion` and
  `detail`; neither may collapse into a gain-only change

Listening is promotion authority. Objective wins cannot compensate for a
failed character or unusable row.

### 4. Sixteen-Times Rejection Probe

Submit the five retained source requests at exactly `16x` before any candidate
audio allocation or render. Every request must return the same typed
unsupported-ratio result with no partial output or retained state. Existing
ReaReaRea `16x` audio remains context only; it is not a Signal target and does
not authorize widening the request range.

### 5. Linked-Stereo Admission

Only after mono and the rejection probe pass, run the retained stereo
mechanics plus long-form material covering stable centre, wide sustain,
transient stereo, delayed channels, unequal levels, and anti-phase content at
`2x`, `4x`, and `8x`. Review neutral and maximum `space` under concealed,
level-matched conditions.

An eligible listener independent of the operator must assess centre stability,
width pumping, side inversion, one-sided texture, channel echo, and whether
`space` widens without detaching the image. Pass requires no relation fault or
unusable row, neutral image stability on every row, and a useful wider image
at maximum `space` on at least two thirds of applicable rows. The operator's
one-ear hearing does not satisfy this gate. Missing review blocks promotion;
it never waives stereo.

### 6. Minimal Admission

Only a candidate passing every prior gate may admit:

- the single internal `creative_cyclic` family
- its private fixed-ratio request and renderer
- structural and synthetic tests required to preserve admitted behavior
- one frozen cyclic engine-version identifier for later cache work

Do not admit a public character enum, product API, cache schema, artifact
surface, runtime route, automatic router, pitch path, dynamic ratio,
`Dream`, `Spectral`, `Rough`, or `Cloud`. Product-facing work remains a later
Contract `085` batch.

## Rejection And Cleanup

Any structural, synthetic, mono-listening, `16x`, or stereo-listening miss
rejects the whole candidate. Record one dominant cause and stopped gate.
Delete the disposable worktree and branch, private module, tests, build state,
local comparator assembly, and generated candidate audio. No rejected
constant, profile, fixture, report mode, hidden API, or partial mechanism moves
to `main`.

One failure returns `g10.031` to cyclic-owner reassessment. It does not
authorize grain-length, hop, window, interpolation, seed, stereo, threshold,
or scalar sweeps. A materially different complete architecture requires a new
brief before another implementation.

## Sources

- [Potenza Akai-style time-stretch source](https://github.com/dar-io-p/potenza-time-stretch/tree/ddb44a8f949b3f49320932e1d2e997b3a02149bb)
- [REAPER](https://www.reaper.fm/)
- [Akaizer](https://the-akaizer-project.blogspot.com/)

## Next Task

No implementation follows this brief. The materially different
`SimilarityAlignedCyclic` replacement also failed and final ownership
reassessment closed explicit `Cyclic`. Do not tune or reimplement either
candidate.
