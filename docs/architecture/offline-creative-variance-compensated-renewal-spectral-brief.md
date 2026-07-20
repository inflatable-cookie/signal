# Offline Creative VarianceCompensatedRenewalSpectral Renderer Brief

Status: rejected at evidence-integrity audit; superseded by audited brief
Owner: dsp
Updated: 2026-07-20
Contract: `085`
Roadmap: `g10.031`, Batch 31.22

## Decision

Build one Signal-owned `VarianceCompensatedRenewalSpectral` candidate for neutral
`Dream` at `4x`, `8x`, and `16x`.

It is one complete magnitude-resynthesis renderer:

- one exact sample-centred source/output map
- one sample-rate-normalized long transform
- input phase discarded after analysis
- deterministic stochastic phase renewal per output frame
- adjacent-frame raised-cosine blending
- Signal-derived overlap-statistics compensation
- one linked mid/side law
- deterministic exterior support and exact target crop

No instantaneous-frequency carrier, phase propagation, magnitude recurrence,
transient detector, onset reset, component layer, limiter, compressor, or
post-render gain repair is present.

This is a clean-room architecture. Pinned PaulXStretch source and output are
reference evidence only. Upstream expression, constants, thresholds, random
generator, window coefficients, tables, and control flow do not transfer.

## Supported Request

The private candidate accepts:

- borrowed mono or linked-stereo interleaved finite `f32`
- integer sample rate `F` from `8,000` through `192,000` Hz
- non-empty source frame count `L`
- exact target frame count `T`
- checked ratio range `4*L <= T <= 16*L`
- one explicit `u64` seed
- finite normalized `space` in `[0,1]`

`L` and `T` may not exceed `2^53-1`. Empty input with `T=0` returns empty.
Every other empty, zero-target, overflowed, non-finite, unsupported-channel,
unsupported-rate, compression, or range-miss request fails before output
allocation. Values are rejected, not clamped.

The candidate owns neutral `Dream` only. `motion`, `detail`, pitch composition,
dynamic ratio, reverse, other creative characters, cache, artifact routing,
RealtimePreview, audio-thread execution, and public product exposure remain
unsupported.

Input and required output storage do not count as working state. Every other
allocation does.

## Transform And Map

Use one transform length:

`N=clamp(nearest_power_of_two(round(2*F/3)),8192,131072)`

Power-of-two ties select the larger value. Output block length is `H=N/2`.
At `44.1` and `48` kHz, `N=32768` and `H=16384`.

Use one periodic Hann analysis window:

`w[n]=0.5-0.5*cos(2*pi*n/N)`, `0<=n<N`.

Its energy calibration is:

`G=sqrt(N/sum_n(w[n]^2))`.

Output block `j` begins at `y_j=j*H`. Analysis frame `j` owns source centre:

`x_j=((y_j+0.5)*L/T)-0.5`.

Evaluate `(2*y_j+1)*L` over denominator `2*T` with checked `u128`, convert
once to `f64`, then subtract `0.5`. This is the sole source/output map.

For analysis index `n`, read `p=x_j+n-(N-1)/2`. Let `i=floor(p)` and
`u=p-i`. Use four-point cubic Lagrange interpolation over `i-1` through `i+2`:

- `c_-1=-u*(u-1)*(u-2)/6`
- `c_0=(u+1)*(u-1)*(u-2)/2`
- `c_1=-(u+1)*u*(u-2)/2`
- `c_2=(u+1)*u*(u-1)/6`

Samples outside `[0,L)` are exact zero. Every linked component reads the same
`p`. No stage shifts the cursor, repeats an event, or owns another timeline.

## Analysis And Phase Renewal

Mono has one native component. Stereo uses orthonormal components:

- `M=(L+R)/sqrt(2)`
- `D=(L-R)/sqrt(2)`

Interpolate, window, and transform each component. Retain non-negative-bin
magnitudes `A_r[j,b]`. Discard input coefficient phase.

Before frame processing, scan each component from source start. Canonicalize
signed zero. Orientation `o_r` is the sign of the first exactly non-zero
sample, or `+1` for exact silence. Retain the exact-silence flag.

Use the Signal-owned counter generator from the rejected brief, not an upstream
random stream. Define `mix64(z)` with wrapping `u64` arithmetic:

1. `z=(z xor (z>>30))*0xBF58476D1CE4E5B9`
2. `z=(z xor (z>>27))*0x94D049BB133111EB`
3. return `z xor (z>>31)`

For frame `j`, bin `b`, and stream tag `s`:

`z=mix64(seed xor mix64(j xor FRAME) xor rotate_left(mix64(b xor BIN),21) xor s)`.

The little-endian ASCII tags are `RNWFRAME`, `RNWBIN00`, `RNWBASE0`, and
`RNWSIDE0`. Convert the high 53 bits to `u=(z>>11)/2^53`, then
`phase(z)=2*pi*u-pi`.

Mono and mid use one base phase `theta[j,b]`:

`Y_r[j,b]=o_r*A_r[j,b]*exp(i*theta[j,b])`.

For side, let `f_b=b*F/N`, `t=clamp((f_b-250)/(1500-250),0,1)`, and
`h=t*t*(3-2*t)`. With side-stream phase `zeta[j,b]`:

`Y_D[j,b]=o_D*A_D[j,b]*exp(i*(theta[j,b]+space*h*zeta[j,b]))`.

DC is real signed magnitude. Nyquist is exact zero. Negative bins are conjugate
mirrors. Exact-silent components remain exact zero. There is no phase,
dormancy, tonal-peak, or reactivation state.

## Frame Blend And Compensation

Inverse-transform each component with scale `1/N`. Let the real frame be
`z_r[j,n]`. Apply no synthesis window.

For block offset `0<=n<H`, define:

- `u=(n+0.5)/H`
- `a=0.5+0.5*cos(pi*u)`
- `b=1-a`
- `c=1/sqrt(a*a+b*b)`

Synthesize:

`q_r[j,n]=G*c*(a*z_r[j,H+n]+b*z_r[j+1,n])`.

Decode stereo after blending:

- `L=(q_M+q_D)/sqrt(2)`
- `R=(q_M-q_D)/sqrt(2)`

The compensation is derived from Signal's overlap statistics. For adjacent
zero-mean equal-variance uncorrelated frames, blend variance is
`sigma^2*(a^2+b^2)`. Multiplying by `c` makes that variance `sigma^2` at every
block position. `1<=c<=sqrt(2)`. `G` separately owns analysis-window energy.

This law removes the deterministic blend-position energy dip. It does not
claim a waveform crest bound: adjacent frames and bins can still align.
Reference-relative synthetic controls own that risk.

Cache exactly two adjacent synthesized frames per component. Frame `j+1`
becomes frame `j` for the next block. No mutable random state, frame-local
gain, overlap denominator, peak follower, limiter, compressor, or output
normalization exists.

## Boundaries And Creative Transients

Exterior envelope length is `E=min(H,floor(T/4))`. If `E>=2`, multiply output
sample `y` by both applicable factors:

- head: `sin((pi/2)*y/(E-1))` for `y<E`
- tail: `sin((pi/2)*(T-1-y)/(E-1))` for `y>=T-E`

Otherwise the factor is `1`. Emit exactly scheduled samples `[0,T)`. Never
append a tail, resize-fill, wrap, reflect, or run a repair pass.

There is deliberately no transient state machine. One long magnitude view
owns attack smear. Every synthesis frame has one mapped analysis read. No
source grain, feedback path, dry attack, or repeated event exists. A broad
attack is expected. A separated secondary attack, periodic replica, click,
stutter, or static freeze is failure.

Any non-finite coefficient, accumulator, compensation value, or output sample
fails the render.

## Memory, Determinism, And Cost

Allocate window, interpolation buffers, component spectra, two adjacent
inverse frames per component, FFT plans, and scratch before processing. Reuse
them for every frame.

Checked working state remains at or below `32 MiB` for stereo at
`N<=131072`, excluding borrowed input and required output. Capacity is
identical for matched one-second and one-hour requests. No allocation occurs
after the first frame begins.

Mono performs one forward and one inverse FFT per new frame. Stereo performs
two of each. Work is `O((T/H)*N*log N)`; memory is duration-independent.
Counter-addressed phase, fixed traversal, explicit ties, and no parallel
reduction require sample-bit-identical output for the same request on one
supported deterministic target.

The renderer is offline only. It never executes or allocates on the audio
thread.

## Recovered PaulX Reference

Batch 31.20 rendered the frozen ten-source inventory through the pinned
PaulXStretch `1.6.0` core at revision
`8ec191fdd7203354c79391cbc04c9fd83fa30ea0`.

Capture used the source transform path with UI buffer control `16384`, default
window, default-disabled onset and optional spectral processing, two upstream
channel engines, and first-`T` crop from the upstream `T+2H` render boundary.
The ignored capture lives under
`target/creative-stretch-paulx-reference-31-20/`.

Qualification against the installed 1.6.0 default render of retained musical
source `M001` at `4x` found `2048`-sample RMS-envelope correlation `0.881` and
`0.889` for the two channels, with zero and one block lag. The raw core was
`3.086` and `3.035 dB` louder, matching the installed app's `-3 dB` main
volume. Crest is gain-invariant.

Worst-channel crest-factor growth, in dB:

| Source | `4x` | `8x` | `16x` |
| --- | ---: | ---: | ---: |
| uniform noise | 9.932 | 11.899 | 10.432 |
| Rademacher noise | 14.809 | 15.525 | 15.711 |
| amplitude-modulated noise | 13.020 | 12.553 | 14.091 |
| harmonic pad | 6.348 | 6.745 | 6.314 |
| impulse train | -27.569 | -27.301 | -26.884 |

The old `6 dB` ceiling is not PaulX-calibrated. The rejected Signal
`RenewalSpectral` uniform-noise result, `8.263162 dB` at `4x`, is below the
matching PaulX row. It remains rejected under its frozen brief; it is not
evidence that the target family is unattainable.

Other recovered reference bounds:

| Metric | `4x` | `8x` | `16x` |
| --- | ---: | ---: | ---: |
| low-tone error, cents | 7.410 | 8.816 | 7.667 |
| mid-tone error, cents | 2.462 | 5.374 | 4.838 |
| chord max-partial error, cents | 7.976 | 9.331 | 13.457 |
| impulse 95%-energy width, frames | 80,071 | 156,004 | 309,591 |
| impulse centroid error, frames | 49,187 | 114,692 | 246,058 |
| max separated impulse regions | 1 | 1 | 1 |
| uniform-noise autocorrelation peak | 0.01722 | 0.01773 | 0.01709 |
| uniform-noise block-RMS CV | 0.38775 | 0.46001 | 0.49297 |
| mid-tone block-RMS CV | 0.61727 | 0.67914 | 0.70864 |
| mapped silence-gap RMS, dB relative | 2.565 | 2.753 | 3.062 |

The large impulse and mapped-gap values are reference character and scheduler
evidence, not Signal targets. A candidate may improve event placement and gap
separation while retaining the preferred long-form sound.

## Candidate Shape

Use one disposable worktree and branch:

- worktree: `signal-candidate-31-23`
- branch: `candidate/g10-031-variance-compensated-renewal`

Add one private family only:

`crates/signal-dsp-stretch/src/creative_variance_compensated_renewal/`

Use `mod.rs`, `plan.rs`, `analysis.rs`, `phase.rs`, `synthesis.rs`, and
`tests.rs`. The isolated `lib.rs` may declare the module privately. Do not
change production tiers, public APIs, cache identity, artifact plans, reports,
binaries, fixtures, feature flags, dependencies, or product routes.

File ownership is fixed:

- `mod.rs`: private request, error, and render entry point
- `plan.rs`: request validation, transform selection, source map, component
  orientation, block count, and memory bound
- `analysis.rs`: component reads, cubic interpolation, analysis window, and
  magnitude analysis
- `phase.rs`: `mix64`, frame/bin addressing, base/side renewal, Hermitian
  completion, DC, Nyquist, and silence rules
- `synthesis.rs`: preallocated workspace, inverse frames, compensated blend,
  linked decode, exterior envelope, finiteness, and exact crop
- `tests.rs`: construction types, structural controls, frozen synthetics, and
  reference-relative measurements

Comparator and listening assembly stays ignored under `target/`. No hidden
review API, report mode, or experimental module tree enters `main`.

## Construction And Compile Completion

Construction is separate from admission. The candidate is not ready for an
evidence gate until its complete private source and tests compile.

Freeze these internal signatures before implementation:

- `CandidateRequest<'a>` owns `input: &'a [f32]`, `channels: usize`,
  `sample_rate: u32`, `target_frames: usize`, `seed: u64`, and `space: f32`
- `CandidateError` is one closed private enum covering request, size,
  allocation-bound, and non-finite-processing failures
- `render(CandidateRequest<'_>) -> Result<Vec<f32>, CandidateError>` is the
  only candidate entry point
- planning uses `usize` for allocated lengths, checked `u128` only for the
  rational map numerator and denominator, `f64` for map and coefficient
  evaluation, and `f32` for stored samples and spectra

All test accumulators have concrete types at declaration. In particular, the
side-power relation uses a first-render `f64` baseline followed by direct
`f64` comparisons; it does not use an unconstrained `Option`. Empty vectors,
empty collections, numeric folds, and `None` values crossing a helper boundary
must name their element or accumulator type explicitly.

Test names use two prefixes only:

- `variance_compensated_renewal_structural_`
- `variance_compensated_renewal_synthetic_`

No required test is ignored. The compile-completion command is
`effigy test compile`. Compiler diagnostics may be repaired during construction
only when the edit changes types, imports, visibility, ownership plumbing, or
test assembly without changing a DSP formula, constant, request decision,
source, measurement, threshold, or assertion. Any diagnostic requiring such a
semantic change stops construction and returns to a new brief.

After one clean compile-completion run, create one local checkpoint commit on
the isolated branch. Record its hash. Candidate source, tests, and assertions
are then frozen. No edit or rebuild-as-repair follows a structural or later
gate miss. The branch is never pushed.

## Frozen Synthetic Inventory

Use `F=48000`, `L=96000`, and target lengths `4L`, `8L`, and `16L`. Unless a
row says otherwise, source support outside `24000<=n<72000` is exact zero.
Apply a `2048`-sample half-cosine entrance and exit to sustained regions.

Mono sources:

- low tone: `0.5*sin(2*pi*110*n/F)`
- mid tone: `0.5*sin(2*pi*440*n/F)`
- chord: equal `0.1` components at `110`, `164.813778`, `220`, `277.182631`,
  and `329.627557` Hz
- harmonic pad: partials `k=1..8` at `110*k` Hz with amplitude `0.35/k`
- impulse: value `1` at `n=L/2`, exact zero elsewhere
- impulse train: values `1`, `-0.8`, `0.65`, and `-0.5` at frames `19200`,
  `38937`, `58103`, and `77797`
- silence gap: harmonic pad with exact zero from `42000` through `53999`

For noise, use little-endian ASCII tag `RNWTEST0`. For active index `n`, set
`r=mix64(n xor TEST)`:

- uniform: `0.5*(2*((r>>11)/2^53)-1)`
- Rademacher: `+0.5` when the high bit is one, otherwise `-0.5`
- amplitude-modulated: Rademacher multiplied by
  `0.5+0.375*sin(2*pi*1.7*n/F)`

Stereo relations:

- duplicate: identical mid tone
- common polarity: duplicate and its exact negation
- anti-phase: mid tone left and its exact negation right
- delay: harmonic pad left and zero-padded `37`-sample delay right
- mixed: chord plus `0.2` uniform noise left; delayed chord minus the same
  `0.2` noise right

Pitch uses the central half of mapped active support. Crest uses mapped active
support after the exterior envelope. Impulse width is the shortest interval
containing `95%` of squared energy. Impulse centroid is the squared-energy
centroid. Secondary regions use a `10 ms` RMS envelope, `5 ms` hop, `-30 dB`
relative floor, and `50 ms` separation. Noise periodicity is maximum normalized
autocorrelation from `20 ms` through `1 s`. Block-RMS variation uses `50 ms`
blocks and `25 ms` hop over mapped active support.

## Admission

Admission begins only from the compile-complete checkpoint. Failure from the
structural gate onward stops the sequence.

### 1. Structural And Hard-Integrity Gate

Exercise empty, one-sample, sub-window, exact-window, silence, DC, impulses,
tones, chords, deterministic noise, duplicate stereo, swap, common polarity,
exact anti-phase, delayed stereo, and mixed stereo at all ratios. Cover
`space=0`, `0.5`, and `1` without a full Cartesian product.

Pass requires:

- every request decision occurs before output allocation
- exact `T`, finite output, and exact-zero silence
- byte-identical repeat renders; changed seed changes active interior output
- exact `N`, `H`, `x_j`, interpolation, blend, compensation, and crop laws
- strictly increasing source centres and no second source read
- analysis phase absent; every active phase has one frame/bin address
- state at most `32 MiB`, duration-independent, fully allocated before render
- duplicate stereo equals duplicated mono within `1e-6`
- swap swaps, common polarity negates, and anti-phase remains anti-phase within
  `1e-6`
- `space` preserves silence, duplicate stereo, total frame mid/side spectral
  power, and length
- non-empty `T>=8` output begins and ends at exact zero
- every non-zero sustained row has non-zero interior energy and no exact-zero
  interior block of length `H`
- no forbidden carrier, recurrence, detector, layer, limiter, or post-gain

These are integrity laws. No creative-reference score can waive them.

### 2. Reference-Relative Synthetic Gate

Use the frozen inventory above. Render all rows at `4x`, `8x`, and `16x`.
RMS matching and level scaling occur only in the comparator, never inside the
renderer. Crest growth is invariant to that common scale.

Pass requires:

- each crest-growth row at most its matching PaulX table value plus `2 dB`
- dominant-tone and chord-partial error at most the matching PaulX row plus
  `2` cents
- impulse 95%-energy width from `0.75` through `1.50` times PaulX
- impulse centroid error no greater than PaulX plus `10%` of PaulX width
- no more separated secondary impulse regions than PaulX; largest relative
  secondary peak at most `3 dB` higher
- non-periodic-noise autocorrelation no greater than PaulX plus `0.05`
- stationary tone and noise block-RMS CV no greater than PaulX plus `0.05`
- mapped silence-gap RMS no greater than PaulX plus `3 dB`
- zero dropout, non-finite, or interior-discontinuity findings

Reference-relative failure rejects this candidate. It does not close neutral
`Dream`. Record the dominant mismatch before any new architecture decision.

### 3. Long-Form Mono Gate

Use retained percussion, bass, vocals, pads/sustains, and full-mix sources at
`4x`, `8x`, and `16x`. Review `8x` first. Conceal identities. Apply the
retained common-RMS and `0.95` peak listening policy.

Compare only neutral `VarianceCompensatedRenewalSpectral` and PaulXStretch. Record
smoothness, musical usefulness, source identity, evolution, grain, atonal
ringing, periodicity, event stability, crest distraction, and exterior
behavior.

Pass requires:

- no unusable candidate row
- candidate preferred or tied on at least `12` of `15` rows
- no source family loses all three ratios
- no exposed vocoder colour, rough periodicity, cyclic repetition, doubled
  attack, stutter, or static freeze

Listening is promotion authority. Objective scores cannot waive a loss.

### 4. Linked-Stereo Gate

Only after mono passes, run structural transformations and the retained
long-form stereo pack at all ratios and `space` values.

An eligible listener independent of the operator assesses centre stability,
width, width pumping, one-sided texture, channel echo, low-frequency image,
and monotonic `space`. The operator's one-ear hearing does not satisfy this
gate.

Pass requires no unusable row, no mono or duplicate instability, and image
stability preferred or tied with PaulXStretch on at least `12` of `15`
neutral-`space` rows. Missing independent review blocks admission.

## Rejection, Cleanup, And Minimal Admission

Any admission-gate miss rejects the complete candidate. Record one dominant
cause and stopped gate. Delete the disposable worktree, branch, local
checkpoint, module, tests, build state, and candidate listening assembly.
Retain only external comparator evidence under ignored `target/` and the docs
closeout.

Do not tune or rerun the failed candidate. Another implementation requires a
new complete brief. Failure does not close the PaulX-like product target unless
the operator explicitly stops it.

Only a complete pass may admit:

- the private `creative_variance_compensated_renewal` family
- its fixed-ratio neutral-`Dream` request and renderer
- structural and synthetic regression tests
- one internal creative-engine version identifier

Do not admit a public character enum, `motion`, `detail`, cache schema,
artifact surface, report mode, runtime route, pitch path, dynamic ratio,
another creative character, router, Loophole, or Chorus integration.

## Batch 31.23 Result

The isolated implementation compiled and was frozen at local checkpoint
`2548c27947b28a59a265cf1bb60ca2b03455b08a`. Seven structural tests then
passed. The synthetic test command also returned green, but review before
listening found that its frozen assertions did not implement the complete gate:

- impulse-train crest growth was not measured against its PaulX row
- separated secondary impulse regions and peak level were not measured
- noise autocorrelation sampled every sixteenth lag instead of taking the
  required maximum
- the full interior-discontinuity condition was not implemented

That green receipt is invalid. The checkpoint could not be repaired or rerun
under this brief. Long-form audio was assembled but never opened, then deleted
with the worktree, branch, checkpoint, private module, tests, and candidate
build state.

This is an evidence-construction rejection. It establishes no DSP or listening
result for the variance-compensated renewal topology. The PaulX-like product
target remains open.

## Sources

- [Creative Stretch Source Triangulation](../research/specimen-dossiers/creative-stretch-source-triangulation.md)
- [PaulXStretch pinned magnitude and phase path](https://github.com/essej/paulxstretch/blob/8ec191fdd7203354c79391cbc04c9fd83fa30ea0/Source/PS_Source/Stretch.cpp#L109-L263)
- [PaulXStretch pinned frame blend and source accumulator](https://github.com/essej/paulxstretch/blob/8ec191fdd7203354c79391cbc04c9fd83fa30ea0/Source/PS_Source/Stretch.cpp#L320-L563)
- [PaulXStretch pinned output-duration boundary](https://github.com/essej/paulxstretch/blob/8ec191fdd7203354c79391cbc04c9fd83fa30ea0/Source/PS_Source/StretchSource.cpp#L775-L782)

## Next Task

Use the audited successor brief for execution authority. Run `g10.031` Batch
31.25 only; do not recover this candidate's deleted source or tests.
