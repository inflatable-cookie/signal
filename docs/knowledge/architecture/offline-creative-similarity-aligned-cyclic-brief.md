# Offline Creative SimilarityAlignedCyclic Renderer Brief

Status: rejected at structural admission; candidate deleted
Owner: dsp
Updated: 2026-07-19
Contract: `085`
Roadmap: `g10.031`, Batches 31.13-31.14

## Decision

Build one Signal-owned `SimilarityAlignedCyclic` renderer for explicit
fixed-ratio `Cyclic` expansion above `1x` through `8x`. It is an offline,
sample-domain renderer with:

- one exact ideal source/output map
- one regular output-segment lattice
- one bounded waveform-similarity search per segment
- one strictly increasing integer source-anchor path
- forward unit-rate native-channel reads
- complementary overlap crossfades and exact rolling output

The search aligns every new segment head to the natural continuation of the
previous selected segment. This changes the rejected `CyclicGrain` join owner:
fixed source offsets no longer crossfade without regard to waveform agreement.
The bounded deviation still creates the commanded Akai-style cycle while the
alignment search owns pitch-period and join continuity.

There is no phase vocoder, pitch detector, pitch shifter, transient detector,
random grain cloud, spectral layer, feedback path, automatic router, or
external production dependency.

The original WSOLA work and maintained SoundTouch architecture support the
high-level feasibility of similarity-selected waveform segments. Signal does
not copy source expression, constants, thresholds, search order, tables,
vectorization, or tuning. ReaReaRea remains a behavioral reference only.

## Supported Request

The isolated candidate accepts:

- mono or linked-stereo interleaved finite `f32`
- sample rate `F` from `8,000` through `192,000` Hz
- non-empty input frame count `L`
- exact target frame count `T`
- derived ratio `q=T/L`, where `q=1` or `1<q<=8`
- finite normalized `motion`, `detail`, and `space` in `[0,1]`
- one explicit `u64` seed

Target frames own duration. Use checked integer comparisons `T==L` or
`L<T<=8*L` before forming any floating ratio. Non-empty `L` and `T` may not
exceed `2^53-1`. Empty input with `T=0` returns empty output. Every other
empty, zero-target, overflowed, non-finite, unsupported-channel,
unsupported-sample-rate, control, compression, or ratio-above-`8x` request
fails before output allocation. Values are rejected, not clamped.

`q=1` is an exact finite-input passthrough. It does not allocate renderer
state, run alignment, respond to macros, or respond to seed. Dynamic ratio,
reverse, pitch composition, cache, artifact routing, RealtimePreview,
audio-thread execution, other creative characters, and public product
integration are unsupported.

The input is borrowed. The required output buffer is not working state. Every
other allocation counts against the frozen state cap.

## Ideal Map And Output Lattice

For output sample-centre coordinate `y`, the sole ideal map is:

`a(y)=((y+0.5)*L/T)-0.5`.

Let `d=detail` and `m=motion`. Freeze the overlap length, rounded to a multiple
of eight:

`O=clamp(8*floor((0.064*F*2^(-d))/8+0.5),256,12288)`.

`detail=0` gives approximately `64 ms` overlap support. `detail=1` gives
approximately `32 ms`. Higher `detail` never increases `O`.

Freeze the output launch hop:

`H=floor(O*(2.5-m)+0.5)`.

Thus `1.5*O<=H<=2.5*O`. Higher `motion` never increases `H`: it creates denser
alignment decisions and faster cyclic movement. Segment length is:

`N=H+O`.

The seed selects only the render-wide output-lattice phase:

`phi=seed mod H`.

Segment starts are `y_k=phi+k*H` for signed integer `k`. Schedule every segment
whose half-open support `[y_k,y_k+N)` intersects `[0,T)`, in increasing `k`.
Signed indices are never rebased by source length, target length, or crop.
Different seeds may alias modulo `H`; seed is variation identity, not a claim
of unique output for every `u64`.

`space` does not alter mapping, geometry, search, or normalization. Consumers
receive semantic macros only. Segment, overlap, hop, search, confidence, and
lattice controls remain private.

## Nominal And Realized Source Path

The integer source anchor nearest the ideal map at launch `y_k` is evaluated
without floating-point mapping:

`r_k=floor(((2*y_k+1)*L)/(2*T))`.

Use signed `i128` numerator arithmetic and mathematical floor division. This
is exactly `floor(a(y_k)+0.5)`. Source indices outside `[0,L)` are legal
zero-padded reads.

The first scheduled segment uses `x_k=r_k`. Every later selected anchor must
satisfy:

- `x_k>x_(k-1)`
- `|x_k-r_k|<=R`
- `x_k` is selected only by the frozen search below

Freeze search radius and coarse stride:

`R=clamp(floor(0.012*F+0.5),96,2304)`

`J=clamp(floor(F/4000+0.5),2,48)`.

`R` is approximately `12 ms`. `J` is approximately `0.25 ms`. Neither varies
with ratio or semantic macros.

For every later segment, define the legal integer interval:

`lo=max(r_k-R,x_(k-1)+1)`

`hi=r_k+R`.

The supported geometry guarantees `lo<=hi`; a violation is structural failure.
Define the nominal legal fallback:

`n_k=clamp(r_k,lo,hi)`.

Every correction is therefore bounded around the current ideal anchor. It is
never integrated into the next nominal anchor. The selected `x_k` sequence is
the single realized source-anchor path. Every segment read derives from it;
there is no adaptive free-running cursor or second event timeline.

## Similarity Score

For candidate anchor `x`, compare the previous segment's natural continuation
with the new segment head over the exact synthesis overlap:

`u_c[n]=P_c(x_(k-1)+H+n)`

`v_c[n]=P_c(x+n)`

for channel `c` and `0<=n<O`. `P_c(i)` is native input channel `c` at integer
frame `i`, or exact zero outside `[0,L)`.

For an index set `S`, evaluate one zero-mean normalized correlation per channel
in `f64`, using ascending index order:

`cov_c=sum(u*v)-sum(u)*sum(v)/|S|`

`var_u_c=max(sum(u*u)-sum(u)^2/|S|,0)`

`var_v_c=max(sum(v*v)-sum(v)^2/|S|,0)`.

If either variance is zero, `rho_c=0`. Otherwise:

`rho_c=clamp(cov_c/sqrt(var_u_c*var_v_c),-1,1)`.

The linked score is:

`score(x)=(1/C)*sum_c(rho_c)`.

The fixed channel average makes the decision invariant to channel order,
duplicate layout, whole-source polarity, and exact anti-phase relationships.
No channel sum becomes synthesis content. A silent channel contributes zero;
it does not create an independent fallback or trajectory.

The coarse index set is `0,J,2J,...<O`, plus `O-1` when it is not already
present. The full index set is every integer `0..O-1`. Any non-finite
accumulator or score stops the render with failure.

## Deterministic Bounded Search

For each segment after the first:

1. Build the unique coarse candidate set in ascending order:
   - `lo`, `hi`, and `n_k`
   - every `r_k+j*J` inside `[lo,hi]` for signed integer `j`
2. Score that set with the coarse index set.
3. Rank by:
   - higher score
   - smaller `|x-r_k|`
   - smaller source anchor `x`
4. Retain the best three distinct coarse anchors, or all anchors if fewer than
   three exist.
5. Build the unique full candidate set in ascending order from:
   - `n_k`
   - every integer anchor within `J` samples of each retained coarse anchor,
     intersected with `[lo,hi]`
6. Score that set with every overlap sample and apply the same ranking.
7. If the best full score is below `0.25`, select `x_k=n_k`; otherwise select
   the best full anchor.

Exact silence, constant regions, and zero-variance comparisons score zero and
therefore follow the nominal legal path. Low-confidence noise does not acquire
arbitrary anchor motion. There is no quick mode, randomized candidate order,
history-weighted score, transient exception, pitch epoch, or candidate-time
confidence choice.

The coarse set may contain at most `128` anchors. The full set may contain at
most `3*(2*48+1)+1=292` anchors. Candidate storage is fixed-capacity and
allocated before segment processing.

## Segment Read And Synthesis

Every selected segment reads native input at unit rate:

`p=x_k+n`, for `0<=n<N`.

There is no interpolation. Every channel uses the same integer `p`, anchor,
window, and normalization. Reads outside `[0,L)` are exact zero.

Freeze the overlap ramp for `0<=n<O`:

`f[n]=0.5-0.5*cos(pi*(n+0.5)/O)`.

Freeze the segment window for `0<=n<N`:

- `w[n]=f[n]` when `0<=n<O`
- `w[n]=1` when `O<=n<H`
- `w[n]=1-f[n-H]` when `H<=n<H+O`

The previous segment tail and current segment head sum to unity at every
interior overlap sample. Because `H>=1.5*O`, at most two segments touch any
output frame.

For output index `y=y_k+n`, add `w[n]*P_c(x_k+n)` to one `f64` numerator per
channel and `w[n]` to one shared `f64` normalization accumulator. Emit:

`o_c[y]=numerator_c[y]/normalization[y]`.

Normalization at or below `1e-12` inside `[0,T)` is structural failure. This
is a normalized crossfade, not a gain envelope. At neutral `space`, every
output sample is a convex combination of native input samples.

There is no transient copy, residual mix, attack layer, tail layer, limiter,
compressor, loudness correction, DC repair, or post-render normalization.

## Pitch, Events, And Replica Ownership

Unit-rate reads are the complete pitch-preservation law. Similarity-selected
anchors align the waveforms that actually meet in the crossfade. No resampler
or pitch stage follows.

The renderer has no transient detector or classification state. Every event is
owned by the same ideal map, selected source path, and scheduled segment reads.
Alignment may move a segment by at most `R`; it may not hold local time at
unity and compensate elsewhere.

For an isolated impulse at integer source frame `e`, define the permitted
output set:

`I_e={y in [0,T): exists k with n=y-y_k, 0<=n<N, x_k+n=e, w[n]>0}`.

Output outside `I_e` must be exact zero within `1e-7` absolute amplitude.
Multiple members of `I_e` are commanded cyclic reads. Any lobe outside this
set, feedback echo, detector copy, or boundary repair is an uncommanded replica
and rejects the candidate.

## Linked Stereo And Space

The ideal map, lattice, search candidates, similarity scores, confidence
decision, selected anchors, source positions, windows, and normalization are
shared across linked channels. Each channel contributes only native samples.
There is no dominant-channel choice, independent left/right search, or channel
sum used as output.

After normalized synthesis, linked stereo applies one bounded mid/side law.
For `s=space`:

`mid=(left+right)/2`

`side=(left-right)/2`

`side'=side*(1+0.5*s)`

`left'=mid+side'`, `right'=mid-side'`.

Mono ignores `space`. Neutral `space` is identity. Duplicate stereo remains
duplicate. Channel swap and common polarity commute with analysis and
synthesis. Maximum side gain is `1.5`; no channel-local state or level repair
follows.

## Boundaries And Exact Length

Schedule signed pre-roll and post-roll segments only when their support
intersects `[0,T)`. Source reads outside `[0,L)` use zero padding. Output
outside `[0,T)` is discarded, never wrapped or reflected.

Use a rolling numerator ring per channel and one shared normalization ring.
Finalize frames in ascending output order only after the next segment start
proves that no current or future segment can touch them. After the final
scheduled segment, finalize the remaining target crop. Write exactly indices
`[0,T)` into the required output buffer.

Never resize-fill, append a synthesis tail, repair an edge, or add an implicit
fade. A non-finite map value, anchor, score, weight, accumulator,
normalization, width result, or output sample stops the render with failure.

## Memory, Determinism, And Cost

Derived maxima are:

- `O<=12288`
- `H<=30720`
- `N<=43008`
- rolling ring capacity at most `N+H<=73728`
- coarse candidate capacity `128`
- full candidate capacity `292`

Allocate before segment processing:

- one `N`-sample `f64` window table
- one `f64` numerator ring per channel
- one shared `f64` normalization ring
- bounded score accumulators and fixed candidate arrays
- bounded scheduler and linked-width state

Checked renderer-owned allocation must stay at or below `4 MiB` for `C<=2`,
excluding only borrowed input and required output. Capacity is identical for
matched one-second and one-hour requests. No source copy, segment list, anchor
list, event list, output-length normalization array, or duration-growing cache
is allowed. No allocation occurs after segment processing begins.

Synthesis cost is `O(C*T)` with at most two contributions per channel and
output frame. Alignment adds, per launch, at most `128` coarse scores over at
most `ceil(O/J)+1` samples per channel and `292` full scores over `O` samples
per channel. Total cost is bounded by:

`O((T/H)*C*(128*ceil(O/J)+292*O)+C*T)`.

There is no FFT, duration-dependent planning pass, parallel score reduction,
or input-dependent allocation. Offline execution may be expensive at high
sample rates; its work remains explicit and bounded.

Exact integer mapping, signed scheduling, fixed traversal, `f64` accumulation,
explicit tie rules, direct seed reduction, and preallocated state require
sample-bit-identical output for the same complete request on the same supported
deterministic target.

## Candidate Shape

The disposable candidate adds one private family only:

`crates/signal-dsp-stretch/src/creative_similarity_cyclic/`

Use `mod.rs`, `plan.rs`, `schedule.rs`, `alignment.rs`, `synthesis.rs`, and
`tests.rs`. They own the private request, checked geometry, exact rational map,
signed lattice, bounded score and search, selected anchor path, rolling
synthesis, and admission tests. `lib.rs` may declare the module privately in
the candidate worktree.

Do not alter production stretch modules, `StretchQuality`,
`StretchBackendTier`, `TimeStretcher`, `OfflineHighQualityStretcher`, cache
identity, artifact plans, runtime reports, binaries, or product routes.
Candidate-only comparator assembly and generated audio stay under ignored
`target/`. No hidden review API, report mode, fixture family, feature flag,
experimental module tree, or external production dependency is allowed.

## Fixed Admission

Failure stops the sequence.

### 1. Structural Gate

Exercise identity, `2x`, `4x`, and `8x` over empty, one-sample, sub-segment,
exact-segment, silence, constants, crop and lattice impulses, tones,
deterministic noise, duplicate stereo, swapped stereo, common-polarity stereo,
anti-phase stereo, delayed stereo, and unequal-level stereo. Probe ratios
immediately above `1x`, immediately below `8x`, and exactly `16x`. Cover macro
values `0`, `0.5`, and `1` without their Cartesian product.

Pass requires:

- byte-exact finite identity passthrough with no renderer allocation
- exact `T` frames at every supported ratio without resize fill
- finite mapping, score, window, state, normalization, width, and output
- exact-zero output for exact-zero input
- exact rational agreement for every `r_k`
- every selected anchor strictly increasing and within `R` of `r_k`
- exact agreement with the two-stage search and frozen tie order
- zero-variance and sub-`0.25` rows selecting `n_k`
- one constructed unique-offset row selecting its known correlated anchor
- duplicate, swap, common-polarity, and exact anti-phase score covariance
- exact shared anchors, positions, weights, and normalization across channels
- `f[n]+(1-f[n])=1` within `1e-12` and crop normalization above `1e-12`
- no more than two active segments at any target frame
- byte-identical repeats for the same complete request
- two seeds with different `phi` changing an active non-identity render
- `O` non-increasing with `detail`; `H` and `H/O` non-increasing with `motion`
- state at or below `4 MiB`, equal capacity for matched short and long renders,
  fixed candidate capacities, and no processing allocation
- mono output matching either channel of duplicate stereo within `1e-6`
- duplicate, swap, and common-polarity output covariance within `1e-6`
- neutral `space` relation error within `1e-6`; side width non-decreasing at
  `space` values `0`, `0.5`, and `1`
- at `space=0`, output peak no greater than the largest native input peak plus
  `1e-6`; at `space=1`, no greater than `1.5` times that peak plus `1e-6`
- compression, dynamic ratio, invalid controls, unsupported layouts, and
  `16x` failing before output allocation

The known-offset control uses one fixed zero-mean deterministic sequence as
the reference continuation and places one exact copy at a legal offset. All
other candidate support is zero. The expected anchor is fixed before the
candidate runs.

### 2. Creative Synthetic Gate

Run the retained neutral `110 Hz`, `2x` pitch row first with `motion=0.5`,
`detail=0.5`, `space=0`, and `seed=0`. It must measure at most `15` cents error
under the unchanged measurement law. Failure stops the candidate before any
other synthetic row.

After it passes, run isolated tones, chords, harmonic pads, impulses, impulse
trains, amplitude-modulated noise, deterministic broadband noise, silence
gaps, and linked-stereo relations at `2x`, `4x`, and `8x`. Test neutral macros
first, then each macro endpoint independently.

Pass requires:

- dominant tone and chord-partial pitch error at most `15` cents
- zero non-finite findings
- no `5 ms` output window below `-80 dBFS` inside a continuously active tone,
  chord, or noise case whose corresponding source window is above `-40 dBFS`
- RMS-matched exterior-inclusive first-difference peak growth at most `6 dB`
- neutral-space active-support RMS drift from `-9 dB` through `+3 dB`
- RMS-matched crest-factor growth at most `6 dB`
- impulse amplitude outside the exact scheduled set `I_e` at or below `1e-7`
- no selected anchor outside its ideal neighbourhood or out of source order
- every new stable envelope-modulation peak above `-18 dB` on non-periodic
  noise lying at planned launch cadence `F/H` or one of its harmonics
- higher `detail` never lengthening measured isolated-event cycle
- higher `motion` never lowering measured launch density
- seed changing lattice alignment without changing target length, dominant
  pitch, or active-support RMS by more than `1 dB`
- every linked-stereo mechanics row passing before listening

Planned launch cadence and its harmonics are intentional cyclic behavior.
Metrics reject unscheduled replicas, pitch displacement, hidden oscillators,
dropouts, level collapse, arbitrary anchor drift, and semantic inversions.
They do not decide whether the cycle is musical.

The first-difference sequence includes exterior zero to first sample and last
sample back to exterior zero. Exact silence uses the structural zero rule, not
a decibel ratio.

### 3. Long-Form Mono Listening

Use the retained percussion, bass, vocal, pad/sustain, and full-mix sources at
`2x`, `4x`, and `8x`. Only after synthetic admission, capture the missing `2x`
ReaReaRea references from the same sources with REAPER `7.69`. Keep the
retained `4x` and `8x` captures. Assemble everything only under ignored
`target/` using the retained common-RMS / `0.95` peak-ceiling policy.

Compare concealed neutral `Cyclic` (`motion=0.5`, `detail=0.5`, `space=0`,
`seed=0`) against ReaReaRea. Review `8x`, then `4x`, then `2x`. Review
`motion` and `detail` endpoints separately only after the neutral decision.

Pass requires:

- recognizable Akai-style cyclic character on at least `12` of `15` rows
- musically useful output on at least `10` of `15` rows
- candidate preferred or tied with ReaReaRea on at least `10` of `15` rows,
  including at least `7` of the `10` primary `4x` and `8x` rows
- no unusable row and no source family losing all three ratios
- no uncommanded click, dropout, arbitrary level step, doubled cycle,
  source-obscuring buzz, or unstable event drift at neutral
- independently recognizable, useful directions for `motion` and `detail`;
  neither may collapse into gain-only change

Listening is promotion authority. Objective wins cannot compensate for a
failed character or unusable row.

### 4. Sixteen-Times Rejection Probe

Submit the five retained source requests at exactly `16x`. Every request must
return the same typed unsupported-ratio result before any `16x` output
allocation or render, with no partial output or retained state. Existing
ReaReaRea `16x` audio is context only; it does not widen Signal support.

### 5. Linked-Stereo Admission

Only after mono and the rejection probe pass, run the retained stereo mechanics
plus long-form material covering stable centre, wide sustain, transient
stereo, delayed channels, unequal levels, and anti-phase content at `2x`,
`4x`, and `8x`. Review neutral and maximum `space` under concealed,
level-matched conditions.

An eligible listener independent of the operator must assess centre stability,
width pumping, side inversion, one-sided alignment, channel echo, and whether
`space` widens without detaching the image. Pass requires no relation fault or
unusable row, neutral image stability on every row, and useful maximum-space
widening on at least two thirds of applicable rows. The operator's one-ear
hearing does not satisfy this gate. Missing review blocks promotion; it never
waives stereo.

### 6. Minimal Admission

Only a candidate passing every prior gate may admit:

- the single internal `creative_similarity_cyclic` family
- its private fixed-ratio request and renderer
- structural and synthetic tests required to preserve admitted behavior
- one frozen cyclic engine-version identifier for later cache work

Do not admit a public character enum, product API, cache schema, artifact
surface, runtime route, automatic router, pitch path, dynamic ratio, `Dream`,
`Spectral`, `Rough`, or `Cloud`. Product-facing work remains a later Contract
`085` batch.

## Rejection And Cleanup

Any structural, synthetic, mono-listening, `16x`, or stereo-listening miss
rejects the whole candidate. Record one dominant cause and stopped gate.
Delete the disposable worktree and branch, private module, tests, build state,
local comparator assembly, and generated candidate audio. No constant,
profile, fixture, report mode, hidden API, score helper, search mechanism, or
partial renderer moves to `main`.

No correction, rerun, geometry change, search-radius change, confidence
change, score variant, candidate-count change, window change, macro remap,
stereo repair, threshold sweep, or test-tone substitution follows a miss.

If pitch displacement or waveform-join error is again the dominant cause, two
complete cyclic architectures have failed for the same reason and explicit
`Cyclic` closes. A different dominant cause returns only to docs-level owner
reassessment; it does not authorize a third implementation directly.

## Candidate Decision

Batch 31.14 implemented this brief once in the isolated
`signal-candidate-31-14` worktree. Compile-only validation passed. Structural
admission then failed its frozen known-offset recovery case: with an exact
natural-continuation anchor at source frame `6352`, the two-stage search chose
frame `6432`.

The coarse shortlist can exclude an exact match that lies between coarse
samples when neighbouring noise correlations do not rank in the top three.
The full search then cannot recover that anchor. This is a structural search-
reachability failure, not a pitch or listening result. The gate was not
corrected or rerun. The worktree, branch, private module, tests, and build state
were deleted. No candidate surface entered `main`.

Batch 31.15 found no third materially different, source-backed cyclic topology.
SOLA/WSOLA variants repair this search owner; pitch-/epoch-synchronous methods
lack one full-mix linked period owner and retained `8x` evidence; other hybrids
reopen closed seams or separate programs. Explicit `Cyclic` is closed without
promotion.

## Sources

- [Verhelst and Roelands, WSOLA](https://doi.org/10.21437/Eurospeech.1993-59)
- [SoundTouch algorithm notes](https://soundtouch.surina.net/README.html)
- [SoundTouch source, studied revision `f738b113`](https://codeberg.org/soundtouch/soundtouch/commit/f738b1132ec1fd56efc90367898244cf52d9e6a5)
- [REAPER](https://www.reaper.fm/)

## Next Task

No implementation follows this brief. Explicit `Cyclic` is closed. Reopen only
from new complete-system owner evidence or an explicit operator decision for a
separate creative research program.
