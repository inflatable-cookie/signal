# Offline Creative ContinuousExcitationComplexRelation Renderer Brief

Status: rejected at relation proof; current diffusive owner closed
Owner: dsp
Updated: 2026-07-19
Contract: `085`
Roadmap: `g10.031`, Batch 31.7

## Outcome

Batch 31.8 implemented this brief once in the disposable
`signal-candidate-31-8` worktree. Compile-only completion passed. The first
admitted coefficient proof then failed exact anti-phase enumeration: the
renderer produced `-1+0i` where the proof expected `+1-0i`.

The miss exposed an incompatible proof expectation. Common polarity on exact
anti-phase stereo is also a channel swap, so componentwise negation equals a
plain swap; it cannot also equal a negated swap. The frozen stop rule still
rejects the candidate on any relation-proof miss. No test correction, rerun,
renderer row, crest row, synthetic row, or listening row followed. The
worktree, branch, six-file module, tests, and build state were deleted.

This closes `ContinuousExcitationComplexRelation` and the current diffusive
owner. The brief remains rejection evidence, not implementation authority.

## Decision

Build one final fixed-ratio candidate named
`ContinuousExcitationComplexRelation`. It preserves the accepted continuous
full-complex excitation topology and replaces every polar native-relation
calculation with one value-symmetric complex relation law.

This is not a repaired Batch 31.6 branch. It is one new disposable candidate
under a complete brief. No angle subtraction, wrapped native delta,
index-owned reference, threshold fallback, phase-only excitation, limiter,
crest repair, or post normalization is allowed.

The candidate owns `Dream`, `Spectral`, and `Rough` from `4x` through `16x`.
`Cloud`, `Cyclic`, range routing, dynamic ratio, cache, pitch composition,
RealtimePreview, and product APIs remain closed.

## Reassessment Proof

Batch 31.6 failed because it reconstructed native relation as:

`exp(i*wrap(arg(X_c)-arg(R)))`.

That expression is mathematically polarity-invariant but not value-stable
enough near branch cuts and low-energy bins. Its common-polarity output missed
the frozen bound by `0.0013287` while channel swap remained exact.

The replacement uses direct complex products. For a non-zero linked sum:

`U_c=unit(X_c*conj(R))`.

Under common polarity, both `X_c` and `R` negate, so their product is unchanged.
Under channel swap, `R` is unchanged and the `U_c` values swap. No angle or
wrapped subtraction exists.

Exact linked cancellation cannot have a non-zero reference that is both
swap-invariant and polarity-odd. It therefore has a separate explicit law:
one unoriented complex axis plus the already linked source-orientation state.
The law distinguishes exact whole-source anti-phase from incidental
single-bin cancellation. Duplicate, swap, polarity, anti-phase, silence, and
signed-zero behavior are frozen below.

## Supported Request

Accept only:

- mono or linked stereo interleaved finite `f32`
- sample rate `8,000` through `192,000` Hz
- exact target frame count `T`
- non-empty ratio `q=T/L` in `[4,16]`
- `Dream`, `Spectral`, or `Rough`
- finite `motion`, `detail`, and `space` in `[0,1]`
- one explicit `u64` seed

Valid empty input with `T=0` returns empty output. Every other zero length,
non-finite value, unsupported channel count, unsupported character, range
miss, length mismatch, or arithmetic overflow fails before allocation. Values
are rejected, never clamped.

The input is borrowed. Required output storage is excluded from working state;
all other state is included.

## Transform And Timeline

For sample rate `F`:

`N=clamp(next_power_of_two(ceil(0.30*F)),4096,65536)`

`S=N/4`

Use one centred periodic square-root Hann:

`w[n]=sqrt(0.5-0.5*cos(2*pi*n/N))`, `0<=n<N`.

Synthesis centres are `y_k=k*S` for every signed `k` whose centred support
intersects `[0,T)`. Keep signed pre-roll and post-roll indices.

The only source map is:

`x_k=((y_k+0.5)*L/T)-0.5`.

Read analysis position `p=x_k+n-N/2` by linear interpolation. Outside
`[0,L)` is exact zero. Every channel uses the same `p`. No stage moves the
cursor, adds a source read, or creates another event timeline.

## Source Orientation

Scan source frames once in `f64`. For frame `j`, calculate:

`s_j=sum_c input[j,c]`.

Canonicalize every signed zero to `+0`. Let `o` be the sign of the first
exactly non-zero `s_j`. If none exists, set `o=+1` and
`joint_sum_zero=true`; otherwise set `joint_sum_zero=false`.

Channel swap leaves this state unchanged. Common polarity negates `o` when
`joint_sum_zero=false`. For exact anti-phase stereo, `joint_sum_zero=true` and
the per-bin cancellation law owns the polarity change.

This scan is bounded-memory metadata. It does not alter the source map.

## Value-Symmetric Complex Relation

Transform every native window. For bin `b`, retain `X_c[b]` and
`M_c[b]=|X_c[b]|`. Compute the linked sum in fixed channel order:

`R=sum_c X_c`.

Canonicalize signed zero in `R`. There is no activity threshold.

Define `unit(z)` in `f64` without angle extraction:

1. Canonicalize signed zero in both components.
2. Set `scale=max(|re(z)|,|im(z)|)`.
3. If `scale=0`, return exact complex zero.
4. Set `v=z/scale` and return `v/hypot(re(v),im(v))`, rounded once to `f32`.

For `R!=0` and `X_c!=0`:

`U_c=unit(X_c*conj(R))`.

The product is evaluated in `f64`. A silent native coefficient gets `U_c=0`
and remains silent.

When `R=0` and at least one native coefficient is active, exact mono/stereo
cancellation guarantees that all active native coefficients are antipodal.
For each active coefficient, form `axis(X_c)` by setting `K_c=unit(X_c)`, then
negating it when `re(K_c)<0`, or when `re(K_c)=0` and `im(K_c)<0`. Canonicalize
signed zero. Freeze the shared unoriented axis as:

`K=unit(sum_c axis(X_c))`.

`K(X)=K(-X)` and the sum does not depend on channel order. Then:

- if `joint_sum_zero=true`, `U_c=unit(X_c*conj(K))`
- otherwise, `U_c=o*unit(X_c*conj(K))`

For incidental bin cancellation, both `o` and `X_c` negate under common
polarity, so `U_c` is invariant. For exact whole-source anti-phase, `o` stays
`+1`, the `U_c` values negate and swap, and output polarity changes without
breaking channel swap.

Joint silence sets `R=0`, `K=0`, and every `U_c=0`. DC and Nyquist use the real
part of the same `U_c` law and are forced real. No separate native-sign rule
exists.

## Continuous Excitation

Define one signed output waveform shared across linked channels. Reinterpret
signed sample index `t` as two's-complement `u64`:

`x=seed xor rotate_left(u64(t),17) xor 0x4558434954415445`.

Apply wrapping operations:

1. `x xor=x>>30`; multiply by `0xBF58476D1CE4E5B9`.
2. `x xor=x>>27`; multiply by `0x94D049BB133111EB`.
3. `x xor=x>>31`.

Set `r[t]=o` when the high bit is one and `r[t]=-o` otherwise. For frame `k`:

`e_k[n]=w[n]*r[y_k+n-N/2]`.

Transform once to `E_k[b]`. Let `sigma=sqrt(sum_n w[n]^2)` and retain:

`Q_k[b]=E_k[b]/sigma`.

Keep the complete complex `Q`; never normalize its magnitude. With flat unit
source envelope and full stochastic mix, inverse transform, synthesis window,
and OLA normalization must reconstruct `r[t]/sigma` within `1e-6`.

## Linked Carrier

For bin centre `omega_b=2*pi*b/N` and `dx=x_k-x_(k-1)`, the linked carrier uses
`R` when non-zero. Its reference phasor is `unit(R)`. For cancellation, use
`K` when `joint_sum_zero=true` and `o*K` otherwise. Joint silence is inactive.

For consecutive active frames, derive instantaneous frequency from the direct
phasor product:

`D=unit(reference_k*conj(reference_(k-1))*exp(-i*omega_b*dx))`

`omega_hat=omega_b+atan2(im(D),re(D))/dx`.

This is the only allowed angle extraction in linked propagation; native
relations never use it. Advance carrier by `omega_hat*S`. Cold activation uses
the current reference phasor. Dormancy retains frequency, advances carrier,
and becomes cold after `N` source samples.

No peak tracker, transient reset, identity lock, reassignment, or second grid
exists.

## Magnitude Evolution

Use floor `e=1e-20`. Per native channel and bin:

`ell_c=ln(max(M_c,e))`.

Apply `[1,2,1]/4` three times with replicated spectrum edges to create
`smooth3(ell_c)`, then:

`f_c=(1-frequency_blend)*ell_c+frequency_blend*smooth3(ell_c)`

`V_c=alpha*V_c_prev+(1-alpha)*f_c`.

First activation initializes `V_c=f_c`; `B_c=exp(V_c)`. One linked power law:

`P_raw=sum_(c,b) M_c[b]^2`

`P_env=alpha*P_env_prev+(1-alpha)*P_raw`

`P_candidate=sum_(c,b) B_c[b]^2`

`g=clamp(sqrt(P_env/max(P_candidate,e^2)),0.25,4.0)`

`A_c=g*B_c`.

Gain is computed before excitation and never observes output peaks or realized
`Q`. Exact native silence remains exact zero. Dormant envelopes decay to
`ln(e)` and become cold after `N` source samples.

## Character And Stereo Laws

Let `m=motion`, `d=detail`:

| Character | stochastic mix `p` | `frequency_blend` | `alpha` |
| --- | --- | --- | --- |
| `Dream` | `1.0` | `0.80*(1-d)` | `clamp(0.90-0.25*d-0.15*m,0.50,0.90)` |
| `Spectral` | `0.20+0.20*m` | `0.15*(1-d)` | `clamp(0.94-0.20*d-0.10*m,0.64,0.94)` |
| `Rough` | `1.0` | `0.10*(1-d)` | `0.45*(1-d)*(1-0.5*m)` |

For every active bin:

`C_c=A_c*exp(i*carrier)*U_c`

`D_c=A_c*Q_k[b]*U_c`

`Y_c=sqrt(1-p)*C_c+sqrt(p)*D_c`.

There is no complex renormalization or gain after the mix. DC and Nyquist are
forced real; negative bins are exact conjugates.

Character weights are non-negative `(dream,spectral,rough)` values summing to
one. Interpolate `p`, `frequency_blend`, and `alpha` once before frame zero.
Public vertices select one character; intermediate weights exist only for seam
validation.

For stereo `space=s`, at bin frequency `f`:

`h=clamp((f-200)/(1000-200),0,1)`

Set `mid=(Y_l+Y_r)/2`, `side=(Y_l-Y_r)/2`, `side'=side*(1+s*h)`, then
reconstruct channels. Mono and duplicate stereo remain unchanged. Bass below
`200 Hz` is unchanged. Maximum side gain is `2`.

## Transients, Synthesis, And Boundaries

There is no transient detector. The long window and envelope own commanded
smear. Every frame has one source read. No event is copied, reset, isolated,
mixed back, or repeated. `detail` changes envelope smoothing only.

For synthesis:

- construct native spectra from the equations above
- apply `space`, force real endpoints, and mirror conjugates
- inverse FFT with `1/N` scale
- multiply by the same square-root Hann
- add to channel-local rolling overlap rings
- add `w[n]^2` to one shared rolling normalization ring

Emit only after no later frame can touch a sample. Normalization must exceed
`1e-12` across the crop. Return exactly `[0,T)` without resize fill. There is
no wrap, reflection, hidden extension, fade, boundary layer, attack layer,
tail layer, crest repair, or second normalization.

Any non-finite coefficient, accumulator, normalizer, or sample is failure.

## Memory, Determinism, And Cost

Preallocate source and synthesis buffers, one shared excitation buffer,
complex native relations, magnitude and carrier state, window, rolling OLA,
normalization, FFT plans, and scratch. Checked working state is at most
`32 MiB` for two channels and `N<=65536`, excluding borrowed input and required
output. Capacity is independent of source duration. No allocation occurs after
frame processing begins.

One source-orientation scan precedes frames. Every frame performs one source
forward and one inverse FFT per channel plus one shared excitation forward FFT.
Cost is linear in `T/S` and `N log N`.

Fixed traversal, `f64` relation products, scaled normalization, canonical
signed zero, deterministic excitation, and explicit tie laws require
sample-bit-identical output for the same request on one supported deterministic
target.

## Candidate Shape

The disposable candidate adds only:

`crates/signal-dsp-stretch/src/creative_excitation_relation/`

Use `mod.rs`, `plan.rs`, `analysis.rs`, `excitation.rs`, `synthesis.rs`, and
`tests.rs`. `lib.rs` may declare the module privately. Do not change production
tiers, public APIs, cache, artifact plans, reports, binaries, routes, or
dependencies. Candidate listening material stays ignored under `target/`.

## Fixed Admission

Failure stops the sequence.

### 1. Relation Proof

Before a renderer runs, exhaustively enumerate finite canonical coefficient
pairs from `{0,+0,-0,+1,-1,+i,-i,1+i,1-i,-1+i,-1-i,tiny,large}`. Cover mono,
duplicate, unequal, exact anti-phase, near-cancelled, and one-silent-channel
cases.

Pass requires within `1e-7` in `U` and `1e-6` after one synthesis frame:

- duplicate relations are identical
- channel swap swaps relations and output
- common polarity produces the contracted invariant branch
- exact whole-source anti-phase negates and swaps relations as frozen
- incidental exact-bin cancellation uses `o` and remains polarity-covariant
- silence stays exact zero
- no native angle or wrapped subtraction appears

Any miss closes the candidate before the full renderer structural gate.

### 2. Structural Gate

The first renderer row is the exact failed Batch 31.6 common-polarity case.
Then rerun every prior structural row at `4x`, `8x`, and `16x` across character
and macro boundaries.

Pass requires exact target length, finiteness, normalization coverage, exact
silence, repeated-output identity, seed effect, strict map monotonicity,
duration-independent state below `32 MiB`, no processing allocation,
full-complex excitation reconstruction within `1e-6`, and duplicate, swap,
polarity, anti-phase, delayed-stereo, and `space` mechanics within `1e-6`.

### 3. Crest Gate

The first creative row remains neutral `Dream`, `4x`, and the frozen
deterministic uniform-noise source. Then run uniform, Rademacher, and
amplitude-modulated noise, harmonic pads, and impulse trains for every ratio
and character.

Worst-channel RMS-matched crest-factor growth over active support must be at
most `6 dB`. Any miss closes the current diffusive owner. No limiter, repair,
distribution change, or rerun follows.

### 4. Remaining Synthetic Gate

Retain the frozen requirements:

- dominant pitch error at most `15` cents
- no non-finite, dropout, or exterior-click finding
- no secondary impulse lobe above `-12 dB` outside mapped smear
- no new stable periodicity above `-18 dB` on non-periodic noise
- higher `motion` never increases complete-coefficient correlation
- higher `detail` never widens the impulse envelope
- `Dream` is smoother than `Rough`
- `Spectral` has greater stable per-bin concentration than `Dream`
- every linked-stereo mechanics row passes

Impulse smear support is the cropped interval:

`[q*(e-N/2)-N/2,q*(e+N/2)+N/2]`.

### 5. Long-Form And Stereo Gates

Only after all synthetic gates pass, use the retained fifteen-row mono pack at
`4x`, `8x`, and `16x`, reviewing `8x` first under common-RMS / `0.95` peak
matching.

`Dream` must be preferred or tied with PaulXStretch on `12/15` rows with no
source family losing every ratio. `Spectral` must be recognizable on `12` and
useful on `10`. `Rough` must be recognizable on `12` and useful on `8`. No row
may be unusable. Listening is authority.

Then run the retained stereo pack. An independent eligible listener must
assess image stability, centre pull, width pumping, one-sided texture, and
channel echo. Missing independent review blocks promotion.

### 6. Minimal Admission

Only a complete pass may admit the private
`creative_excitation_relation` family, preserving structural and synthetic
tests, and one internal engine-version identifier. Public character, product,
cache, artifact, routing, pitch, dynamic-ratio, `Cloud`, and `Cyclic` surfaces
remain closed.

## Rejection And Cleanup

Any relation, structural, crest, synthetic, mono, or stereo miss rejects the
whole candidate. Record one dominant cause and stopped gate. Delete the
worktree, branch, module, tests, generated state, and listening assembly. No
candidate mechanism or harness surface moves to `main`.

This is the final candidate for the current diffusive owner. Any failure closes
that owner and triggers range-owner reassessment. It does not authorize another
relation, excitation, window, coefficient, smoothing, seed, or scalar variant.

## Sources

- [PaulXStretch official repository](https://github.com/essej/paulxstretch)
- [Moliner et al., Noise Morphing for Audio Time Stretching](https://www.pure.ed.ac.uk/ws/portalfiles/portal/428590250/2024_NoiseMorphing_SPL_Moliner.pdf)

## Next Task

Run `g10.031` Batch 31.13 only to freeze the separately selected
`SimilarityAlignedCyclic` brief. Do not repair this relation proof, reopen a
diffusive variant, tune `CyclicGrain`, or begin Cloud, routing, or product
implementation.
