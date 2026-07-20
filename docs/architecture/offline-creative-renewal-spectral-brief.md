# Offline Creative RenewalSpectral Renderer Brief

Status: frozen; candidate rejected at crest gate
Owner: dsp
Updated: 2026-07-20
Contract: `085`
Roadmap: `g10.031`, Batch 31.17

## Decision

Build one Signal-owned `RenewalSpectral` candidate for neutral `Dream` from
`4x` through `16x`. It is one offline magnitude-resynthesis renderer:

- one exact sample-centred source/output map
- one long analysis transform and one output-frame cadence
- source phase discarded after every analysis
- fresh deterministic stochastic phase on every synthesis frame
- one pairwise equal-power frame crossfade
- one linked mid/side law for stereo
- fixed energy calibration, bounded exterior envelope, and exact crop

There is no instantaneous-frequency carrier, phase propagation, peak tracker,
magnitude recurrence, transient detector, onset reset, component layer,
limiter, compressor, or post-render normalization.

This is a clean-room Signal architecture. PaulXStretch supplies whole-path and
audible-target evidence only. Its expression, constants, thresholds, random
generator, window, amplitude correction, and control flow do not transfer.

## Supported Request

The isolated candidate accepts:

- borrowed mono or linked-stereo interleaved finite `f32`
- integer sample rate `F` from `8,000` through `192,000` Hz
- non-empty input frame count `L`
- exact target frame count `T`
- checked ratio range `4*L <= T <= 16*L`
- one explicit `u64` seed
- finite normalized `space` in `[0,1]`

`L` and `T` may not exceed `2^53-1`. Empty input with `T=0` returns empty
output. Every other empty, zero-target, overflowed, non-finite,
unsupported-channel, unsupported-rate, compression, or range-miss request
fails before output allocation. Values are rejected, not clamped.

The candidate owns neutral `Dream` only. `motion` and `detail` are deliberately
absent from its private request. They require later admitted target evidence;
the candidate does not invent macro behavior to widen its claim. Dynamic
ratio, pitch composition, reverse, other creative characters, cache, artifact
routing, RealtimePreview, audio-thread execution, and public product exposure
are unsupported.

Input and required output storage do not count as working state. Every other
allocation does.

## Transform Geometry

Freeze one sample-rate-normalized support:

`N=clamp(nearest_power_of_two(round(2*F/3)),8192,131072)`

Power-of-two ties select the larger value. The synthesis block length is:

`H=N/2`.

At `44.1` and `48` kHz, `N=32768`. The rule is derived from roughly
two-thirds of a second of tonal support, not from the PaulXStretch UI label.

The analysis window is one periodic Hann:

`w[n]=0.5-0.5*cos(2*pi*n/N)`, `0<=n<N`.

Its fixed energy calibration is:

`G=sqrt(N/sum_n w[n]^2)`.

No character, ratio, seed, channel, or source event changes `N`, `H`, `w`, or
`G`. There is no second scale, adaptive transform, short transient window, or
post-render layer.

## Source Map And Scheduler

Output block `j` begins at `y_j=j*H`. Generate frame `j` and frame `j+1` for
every block whose support intersects `[0,T)`. Frame `j` owns source centre:

`x_j=((y_j+0.5)*L/T)-0.5`.

Evaluate the fraction as checked `u128` numerator `(2*y_j+1)*L` over
denominator `2*T`, then convert once to `f64` and subtract `0.5`. This is the
sole source/output map. It is strictly increasing and maps output sample
centres to source sample centres. Frame generation may extend beyond `T` only
to synthesize the last cropped block; it does not extend the returned output.

For analysis index `n`, read:

`p=x_j+n-(N-1)/2`.

Let `i=floor(p)` and `u=p-i`. Use four-point cubic Lagrange interpolation over
source samples at `i-1`, `i`, `i+1`, and `i+2`:

`v(p)=c_-1*v[i-1]+c_0*v[i]+c_1*v[i+1]+c_2*v[i+2]`, where

- `c_-1=-u*(u-1)*(u-2)/6`
- `c_0=(u+1)*(u-1)*(u-2)/2`
- `c_1=-(u+1)*u*(u-2)/2`
- `c_2=(u+1)*u*(u-1)/6`

Samples outside `[0,L)` are exact zero. Every linked component reads the same
`p`. No later stage shifts the cursor, repeats an event, or owns another
timeline.

## Linked Analysis Representation

Mono has one native component. Stereo uses the orthonormal components:

`M=(L+R)/sqrt(2)`

`D=(L-R)/sqrt(2)`.

Interpolate, window, and transform each component. Retain only non-negative-bin
magnitudes `A_r[j,b]`; discard every input coefficient phase immediately.
Here `r` is mono, mid, or side.

Before frame processing, scan each component from source start in fixed order.
Canonicalize signed zero. Its orientation `o_r` is the sign of the first
exactly non-zero sample, or `+1` when the component is identically zero. Also
retain the exact-silence flag. Orientation preserves deterministic duplicate,
swap, and common-polarity behavior without retaining spectral phase or solving
the rejected per-bin relation proof.

The scan is metadata only. It cannot change the source map or frame schedule.

## Deterministic Phase Renewal

Every interior bin receives new counter-addressed phase. There is no mutable
random stream and no preceding-frame phase state.

Define `mix64(z)` with wrapping `u64` arithmetic:

1. `z=(z xor (z>>30))*0xBF58476D1CE4E5B9`
2. `z=(z xor (z>>27))*0x94D049BB133111EB`
3. return `z xor (z>>31)`

For frame `j`, bin `b`, and stream tag `s`:

`z=mix64(seed xor mix64(j xor FRAME) xor rotate_left(mix64(b xor BIN),21) xor s)`.

The little-endian ASCII tags are:

- `FRAME="RNWFRAME"`
- `BIN="RNWBIN00"`
- base stream `s="RNWBASE0"`
- side stream `s="RNWSIDE0"`

Convert the high 53 bits of `z` to `u=(z>>11)/2^53` in `[0,1)`, then set:

`phase(z)=2*pi*u-pi`.

For interior bin `b`, base phase is `theta[j,b]`. Mono and mid use:

`Y_r[j,b]=o_r*A_r[j,b]*exp(i*theta[j,b])`.

The side component uses one linked opposing field. Let bin frequency
`f_b=b*F/N`, `t=clamp((f_b-250)/(1500-250),0,1)`, and
`h=t*t*(3-2*t)`. With side phase `zeta[j,b]` from the side stream:

`Y_D[j,b]=o_D*A_D[j,b]*exp(i*(theta[j,b]+space*h*zeta[j,b]))`.

Thus `space=0` shares one phase field between mid and side. Higher `space`
adds bounded, high-frequency-weighted side diffusion from one shared linked
decision. It never creates channel-local random streams. Duplicate stereo has
zero side and remains duplicate at every `space` value.

DC is the real signed magnitude `o_r*A_r[j,0]`. Nyquist is exact zero. Negative
bins are conjugate mirrors. Exact-silent components remain exact zero.

Phase has no propagation, dormancy, or reactivation rule because it has no
state. Tonal peaks have no identities. Every frame is one complete renewal.

## Frame Synthesis And Crest Ownership

Inverse-transform every component with scale `1/N`. Let its real frame be
`z_r[j,n]`. No synthesis window is applied.

For block offset `0<=n<H`, define the equal-power pair:

`a[n]=cos((pi/2)*(n+0.5)/H)`

`b[n]=sin((pi/2)*(n+0.5)/H)`.

Synthesize component sample:

`q_r[j,n]=G*(a[n]*z_r[j,H+n]+b[n]*z_r[j+1,n])`.

Decode stereo after the crossfade:

`L=(q_M+q_D)/sqrt(2)`

`R=(q_M-q_D)/sqrt(2)`.

The power-complementary crossfade and fixed window-energy gain are the only
interior amplitude law. There is no overlap denominator, frame-local RMS gain,
peak follower, limiter, compressor, output matching, or crest repair.

The exterior envelope length is `E=min(H,floor(T/4))`. If `E>=2`, multiply
sample `y` by both applicable factors:

- head: `sin((pi/2)*y/(E-1))` for `y<E`, otherwise `1`
- tail: `sin((pi/2)*(T-1-y)/(E-1))` for `y>=T-E`, otherwise `1`

If `E<2`, the exterior factor is `1`. This envelope is part of the renderer,
not a repair pass. It makes the first and last sample exact zero for normal
supported renders while leaving at least half the output outside the fades.

Emit exactly the first `T` scheduled samples. A non-finite coefficient,
accumulator, or output sample fails the render. Never resize-fill, wrap,
reflect, append a tail, or run a second normalization.

## Transients And Replica Prevention

The renderer deliberately has no transient state machine:

- one long magnitude view owns attack smear
- every output frame has one mapped source read
- no onset is detected, reset, reassigned, isolated, or mixed back
- no source grain, cyclic buffer, feedback path, or repeated attack exists
- `space` changes only the linked side phase field

Impulse spread is expected creative behavior. A separated secondary attack,
periodic replica, stutter, or static freeze is failure.

## Memory, Determinism, And Cost

Allocate the window, interpolation buffers, component spectra, two adjacent
inverse frames per component, FFT plans, and scratch before processing. Reuse
them for every frame.

Checked working state must remain at or below `32 MiB` for stereo and
`N<=131072`, excluding borrowed input and required output. Capacity must be
identical for matched one-second and one-hour requests. No allocation occurs
after the first frame begins.

Mono performs one forward and one inverse FFT per new frame. Stereo performs
two of each. Work is `O((T/H)*N*log N)` with linear interpolation and bin work;
memory is independent of duration. Counter-addressed phase, fixed traversal,
explicit ties, and no parallel reduction require sample-bit-identical output
for the same request on one supported deterministic target.

The renderer is offline only. It never runs or allocates on the audio thread.

## Candidate Shape

Use one disposable worktree and branch:

- worktree: `signal-candidate-31-18`
- branch: `candidate/g10-031-renewal-spectral`

The candidate adds one private family only:

`crates/signal-dsp-stretch/src/creative_renewal/`

Use `mod.rs`, `plan.rs`, `analysis.rs`, `phase.rs`, `synthesis.rs`, and
`tests.rs`. The isolated `lib.rs` may declare the module privately. Do not
change production tiers, public APIs, cache identity, artifact plans, reports,
binaries, fixtures, feature flags, dependencies, or product routes.

Comparator and candidate listening assembly stays ignored under `target/`.
No hidden review API, report mode, or experimental module tree enters `main`.

## Frozen Synthetic Inventory

Creative synthetic rows use `F=48000`, `L=96000`, and target lengths
`4*L`, `8*L`, and `16*L`. Unless a row says otherwise, source support outside
`24000<=n<72000` is exact zero. Apply a `2048`-sample half-cosine entrance and
exit to sustained active regions.

Freeze these mono sources:

- low tone: `0.5*sin(2*pi*110*n/F)`
- mid tone: `0.5*sin(2*pi*440*n/F)`
- chord: equal `0.1` components at `110`, `164.813778`, `220`, `277.182631`,
  and `329.627557` Hz
- harmonic pad: partials `k=1..8` at `110*k` Hz with amplitude `0.35/k`
- impulse: value `1` at `n=L/2`, exact zero elsewhere
- impulse train: values `1`, `-0.8`, `0.65`, and `-0.5` at source frames
  `19200`, `38937`, `58103`, and `77797`
- silence gap: the harmonic pad with exact zero from `n=42000` through `53999`

The fixed noise tag is little-endian ASCII `"RNWTEST0"`. For active sample
index `n`, set `r=mix64(n xor TEST)`, using the renderer's frozen `mix64`:

- uniform noise: `0.5*(2*((r>>11)/2^53)-1)`
- Rademacher noise: `+0.5` when the high bit is one, otherwise `-0.5`
- amplitude-modulated noise: Rademacher noise multiplied by
  `0.5+0.375*sin(2*pi*1.7*n/F)`

Freeze these stereo relations from the mono sources:

- duplicate: identical mid tone
- common polarity: the duplicate row and its exact negation
- anti-phase: mid tone left and its exact negation right
- delay: harmonic pad left and a zero-padded `37`-sample delay right
- mixed: chord plus `0.2` uniform noise left; the `37`-sample delayed chord
  minus the same `0.2` noise right

Pitch uses the central half of mapped active support. Crest uses mapped active
support after excluding the renderer's fixed exterior envelope. Impulse width
is the shortest interval containing `95%` of squared output energy. Impulse
centroid is the squared-energy centroid. Secondary impulse regions use a
`10 ms` RMS envelope, a `5 ms` hop, a `-30 dB` relative activity floor, and
`50 ms` minimum separation. Noise periodicity is the largest normalized
autocorrelation from `20 ms` through `1 s`. Block-RMS variation uses `50 ms`
blocks with `25 ms` hop over mapped active support.

## Fixed Admission

Failure stops the sequence.

### 1. Structural Gate

Exercise empty, one-sample, sub-window, exact-window, silence, DC, impulses,
tones, chords, deterministic uniform and Rademacher noise, duplicate stereo,
swap, common polarity, exact anti-phase, delayed stereo, and mixed stereo at
`4x`, `8x`, and `16x`. Cover `space` values `0`, `0.5`, and `1` without a full
Cartesian product.

Pass requires these hard integrity conditions:

- valid empty input returns empty; every invalid request fails before output
- output length is exactly `T`; every output is finite
- exact-zero input produces exact-zero output
- repeated renders are byte-identical; a changed seed changes an active
  interior sample
- `N`, `H`, every `x_j`, and every interpolation coefficient match this brief
- analysis phase is absent from synthesis; every active interior phase comes
  from its frame/bin counter address
- source centres are strictly increasing and no second source read exists
- renderer-owned state is at most `32 MiB`, duration-independent, and fully
  allocated before processing
- duplicate stereo equals duplicated mono within `1e-6`
- channel swap swaps output and common polarity negates output within `1e-6`
- exact anti-phase remains anti-phase within `1e-6`
- `space` preserves exact silence, duplicate stereo, each frame's total
  mid/side spectral power, and output length
- every non-empty render with `T>=8` begins and ends at exact zero
- no forbidden carrier, recurrence, detector, layer, limiter, or post-gain
  stage exists

The `1e-6` stereo bound is an algebraic implementation tolerance for the
mid/side equations, not a copied creative-character metric.

### 2. Crest Gate

The first rendered row is neutral `Dream`, `space=0`, `4x`, and the retained
deterministic uniform-noise source. Then run uniform noise, Rademacher noise,
amplitude-modulated noise, harmonic pads, and impulse trains at all three
ratios.

Worst-channel RMS-matched crest-factor growth over active support must not
exceed `6 dB`. This is comparator-calibrated: retained PaulXStretch `Dream`
peaked at `3.88 dB`, leaving `2.12 dB` implementation margin. Any miss rejects
the candidate before later metrics. No gain, distribution, window, or gate
repair follows.

### 3. Comparator-Calibrated Synthetic Gate

Under ignored `target/`, render the same synthetic sources through pinned
PaulXStretch `v1.6.0`, default processing, UI FFT `16384`, then exact-crop and
RMS-match them with the retained policy. Do not check those files into `main`.

Run tones, chords, harmonic pads, impulses, impulse trains, amplitude-modulated
noise, non-periodic noise, and silence gaps at `4x`, `8x`, and `16x`.
Candidate pass requires:

- dominant-tone and chord-partial error no more than the matching PaulXStretch
  row plus `2` cents measurement tolerance
- impulse 95%-energy width from `0.75` through `1.50` times the matching
  PaulXStretch width
- impulse energy-centroid error no greater than the matching PaulXStretch error
  plus `10%` of that row's PaulXStretch 95%-energy width
- no more separated secondary impulse regions than the matching PaulXStretch
  row; their largest relative peak may be at most `3 dB` higher
- non-zero-lag autocorrelation peak on non-periodic noise no more than the
  matching PaulXStretch value plus `0.05`
- block-RMS coefficient of variation on stationary tones and noise no more
  than the matching PaulXStretch value plus `0.05`
- silence-gap RMS no more than `3 dB` above the matching PaulXStretch row
- zero dropout, non-finite, or interior-discontinuity findings

The relative margins are measurement allowances around the named comparator,
not claims of transparent fidelity. Metrics reject a direction mismatch.
They do not promote musical quality.

### 4. Long-Form Mono Gate

Use the retained percussion, bass, vocals, pads/sustains, and full-mix sources
at `4x`, `8x`, and `16x`. Review `8x` first. Conceal identities and retain the
common-RMS / `0.95` peak listening policy.

Compare neutral `RenewalSpectral` only with PaulXStretch. Record smoothness,
musical usefulness, source identity, evolution, grain, atonal ringing,
periodicity, event stability, crest distraction, and exterior behavior.

Pass requires:

- no unusable candidate row
- candidate preferred or tied on at least `12` of `15` rows
- no source family loses all three ratios
- no exposed vocoder colour, rough periodicity, cyclic repetition, doubled
  attack, stutter, or static freeze

Listening is promotion authority. Objective scores cannot waive a loss.

### 5. Linked-Stereo Gate

After mono passes, run the structural stereo transformations and retained
long-form stereo pack at all three ratios and `space` values `0`, `0.5`, and
`1`.

An eligible listener independent of the operator must assess centre stability,
width, width pumping, one-sided texture, channel echo, low-frequency image,
and whether `space` moves in one useful direction. The operator's one-ear
hearing does not satisfy this gate.

Pass requires no unusable row, no mono or duplicate instability, and candidate
image stability preferred or tied with PaulXStretch on at least `12` of `15`
neutral-`space` rows. Missing independent review blocks admission.

### 6. Minimal Admission

Only a complete pass may admit:

- the private `creative_renewal` family
- its fixed-ratio neutral-`Dream` request and renderer
- the structural and synthetic regression tests
- one internal creative-engine version identifier

Do not admit a public character enum, `motion`, `detail`, cache schema,
artifact surface, report mode, runtime route, pitch path, dynamic ratio,
another creative character, router, Loophole, or Chorus integration.

## Rejection And Cleanup

Any structural, crest, synthetic, mono, or stereo miss rejects the complete
candidate. Record one dominant cause and stopped gate. Delete the disposable
worktree, branch, module, tests, candidate build state, and candidate listening
assembly. Retain only the existing external comparator pack under ignored
`target/` and the docs closeout.

Do not tune or rerun the failed candidate. Return to architecture
reassessment. Another implementation requires a materially different complete
brief; a window, hop, phase distribution, spatial coefficient, gain, or gate
change is not a new family.

## Candidate Decision

Batch 31.18 implemented this brief once in its named disposable worktree.
Compile-only validation and the complete structural gate passed. The mandated
first crest row then measured `8.263162 dB` of crest-factor growth against the
frozen `6 dB` ceiling.

The dominant cause is uncontrolled cross-bin waveform summation after complete
independent phase renewal. No correction, rerun, later synthetic row, listening
gate, or admission followed. The worktree, branch, private module, tests, and
candidate build state were removed. The frozen brief remains evidence, not
implementation authority.

## Sources

- [Creative Stretch Source Triangulation](../research/specimen-dossiers/creative-stretch-source-triangulation.md)
- [PaulXStretch pinned magnitude and phase path](https://github.com/essej/paulxstretch/blob/8ec191fdd7203354c79391cbc04c9fd83fa30ea0/Source/PS_Source/Stretch.cpp#L109-L263)
- [PaulXStretch pinned frame and source scheduler](https://github.com/essej/paulxstretch/blob/8ec191fdd7203354c79391cbc04c9fd83fa30ea0/Source/PS_Source/Stretch.cpp#L320-L563)

## Next Task

Run Batch 31.19 only. Reassess neutral-`Dream` crest ownership at architecture
level, including whether a materially different source-backed whole-renderer
path remains. Do not tune or reimplement `RenewalSpectral`.
