# Offline Creative ContinuousExcitationSpectral Renderer Brief

Status: frozen; candidate implementation unopened
Owner: dsp
Updated: 2026-07-19
Contract: `085`
Roadmap: `g10.031`, Batch 31.5

## Decision

Build one Signal-owned `ContinuousExcitationSpectral` renderer. It replaces the
rejected independent-bin `DiffuseSpectral` topology for the fixed-ratio
`Dream`, `Spectral`, and `Rough` range.

The renderer is one long-window STFT system with:

- one sample-centred source/output map
- one transform grid and normalized overlap-add law
- native-channel spectral envelopes
- one bounded, continuous, output-synchronous stochastic excitation
- one linked instantaneous-frequency carrier retained only as a character mix
- no independently evolving per-bin random rotors

The decisive ownership change is waveform-level. The stochastic phase and its
realized bin magnitudes come from the transform of one continuous bounded
waveform. They are not generated independently per bin and are never reduced
to phase-only coefficients. This preserves cross-bin and cross-frame
relationships before source-envelope shaping.

This is a clean-room Signal architecture. The PaulXStretch repository defines
an audible target, not implementation authority. Moliner et al.'s noise
morphing paper supports continuous full-complex stochastic excitation as a
general technique; Signal owns the geometry, controls, linked-channel law,
boundaries, constants, validation, and Rust expression.

## Why The First Topology Closed

The first candidate passed structural controls but neutral `Dream` at `4x`
grew deterministic uniform-noise crest factor by `7.08 dB`, above the frozen
`6 dB` ceiling. Its independent per-bin phase diffusion destroyed the bounded
source waveform's cross-bin relationships. With many bins, that construction
tends toward a Gaussian-like time waveform. The failure is therefore
topological, not evidence for changing `rho`, phase mix, a window, or a
threshold.

The retained musical comparator pack confirms that the gate is correctly
placed:

| Comparator region | Rows over `6 dB` | Maximum growth |
| --- | ---: | ---: |
| PaulXStretch / `Dream` | `0/15` | `3.88 dB` |
| REAPER `Rrreeeaaa` / `Rough` | `0/15` | `4.37 dB` |
| REAPER `ReaReaRea` / `Cyclic` control | `0/15` | `1.30 dB` |
| CDP `SPECTSTR` / `Spectral` colour | `15/15` | `13.97 dB` |

The `6 dB` limit remains mandatory for every Signal character. `Spectral`
targets CDP's recognizable exposed colour, not its peak defect. There is no
limiter, compressor, output crest repair, post normalization, or gate waiver.

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

## Transform, Map, And Analysis

For sample rate `F`, freeze:

`N = clamp(next_power_of_two(ceil(0.30*F)),4096,65536)`

`S = N/4`

Every channel uses the same centred periodic square-root Hann:

`w[n]=sqrt(0.5-0.5*cos(2*pi*n/N))`, `0<=n<N`.

Synthesis centres are `y_k=k*S` for every signed integer `k` whose centred
window support intersects `[0,T)`. Pre-roll and post-roll frames keep their
signed indices.

The sole source map is:

`x_k=((y_k+0.5)*L/T)-0.5`.

At analysis index `n`, read `p=x_k+n-N/2` with linear interpolation and exact
zero outside `[0,L)`. Every channel uses the same `p`. No later stage moves the
source cursor, creates a second read, or owns another event timeline.

Window and transform every native channel. Retain native coefficient `X_c[b]`,
magnitude `M_c[b]`, and phase. Select the linked reference exactly as follows:

1. Set `R=sum_c X_c[b]` and `A=sum_c |X_c[b]|`.
2. If `|R|>=1e-6*A`, use `R`.
3. Otherwise use the coefficient with lexicographically greatest
   `(magnitude,real,imaginary)`.
4. Exact joint silence uses reference phase zero.

For active interior bins, native relation is
`delta_c=wrap(arg(X_c)-arg(R))`. Exactly silent native coefficients remain
zero. Selection is value-symmetric, so duplicate, swap, and common-polarity
transforms commute.

## Continuous Excitation Owner

First freeze one channel-symmetric source orientation. Scan source frames from
the start in `f64` and take the sign `o` of the first exactly non-zero channel
sum. If every channel sum is zero, use `o=+1`; exact anti-phase stereo is then
oriented by the linked-reference fallback and native relations. This bounded
preflight scan is invariant to channel swap and flips under common polarity
outside the exact anti-phase case.

Define one signed, random-access output waveform `r[t]` shared by all linked
channels. Reinterpret signed sample index `t` as two's-complement `u64` and set:

`x=seed xor rotate_left(u64(t),17) xor 0x4558434954415445`.

Apply the frozen Signal mixer with wrapping multiplication:

1. `x xor=x>>30`; multiply by `0xBF58476D1CE4E5B9`.
2. `x xor=x>>27`; multiply by `0x94D049BB133111EB`.
3. `x xor=x>>31`.

Set `r[t]` to `o` when the high bit is one and `-o` otherwise. No platform RNG,
entropy source, mutable global state, channel-local draw, or duration-sized
noise buffer exists.

For frame `k`, form:

`e_k[n]=w[n]*r[y_k+n-N/2]`.

Transform it once, shared across channels, to `E_k[b]`. Let
`sigma=sqrt(sum_n w[n]^2)` and retain the full complex coefficient:

`Q_k[b]=E_k[b]/sigma`.

Do not normalize `Q` to unit magnitude. Its realized magnitude and phase are
one indivisible waveform field. With a flat unit source envelope and a fully
stochastic character, inverse transform, synthesis window, and normalizer must
reconstruct `r[t]/sigma` over the crop. That exact bounded-excitation control
is the topology proof.

`Q` is absolute on the output lattice. It has no dormant state and is never
advanced from a previous spectral frame.

## Coherent Carrier

Each interior bin retains one linked carrier phase and instantaneous frequency.
Let `omega_b=2*pi*b/N` and `dx=x_k-x_(k-1)`. When current and preceding linked
reference coefficients are active:

`omega_hat=omega_b+wrap(phi_k-phi_(k-1)-omega_b*dx)/dx`.

Otherwise retain the last active `omega_hat`; a cold bin uses `omega_b`.
Advance `carrier=wrap(carrier+omega_hat*S)`. First activation or reactivation
after `N` dormant source samples initializes from current linked reference
phase.

The carrier is not a stochastic owner. It supplies the stable `Spectral`
contribution and pitch centre. There is no peak tracker, identity lock,
transient reset, event reassignment, or separate synthesis grid.

## Magnitude Owner

Use `e=1e-20`. For every native channel and bin:

`ell_c[b]=ln(max(M_c[b],e))`.

Create `smooth3(ell)` by applying `[1,2,1]/4` three times with replicated DC
and Nyquist edges. Then:

`f_c=(1-frequency_blend)*ell_c+frequency_blend*smooth3(ell_c)`

`V_c=alpha*V_c_prev+(1-alpha)*f_c`.

The first active frame initializes `V_c=f_c`. Candidate envelope is
`B_c=exp(V_c)`.

One linked power law bounds envelope smoothing before excitation:

`P_raw=sum_(c,b) M_c[b]^2`

`P_env=alpha*P_env_prev+(1-alpha)*P_raw`

`P_candidate=sum_(c,b) B_c[b]^2`

`g=clamp(sqrt(P_env/max(P_candidate,e^2)),0.25,4.0)`.

Set `A_c=g*B_c`. This gain belongs to source-envelope evolution. It is computed
before stochastic excitation and never responds to output peaks or realized
`Q` magnitudes. Exact joint silence emits exact-zero envelopes. Dormant source
envelopes decay toward `ln(e)` and become cold after `N` source samples.

## Character And Macro Laws

Let `m=motion` and `d=detail`. Freeze:

| Character | stochastic mix `p` | `frequency_blend` | `alpha` |
| --- | --- | --- | --- |
| `Dream` | `1.0` | `0.80*(1-d)` | `clamp(0.90-0.25*d-0.15*m,0.50,0.90)` |
| `Spectral` | `0.20+0.20*m` | `0.15*(1-d)` | `clamp(0.94-0.20*d-0.10*m,0.64,0.94)` |
| `Rough` | `1.0` | `0.10*(1-d)` | `0.45*(1-d)*(1-0.5*m)` |

For active interior bins:

`C_c=A_c*exp(i*(carrier+delta_c))`

`D_c=A_c*Q_k[b]*exp(i*delta_c)`

`Y_c=sqrt(1-p)*C_c+sqrt(p)*D_c`.

There is no complex-vector renormalization and no gain after this mix. `Dream`
and `Rough` differ through their frozen spectral and temporal envelope laws,
not a second random process. `Spectral` retains a coherent contribution. DC
and Nyquist use the corresponding real excitation and native real sign, then
are forced real. Interior negative bins are exact conjugates.

Increasing `motion` lowers magnitude correlation and never lowers the
stochastic mix. Increasing `detail` reduces spectral and temporal smoothing. Character
is represented internally by non-negative `(dream,spectral,rough)` weights
summing to one; interpolate `p`, `frequency_blend`, and `alpha` once before
frame zero. Public values select vertices. Intermediate weights exist only for
seam validation.

For linked stereo, apply `space=s` after the character mix. At frequency `f`:

`h=clamp((f-200)/(1000-200),0,1)`

Form mid and side, set `side'=side*(1+s*h)`, and reconstruct the channels.
Mono and duplicate stereo remain unchanged. Bass below `200 Hz` is unchanged.
Channel swap and common polarity commute. The maximum side gain is `2`.

## Synthesis, Boundaries, And Exact Length

For each frame and channel:

- construct the non-negative spectrum
- force DC and Nyquist real and mirror interior bins by conjugation
- inverse FFT with scale `1/N`
- multiply by the same square-root Hann
- add to a channel-local rolling overlap ring
- add `w[n]^2` to one shared normalization ring

Emit a sample only after no later window can touch its signed output index.
Divide by normalization above `1e-12`; active crop coverage at or below that
value is failure. Discard pre-roll and append exactly `[0,T)`. Flush post-roll
only far enough to prove coverage. Never resize-fill.

There is no wrap, reflection, hidden extension, fade, boundary repair, attack
layer, tail layer, crest repair, or second normalization. A non-finite value
stops the render.

## Transients And Replica Prevention

The renderer deliberately has no transient detector or reset state:

- long analysis and envelope evolution own commanded attack smear
- every synthesis frame has one mapped source read
- no attack is copied, reset, isolated, or mixed back
- no cyclic buffer or repeated grain exists
- excitation is continuous on the output lattice

`detail` only changes envelope smoothing. A periodic attack, doubled attack,
or stutter outside future `Cyclic` ownership is failure.

## Memory, Determinism, And Cost

Allocate all slabs before processing. State contains source analysis and
synthesis buffers for `C*N`, one shared excitation buffer of `N`, native
envelopes and relations for `C*(N/2+1)`, linked carrier state, the window,
channel overlap rings, one normalization ring, and FFT plans and scratch.

Checked working state must remain at or below `32 MiB` for `C<=2` and
`N<=65536`, excluding borrowed input and required output. Capacity is identical
for matched one-second and one-hour requests. No allocation occurs after frame
processing starts.

One preflight source scan establishes orientation. Every frame then performs
one source forward and one inverse FFT per channel plus one shared excitation
forward FFT and linear bin work. Frame count is linear in `T/S`; memory is
independent of duration. Fixed traversal, random-access excitation, symmetric
ties, and explicit float operations require
sample-bit-identical output for the same request on the same supported
deterministic target.

## Candidate Shape

The disposable candidate adds one private family only:

`crates/signal-dsp-stretch/src/creative_excitation/`

Use `mod.rs`, `plan.rs`, `analysis.rs`, `excitation.rs`, `synthesis.rs`, and
`tests.rs`. `lib.rs` may wire it privately in the isolated worktree. Do not
change production tiers, public APIs, cache identity, artifact plans, runtime
reports, binaries, or product routes.

Listening assembly stays ignored under `target/`. No hidden review API, report
mode, fixture family, feature flag, experimental module tree, or external
production dependency enters `main`.

## Fixed Admission

Failure stops the sequence.

### 1. Structural Gate

Run mono and linked stereo at `4x`, `8x`, and `16x` over empty, one-sample,
sub-window, exact-window, silence, impulses, tones, chords, deterministic
uniform and Rademacher noise, anti-phase stereo, delayed stereo, and mixed
inputs. Exercise every character and every macro boundary without a Cartesian
product.

Pass requires all prior `DiffuseSpectral` structural rows plus:

- the excitation is one signed output waveform shared across linked channels
- source orientation commutes with duplicate, swap, and common-polarity cases,
  including exact anti-phase fallback
- its frame coefficients equal the transform of the overlapping waveform
- full complex `Q`, including realized magnitude, reaches synthesis
- a flat-envelope stochastic reconstruction equals `r[t]/sigma` within
  `1e-6` over the active crop
- no independent per-bin random rotor, phase-only normalization, peak follower,
  limiter, compressor, crest repair, or post normalization exists
- exact `T`, finiteness, normalization coverage, exact silence, determinism,
  seed effect, duration-independent capacity, no processing allocation, linked
  transformations, `space`, and invalid-request controls pass

### 2. Crest Admission

The first rendered gate row is the prior failure: neutral `Dream`, `4x`, and
the same deterministic uniform-noise source and active-support law. Then run
uniform noise, Rademacher noise, amplitude-modulated noise, harmonic pads, and
impulse trains at every ratio and character vertex.

RMS-matched crest-factor growth versus the source must be at most `6 dB` over
each signal's active support. The bound is per channel and the row uses the
worst channel. Any miss rejects the whole candidate before other creative
metrics or listening.

### 3. Remaining Creative Synthetic Gate

Run the retained tones, chords, pads, impulses, impulse trains, silence gaps,
non-periodic noise, and linked-stereo relations at every ratio and character.

Pass requires:

- dominant tone and chord-partial pitch error at most `15` cents
- zero non-finite, dropout, or exterior-click findings
- no secondary impulse-envelope lobe above `-12 dB` outside mapped smear
- no new stable periodicity peak above `-18 dB` on non-periodic noise
- higher `motion` never increases complete-coefficient frame correlation
- higher `detail` never widens measured impulse envelope
- `Dream` has lower frame discontinuity and roughness than `Rough`
- `Spectral` has greater stable per-bin concentration than `Dream`
- every stereo mechanics row passes before listening

The mapped impulse smear support remains:

`[q*(e-N/2)-N/2, q*(e+N/2)+N/2]`, cropped to output.

### 4. Long-Form Mono Listening

Use the retained percussion, bass, vocal, pad/sustain, and full-mix pack at
`4x`, `8x`, and `16x`, with `8x` first. Keep the common-RMS / `0.95` peak
policy. Compare neutral `Dream` with PaulXStretch, `Spectral` with CDP
`SPECTSTR`, and `Rough` with REAPER `Rrreeeaaa`.

No row may be unusable. `Dream` must be preferred or tied on at least `12/15`
rows with no source family losing all ratios. `Spectral` must be recognizable
on at least `12` and useful on at least `10`. `Rough` must be recognizable on
at least `12` and useful on at least `8`. Neutral characters must remain
distinct. Listening, not aggregate metrics, decides usefulness.

### 5. Linked-Stereo Admission

After mono passes, run the retained structural transformations and long-form
stereo pack. An eligible listener independent of the operator must assess
image stability, centre pull, width pumping, one-sided texture, and channel
echo. The operator's one-ear hearing does not satisfy this gate. Missing review
blocks promotion.

### 6. Minimal Admission

Only a candidate passing every gate may admit the private
`creative_excitation` family, its fixed-ratio request and renderer, preserving
tests, and one engine-version identifier. Do not admit a public character enum,
product API, cache schema, artifact surface, runtime route, pitch path, dynamic
ratio, `Cloud`, or `Cyclic`.

## Rejection And Cleanup

Any gate miss rejects the whole candidate. Record one dominant cause and
stopped gate. Delete its worktree, branch, module, tests, local listening
assembly, and generated audio. No rejected coefficient, fixture, report mode,
or hidden API moves to `main`.

This is the second complete topology opportunity in the diffusive-owner lane.
Failure for uncontrolled stochastic crest growth closes the
`ContinuousExcitationSpectral` candidate and the current diffusive-owner
family. It triggers owner-boundary reassessment, not excitation distribution,
window, coefficient, smoothing, seed, or scalar sweeps.

## Sources

- [PaulXStretch official repository](https://github.com/essej/paulxstretch)
- [Moliner et al., Noise Morphing for Audio Time Stretching](https://www.pure.ed.ac.uk/ws/portalfiles/portal/428590250/2024_NoiseMorphing_SPL_Moliner.pdf)

## Next Task

Run Batch 31.6 only. Implement this exact private family once in a disposable
worktree. Run structural admission and the prior failing crest row first. Stop
and delete on failure. Do not produce long-form audio or open `Cloud`,
`Cyclic`, routing, cache, dynamic ratio, or product API work before the fixed
gates pass.
