# Offline Creative DiffuseSpectral Renderer Brief

Status: frozen historical brief; first candidate rejected
Owner: dsp
Updated: 2026-07-19
Contract: `085`
Roadmap: `g10.031`, Batch 31.3

## Decision

Build one Signal-owned `DiffuseSpectral` renderer. It is a fixed-ratio,
offline-only, long-window STFT synthesizer with:

- one sample-centred source/output map
- one transform grid and one overlap-add law
- native-channel magnitudes and one linked phase field
- correlated deterministic phase diffusion
- bounded log-magnitude evolution
- the same equations for `Dream`, `Spectral`, and `Rough`

The characters change frozen coefficients, not topology. `Dream` is the
PaulXStretch-centred default. `Spectral` exposes a stable vocoder-like region.
`Rough` exposes a less correlated, less smoothed novelty region. `Cloud` and
`Cyclic` are unsupported by this renderer and fail before rendering; their
later owners remain separate.

This brief is clean-room architecture. External tools define audible regions
only. Signal does not copy their source expression, constants, tables,
thresholds, masks, or control flow.

## Candidate Outcome

Batch 31.4 implemented this brief once in a disposable worktree. Its structural
controls passed. Creative synthetic admission stopped when neutral `Dream` at
`4x` produced `7.08 dB` deterministic-noise crest-factor growth against the
frozen `6 dB` limit. No limiter or crest-repair stage is authorized here, so
the candidate was rejected and deleted before long-form listening.

This file remains the exact record of the rejected topology. It is not
authority for a parameter sweep or second implementation. Batch 31.5 closed
independent-bin diffusion and replaced its implementation authority with
`offline-creative-continuous-excitation-spectral-brief.md`.

## Supported Request

The isolated candidate accepts:

- mono or linked stereo interleaved `f32`
- finite input at `8,000` through `192,000` Hz
- exact target frame count `T`
- effective output/input ratio `q=T/L` from `4.0` through `16.0`, inclusive
- `Dream`, `Spectral`, or `Rough`
- finite normalized `motion`, `detail`, and `space` in `[0,1]`
- one explicit `u64` seed

Empty input with `T=0` returns empty output. Every other zero-length,
non-finite, out-of-range, arithmetic-overflow, unsupported-channel, or
unsupported-character request fails before allocation. Values are rejected,
not clamped. Dynamic ratio, pitch composition, `Cloud`, `Cyclic`, cache,
artifact routing, RealtimePreview, and audio-thread execution are unsupported.

The input is borrowed. The required output buffer is not counted as working
state. All other duration-independent state is.

## Transform Geometry

For sample rate `F`, freeze:

`N = clamp(next_power_of_two(ceil(0.30*F)),4096,65536)`

`S = N/4`

`N` is the FFT and window length. `S` is the synthesis-centre hop. Every
channel uses the same centered periodic square-root Hann:

`w[n]=sqrt(0.5-0.5*cos(2*pi*n/N))`, `0<=n<N`.

There is one scale. No transient window, multiresolution union, peak mask,
adaptive FFT, post-render layer, or character-specific transform exists.

Synthesis centres are `y_k=k*S` for every signed integer `k` whose centered
window support intersects `[0,T)`. This includes deterministic pre-roll and
post-roll frames. The signed frame index is retained for seeding and ring
addressing; it is never rebased by input duration.

## Source Map And Analysis Sampling

For non-empty input length `L`, target `T`, and synthesis centre `y_k`, the sole
map is:

`x_k=((y_k+0.5)*L/T)-0.5`.

It maps output sample centres to source sample centres. `x_k` is strictly
increasing because `q<=16` and `S>=1024`. Event order cannot reverse.

For analysis-window index `n`, read source position:

`p=x_k+n-N/2`.

Use linear interpolation between `floor(p)` and `floor(p)+1`. Samples outside
`[0,L)` are exactly zero. Every channel reads the same `p`. No downstream
stage moves the source cursor, adds another read, or reconstructs a separate
event timeline.

The maximum map quantization error is zero in the declared fractional map;
linear interpolation owns its realization. Frame centres are not rounded to
integer source samples.

## Analysis And Linked Reference

Window and transform every native channel. For non-negative bin `b`, retain
native coefficient `X_c[b]`, magnitude `M_c[b]`, and phase.

The linked reference coefficient is symmetric in channel order:

1. Compute `R=sum_c X_c[b]` and `A=sum_c |X_c[b]|`.
2. If `|R|>=1e-6*A`, use `R`.
3. Otherwise use the native coefficient with lexicographically greatest
   `(magnitude,real,imaginary)`.
4. Exact joint silence uses reference phase zero.

The fallback depends on coefficient value, not channel index. Duplicate,
swap, and common-polarity transforms therefore commute with selection.

Per-channel relation is:

`delta_c=wrap(arg(X_c)-arg(R))`.

Exactly silent native coefficients remain zero. No mid/side analysis,
independent channel randomization, post-render image projection, or
channel-summed magnitude replaces native synthesis.

## Instantaneous Frequency Carrier

Each bin owns one linked carrier phase and last instantaneous frequency.
Let `omega_b=2*pi*b/N` and `dx=x_k-x_(k-1)`.

When the current and preceding reference coefficients are active:

`omega_hat=omega_b+wrap(phi_k-phi_(k-1)-omega_b*dx)/dx`.

Otherwise use the last active `omega_hat`; a cold bin uses `omega_b`. Advance:

`carrier=wrap(carrier+omega_hat*S)`.

On first activation or reactivation after cold dormancy, initialize `carrier`
from the current linked reference phase. DC and Nyquist remain real and retain
their current analysis sign. Interior negative-frequency bins are exact
conjugates.

This carrier preserves pitch centre and polyphonic motion. It has no peak
tracking, identity locking, transient reset, or event reassignment.

## Correlated Phase Diffusion

Each interior bin owns one unit complex diffusion state `z`. For frame `k` and
bin `b`, derive a counter-based unit phasor `u` from the explicit seed:

1. Reinterpret checked signed `i64 k` as two's-complement `u64`. Set `x` to
   `seed xor (u64(k)*0x9E3779B97F4A7C15) xor
   (u64(b)*0xD1B54A32D192ED03)` with wrapping multiplication.
2. Apply `x xor=x>>30`; multiply by `0xBF58476D1CE4E5B9`.
3. Apply `x xor=x>>27`; multiply by `0x94D049BB133111EB`.
4. Apply `x xor=x>>31`.
5. Convert the high 24 bits to
   `theta=2*pi*(bits+0.5)/2^24`; `u=exp(i*theta)`.

Update:

`z'=rho*z+sqrt(1-rho^2)*u`.

Normalize `z'`; if its magnitude is below `1e-12`, use `u`. A cold bin starts
from `u` for its activation frame.

Blend the coherent carrier and diffusion without interpolating wrapped angles:

`d=normalize((1-phase_mix)+phase_mix*z)`.

If the blend magnitude is below `1e-12`, retain the preceding `d`. The linked
base rotor is `exp(i*carrier)*d`.

Random access by `(seed,k,b)` makes results independent of traversal chunking.
No platform RNG, entropy source, mutable global seed, or channel-local draw is
allowed.

## Magnitude Evolution

Use magnitude floor `e=1e-20`. For each native channel and bin:

`ell_c[b]=ln(max(M_c[b],e))`.

Create `smooth3(ell)` by applying the symmetric kernel `[1,2,1]/4` three
times with replicated DC and Nyquist edges. Blend:

`f_c=(1-frequency_blend)*ell_c+frequency_blend*smooth3(ell_c)`.

The per-channel/bin temporal envelope is:

`E_c=alpha*E_c_prev+(1-alpha)*f_c`.

The first active frame initializes `E_c=f_c`. Candidate magnitude is
`exp(E_c)`. Character coefficients are shared across channels.

One linked power envelope prevents smoothing from inventing arbitrary level:

`P_raw=sum_(c,b) M_c[b]^2`

`P_env=alpha*P_env_prev+(1-alpha)*P_raw`

`P_candidate=sum_(c,b) exp(2*E_c[b])`

`g=clamp(sqrt(P_env/max(P_candidate,e^2)),0.25,4.0)`.

Multiply every native magnitude by the same `g`. No limiter, compressor,
per-channel gain, loudness match, or post-render normalization is permitted.
If `P_raw` and the preceding `P_env` are both exactly zero, emit exact-zero
magnitudes and keep `P_env` zero without evaluating `g`. Exact-zero input
therefore produces exact-zero output for every seed and character.

For a jointly inactive bin, advance its carrier and diffusion while its
magnitude envelope decays toward `ln(e)`. Accumulate dormant age in source
samples. At age `N`, set its magnitude state to `ln(e)` and mark it cold. A
later activation follows the cold initialization rules. This is the complete
dormant/reactivation owner.

## Character And Macro Laws

Let `m=motion` and `d=detail`. Freeze the endpoint coefficients:

| Character | `rho` | `phase_mix` | `frequency_blend` | `alpha` |
| --- | --- | --- | --- | --- |
| `Dream` | `0.97-0.27*m` | `1.0` | `0.80*(1-d)` | `clamp(0.90-0.25*d-0.15*m,0.50,0.90)` |
| `Spectral` | `0.995-0.095*m` | `0.20+0.20*m` | `0.15*(1-d)` | `clamp(0.94-0.20*d-0.10*m,0.64,0.94)` |
| `Rough` | `0.35*(1-m)` | `1.0` | `0.10*(1-d)` | `0.45*(1-d)*(1-0.5*m)` |

Increasing `motion` lowers temporal phase and magnitude correlation. Increasing
`detail` reduces spectral and temporal smoothing. The directions never reverse.

The engine represents character internally as non-negative
`(dream,spectral,rough)` weights summing to one and linearly combines the four
endpoint coefficients before frame zero. Public character values select the
three vertices. Intermediate weights exist only for seam validation until a
consumer-facing morph is separately contracted. They do not add renderers.

The final native coefficient is evolved magnitude times the linked base rotor
times `exp(i*delta_c)`.

`space=s` then widens linked stereo inside spectral synthesis. For bin
frequency `f`:

`h=clamp((f-200)/(1000-200),0,1)`

Form `mid=(Y_left+Y_right)/2` and `side=(Y_left-Y_right)/2`, then set:

`side'=side*(1+s*h)`.

Reconstruct left and right from `mid` and `side'` before conjugate mirroring.
Mono and duplicate stereo remain unchanged for every `space`; bass relation
below `200 Hz` is unchanged. Channel swap and common polarity commute. The
maximum side gain is `2`. `space` has no independent random state and no
post-render image stage.

## Synthesis, Normalization, And Exact Length

For each frame and channel:

- construct non-negative bins from evolved magnitude and final phase
- force DC and Nyquist real
- mirror interior bins by conjugation
- inverse FFT with scale `1/N`
- multiply by the same square-root Hann
- add to one channel-local rolling overlap ring
- add `w[n]^2` to one shared normalization ring

Emit a sample only after no later synthesis window can touch its signed output
index. Divide by normalization above `1e-12`; an active crop sample at or below
that value is structural failure. Discard emitted pre-roll, append exactly
indices `[0,T)` to the required output, and flush deterministic post-roll only
far enough to prove coverage. Never resize-fill the result.

There is no wrap, reflection, hidden extension, fade, boundary repair, attack
layer, tail layer, crest repair, or second normalization pass. A non-finite
coefficient, accumulator, normalization value, or output sample stops the
render with failure.

## Transients And Replica Prevention

The renderer has no transient detector or transient state machine. That is a
frozen creative choice:

- the long window and magnitude envelope intentionally smear attacks
- every synthesis frame has one mapped source read
- no attack is copied, reset, reassigned, isolated, or mixed back
- no cyclic buffer or repeated grain exists
- correlated diffusion changes phase, not source position

`detail` reduces smoothing but never opens another source read or phase-reset
path. Periodic attacks, doubled attacks, or stutter outside a future explicit
`Cyclic` owner therefore remain failures, not creative licence.

## Memory, Determinism, And Cost

Allocate every slab before frame processing. Fixed state contains:

- `C*N` analysis complex samples
- `C*N` synthesis complex samples
- native magnitude, log-envelope, and phase-relation arrays for `C*(N/2+1)`
- linked reference phase, carrier, instantaneous-frequency, diffusion,
  dormant-age, and power state for `N/2+1`
- the `N`-sample window
- one `N+S` overlap ring per channel and one shared normalization ring
- FFT plans and scratch reported by those plans

Checked allocation must remain at or below `32 MiB` for `C<=2` and
`N<=65536`, excluding only borrowed input and required output. Exceeding the
cap fails before rendering. Capacities are identical for matched one-second
and one-hour requests.

Each output frame performs one forward and one inverse `N`-point FFT per
channel plus linear bin work. Frame count is linear in `T/S`; cost therefore
scales with output duration and `N log N`. No duration-growing analysis cache,
frame list, event list, magnitude history, or normalization buffer is allowed.

Fixed traversal, counter-based diffusion, explicit float operations, symmetric
tie rules, and preallocated state require sample-bit-identical output for the
same complete request on the same supported deterministic target.

## Candidate File And Ownership Shape

The disposable candidate adds one private module family only:

`crates/signal-dsp-stretch/src/creative_diffuse/`

Use `mod.rs`, `plan.rs`, `analysis.rs`, `phase.rs`, `synthesis.rs`, and
`tests.rs`. They own the private request and validation, planner, analysis,
character/magnitude state, linked phase field, rolling synthesizer, and tests,
respectively. Do not create alternate candidate files or generalize existing
production modules. `lib.rs` may wire the family privately in the candidate
worktree. It must not change `StretchQuality`, `StretchBackendTier`,
`TimeStretcher`, `OfflineHighQualityStretcher`, cache identity, artifact plans,
runtime reports, binaries, or product routes.

Candidate-only listening assembly stays outside `main` under ignored
`target/`. No hidden review API, report mode, fixture family, experimental
module tree, feature flag, or external production dependency is allowed.

## Fixed Admission

Failure stops the sequence.

### 1. Structural Gate

Run mono and linked stereo at `4x`, `8x`, and `16x` over empty, one-sample,
sub-window, exact-window, silence, impulses at interior and exterior phases,
tones, chords, deterministic noise, anti-phase stereo, delayed stereo, and
mixed inputs. Exercise every character at macro values `0`, `0.5`, and `1`
without taking their Cartesian product; each boundary value appears at least
once per ratio.

Pass requires:

- exact `T` frames without resize fill
- finite state and output
- strictly increasing `x_k` and exact shared source positions across channels
- complete crop normalization coverage
- exact-zero output for exact-zero input
- byte-identical repeated output for every seed case
- different seeds change at least one active interior sample
- renderer-owned allocation at or below `32 MiB` with identical capacity for
  matched short and long renders
- no allocation after processing begins
- duplicate stereo equals duplicated mono, channel swap swaps output, and
  common polarity flips output within `1e-6`
- neutral `space` preserves calibrated interchannel phase and level within
  `1e-6`; width is non-decreasing at `space` values `0`, `0.5`, and `1`
- unsupported ratios, controls, channels, sample rates, and characters fail
  before output

### 2. Creative Synthetic Gate

Use isolated tones, chords, harmonic pads, impulses, impulse trains,
amplitude-modulated noise, silence gaps, and linked-stereo relations at every
ratio and character vertex.

Pass requires:

- dominant tone and chord-partial pitch error at most `15` cents
- zero non-finite, dropout, or exterior-click findings
- RMS-matched crest-factor growth versus the source at most `6 dB`, measured
  over each signal's active support
- no secondary impulse-envelope lobe above `-12 dB` outside the allowed mapped
  smear support
- no new stable periodicity peak above `-18 dB` on non-periodic noise
- higher `motion` never increases measured phase correlation
- higher `detail` never widens the measured impulse envelope
- `Dream` has lower frame-discontinuity and roughness measures than `Rough`
- `Spectral` has greater stable per-bin concentration than `Dream`
- every stereo mechanics row passes before listening

Metrics diagnose the requested direction and reject structural inversions.
They do not decide musical usefulness.

For an impulse at source sample `e`, the allowed smear support is the crop of:

`[q*(e-N/2)-N/2, q*(e+N/2)+N/2]`.

The replica check excludes that interval and tests the remaining output. It
does not misclassify the intended long-window cloud around the source event.

### 3. Long-Form Mono Listening

Use the retained percussion, bass, vocal, pad/sustain, and full-mix sources at
`4x`, `8x`, and `16x`. Level-match under the retained common-RMS / `0.95` peak
policy. Review `8x` first, then the adjacent ratios.

Compare:

- neutral `Dream` against PaulXStretch
- neutral `Spectral` against CDP `SPECTSTR`
- neutral `Rough` against REAPER `Rrreeeaaa`
- macro extremes against their neutral character

Pass requires no unusable candidate row. `Dream` must be preferred or tied
against PaulXStretch on at least `12` of `15` rows, with no source family losing
all three ratios. `Spectral` must be judged recognizably in its intended region
on at least `12` rows and musically useful on at least `10`. `Rough` must be
recognizable on at least `12` and useful on at least `8`; novelty is allowed,
uncommanded cyclic repetition is not. No neutral character may leak another
anchor strongly enough to defeat its label.

Listening remains authority. Aggregate objective wins cannot compensate for a
failed character or unusable row.

### 4. Linked-Stereo Admission

After mono passes, run structural stereo transformations and the retained
long-form stereo pack. An eligible listener independent of the operator must
assess image stability, centre pull, width pumping, one-sided texture, and
channel echo. The operator's one-ear hearing does not satisfy this gate.
Missing review blocks promotion; it never waives stereo.

### 5. Minimal Admission

Only a candidate passing every prior gate may admit:

- the single internal `creative_diffuse` module
- its private fixed-ratio request and renderer
- structural and synthetic tests required to preserve admitted behavior
- one frozen engine-version identifier for later cache work

Do not admit a public character enum, product API, cache schema, artifact
surface, runtime route, pitch path, dynamic ratio, `Cloud`, or `Cyclic`.
Product-facing work remains a later Contract `085` batch.

## Rejection And Cleanup

Any structural, synthetic, mono-listening, or stereo-listening miss rejects the
whole candidate. Record one dominant cause and stopped gate. Delete the
candidate worktree and branch, including its module, tests, local listening
assembly, and generated audio. No rejected constant, profile, fixture, report
mode, or hidden API moves to `main`.

Two complete `DiffuseSpectral` candidates failing for the same dominant cause
trigger architecture reassessment. They do not authorize window, coefficient,
phase, smoothing, seed, or scalar sweeps.

## Next Task

Use `offline-creative-continuous-excitation-spectral-brief.md` as the sole
current candidate authority. Do not tune or reimplement this historical brief.
