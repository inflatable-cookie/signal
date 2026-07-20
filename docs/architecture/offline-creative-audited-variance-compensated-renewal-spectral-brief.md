# Offline Creative AuditedVarianceCompensatedRenewalSpectral Renderer Brief

Status: frozen; candidate admitted to concealed mono listening
Owner: dsp
Updated: 2026-07-20
Contract: `085`
Roadmap: `g10.031`, Batch 31.24

## Decision

Build one fresh Signal-owned `AuditedVarianceCompensatedRenewalSpectral`
candidate for neutral `Dream` at `4x`, `8x`, and `16x`.

Batch 31.23 produced no valid synthetic or listening result. Its renderer
topology therefore remains untested. This candidate retains the source-backed
DSP architecture and replaces the invalid evidence construction:

- one exact sample-centred source/output map
- one sample-rate-normalized long magnitude analysis
- deterministic stochastic phase renewal per output frame
- adjacent-frame raised-cosine blending
- Signal-derived overlap-variance compensation
- one linked mid/side law
- deterministic exterior support and exact target crop
- one compile-linked owner for every structural and synthetic gate

Do not recover source, tests, helper code, or build state from the deleted
Batch 31.23 candidate or system Trash. Implement this brief cleanly.

No instantaneous-frequency carrier, phase propagation, magnitude recurrence,
transient detector, onset reset, component layer, limiter, compressor, or
post-render gain repair is present.

## Supported Request

The private candidate accepts `CandidateRequest<'a>` with exactly:

- `input: &'a [f32]`: finite mono or linked-stereo interleaved samples
- `channels: usize`: `1` or `2`
- `sample_rate: u32`: `8,000` through `192,000` Hz
- `target_frames: usize`: exact output frame count
- `seed: u64`: explicit variation identity
- `space: f32`: finite value in `[0,1]`

Let source frame count be `L` and target frame count be `T`. Require
`4*L<=T<=16*L`, with checked multiplication. `L` and `T` may not exceed
`2^53-1`. Empty input with `T=0` returns empty. Every other empty, zero-target,
overflowed, non-finite, unsupported-channel, unsupported-rate, compression, or
range-miss request fails before output allocation. Values are rejected, not
clamped.

`CandidateError` is one closed private enum covering request, size,
allocation-bound, and non-finite-processing failure. The only candidate entry
is:

`render(CandidateRequest<'_>) -> Result<Vec<f32>, CandidateError>`.

The candidate owns fixed-ratio neutral `Dream` only. `motion`, `detail`, pitch
composition, dynamic ratio, reverse, other characters, cache, artifact
routing, RealtimePreview, audio-thread execution, and public product exposure
remain unsupported.

## Transform, Window, And Map

Use one transform length:

`N=clamp(nearest_power_of_two(round(2*F/3)),8192,131072)`.

Integer rounding of `2*F/3` selects the nearest integer with half upward.
Power-of-two distance ties select the larger power. Output block length is
`H=N/2`. At `44.1` and `48` kHz, `N=32768` and `H=16384`.

Use one periodic Hann analysis window:

`w[n]=0.5-0.5*cos(2*pi*n/N)`, `0<=n<N`.

Energy calibration is:

`G=sqrt(N/sum_n(w[n]^2))`.

Output block `j` begins at `y_j=j*H`. Analysis frame `j` owns source centre:

`x_j=((y_j+0.5)*L/T)-0.5`.

Evaluate `(2*y_j+1)*L` over denominator `2*T` with checked `u128`, convert
once to `f64`, then subtract `0.5`. No other cursor exists.

For analysis index `n`, read `p=x_j+n-(N-1)/2`. Let `i=floor(p)` and `u=p-i`.
Use four-point cubic Lagrange interpolation over `i-1` through `i+2`:

- `c_-1=-u*(u-1)*(u-2)/6`
- `c_0=(u+1)*(u-1)*(u-2)/2`
- `c_1=-(u+1)*u*(u-2)/2`
- `c_2=(u+1)*u*(u-1)/6`

Samples outside `[0,L)` are exact zero. A `FrameSchedule` computes each
`x_j` once and passes it to every linked component. Analysis cannot derive or
advance a source position.

## Magnitude Analysis And Phase Renewal

Mono has one native component. Stereo uses orthonormal components:

- `M=(L+R)/sqrt(2)`
- `D=(L-R)/sqrt(2)`

Interpolate, window, and transform each component. Analysis returns only
non-negative-bin magnitudes `A_r[j,b]`; its type owns no input phase.

Before frame processing, scan each component from source start. Canonicalize
signed zero. Orientation `o_r` is the sign of the first exactly non-zero
sample, or `+1` for exact silence. Retain the exact-silence flag.

Define `mix64(z)` with wrapping `u64` arithmetic:

1. `z=(z xor (z>>30))*0xBF58476D1CE4E5B9`
2. `z=(z xor (z>>27))*0x94D049BB133111EB`
3. return `z xor (z>>31)`

For frame `j`, bin `b`, and stream tag `s`:

`z=mix64(seed xor mix64(j xor FRAME) xor rotate_left(mix64(b xor BIN),21) xor s)`.

Little-endian ASCII tags are `RNWFRAME`, `RNWBIN00`, `RNWBASE0`, and
`RNWSIDE0`. Convert the high 53 bits to `u=(z>>11)/2^53`, then
`phase(z)=2*pi*u-pi`.

Mono and mid use base phase `theta[j,b]`:

`Y_r[j,b]=o_r*A_r[j,b]*exp(i*theta[j,b])`.

For side, let `f_b=b*F/N`, `t=clamp((f_b-250)/(1500-250),0,1)`, and
`h=t*t*(3-2*t)`. With side-stream phase `zeta[j,b]`:

`Y_D[j,b]=o_D*A_D[j,b]*exp(i*(theta[j,b]+space*h*zeta[j,b]))`.

DC is real signed magnitude. Nyquist is exact zero. Negative bins are
conjugate mirrors. Exact-silent components remain exact zero. There is no
phase, dormancy, tonal-peak, or reactivation state.

## Frame Blend, Decode, And Boundaries

Inverse-transform with scale `1/N`. Let the real frame be `z_r[j,n]`. Apply no
synthesis window.

For block offset `0<=n<H`:

- `u=(n+0.5)/H`
- `a=0.5+0.5*cos(pi*u)`
- `b=1-a`
- `c=1/sqrt(a*a+b*b)`
- `q_r[j,n]=G*c*(a*z_r[j,H+n]+b*z_r[j+1,n])`

Decode stereo after blending:

- `L=(q_M+q_D)/sqrt(2)`
- `R=(q_M-q_D)/sqrt(2)`

Cache exactly two adjacent synthesized frames per component. Frame `j+1`
becomes frame `j` for the next block. No mutable random state, frame-local
gain, overlap denominator, peak follower, limiter, compressor, or output
normalization exists.

Exterior envelope length is `E=min(H,floor(T/4))`. If `E>=2`, multiply output
frame `y` by both applicable factors:

- head: `sin((pi/2)*y/(E-1))` for `y<E`
- tail: `sin((pi/2)*(T-1-y)/(E-1))` for `y>=T-E`

Otherwise the factor is `1`. Emit exactly `[0,T)`. Never append a tail,
resize-fill, wrap, reflect, or run a repair pass. Any non-finite coefficient,
accumulator, compensation value, or output sample fails the render.

There is deliberately no transient state machine. One mapped long-magnitude
read owns attack smear. Every synthesis frame has one analysis read. A broad
attack is expected. A separated secondary attack, periodic replica, click,
stutter, or static freeze is failure.

## Memory, Allocation, Determinism, And Cost

Allocate output, window, interpolation buffer, component spectrum, two inverse
frames per component, FFT plans, and FFT scratch before the first analysis
frame. Reuse them for the whole render.

Input and required output storage do not count as working state. Every other
allocation, including FFT-plan heap, does. Actual peak working state must be at
most `32 MiB` for stereo at `N<=131072` and independent of source or target
duration.

`tests.rs` owns one candidate-only counting global allocator. The memory owner:

- enables counting immediately before candidate construction
- records allocation sizes, current live bytes, peak live bytes, and
  allocation/reallocation calls after processing starts
- subtracts the returned output vector's exact capacity in bytes
- includes FFT plans and scratch in the remaining peak
- asserts zero allocation or reallocation after the first frame marker
- compares the pure working-capacity plan at one second and one hour

One `#[cfg(test)]` marker in `synthesis.rs` sets processing-start state directly
before the first analysis frame. It exposes no API and performs no allocation.

Mono performs one forward and one inverse FFT per new frame. Stereo performs
two of each. Work is `O((T/H)*N*log N)`; memory is duration-independent.
Counter-addressed phase, fixed traversal, explicit ties, and no parallel
reduction require byte-identical output for the same request on one supported
deterministic target.

The renderer is offline only. It never executes or allocates on the audio
thread.

## Candidate Isolation

Use exactly:

- worktree: `signal-candidate-31-25`
- branch: `candidate/g10-031-audited-variance-renewal`
- module: `crates/signal-dsp-stretch/src/creative_audited_variance_renewal/`

Module files are `mod.rs`, `plan.rs`, `analysis.rs`, `phase.rs`,
`synthesis.rs`, and `tests.rs`. The isolated `lib.rs` may declare the module
privately.

Ownership:

- `mod.rs`: request, error, render entry
- `plan.rs`: validation, transform, one frame schedule, orientation, counts,
  capacity plan
- `analysis.rs`: component reads, interpolation, window, magnitude-only FFT
- `phase.rs`: counter addressing, renewal, Hermitian completion, DC, Nyquist,
  silence
- `synthesis.rs`: preallocated workspace, inverse frames, blend, decode,
  envelope, crop, finiteness, test-only processing marker
- `tests.rs`: allocator, coverage manifest, sources, metrics, structural and
  synthetic owners

No public API, production tier, cache identity, artifact plan, report, binary,
fixture, feature flag, dependency, or product route changes. Listening
assembly stays ignored under `target/`.

## Evidence Construction

Test names use only:

- `audited_variance_renewal_construction_`
- `audited_variance_renewal_structural_`
- `audited_variance_renewal_synthetic_`

No required test is ignored. Every accumulator and collection has a concrete
type at declaration.

`tests.rs` defines `GateId` and a constant `GATE_OWNERS` table. Each entry
contains one `GateId`, the exact owner name below, and a function pointer to
that test function. The construction test asserts:

- exactly `22` gate entries
- every `GateId` occurs once
- every frozen owner name occurs once
- every owner function pointer is non-null
- structural owner count is `13`
- synthetic owner count is `9`

Removing or renaming an owner breaks compilation through the function pointer.
Duplicate or missing IDs fail the construction test. The manifest does not
replace the assertions inside each owner.

Construction sequence:

1. `effigy test compile`
2. `effigy test cargo-nextest audited_variance_renewal_construction_`
3. require exactly one selected test and one pass
4. create one local checkpoint commit and record its hash

Compiler or construction diagnostics may be repaired before the clean receipt
only when edits change types, imports, visibility, ownership plumbing, or
manifest assembly without changing a DSP formula, constant, request decision,
source, measurement, threshold, or assertion. Any semantic change requires a
new brief.

After the checkpoint, source, tests, assertions, and manifest are frozen. No
repair or rerun follows an admission miss. The branch is never pushed.

## Frozen Gate Ownership

| ID | Exact test owner | Required ownership |
| --- | --- | --- |
| S01 | `audited_variance_renewal_structural_request_preallocation` | every valid and invalid request decision; invalid paths allocate nothing |
| S02 | `audited_variance_renewal_structural_transform_map` | `N`, `H`, checked `x_j`, strict monotonicity, block and frame counts |
| S03 | `audited_variance_renewal_structural_window_interpolation_gain` | periodic Hann, cubic coefficients, exterior zero reads, `G` |
| S04 | `audited_variance_renewal_structural_phase_spectrum` | tags, `mix64`, one address, orientation, silence, DC, Nyquist, Hermitian law |
| S05 | `audited_variance_renewal_structural_blend_envelope_crop` | `a`, `b`, `c`, frame halves, decode, envelope, endpoints, exact crop |
| S06 | `audited_variance_renewal_structural_edge_source_matrix` | empty, one-sample, sub-window, exact-window, silence, DC, impulses, tones, chords, noise at all ratios |
| S07 | `audited_variance_renewal_structural_determinism_seed` | byte repeat, changed-seed active difference, finite output |
| S08 | `audited_variance_renewal_structural_single_timeline` | one schedule call per frame, shared linked centre, no second cursor or source read |
| S09 | `audited_variance_renewal_structural_allocation_memory` | actual `32 MiB` peak, duration-independent plan, all allocations before frame one |
| S10 | `audited_variance_renewal_structural_linked_stereo` | duplicate/mono, swap, common polarity, anti-phase, delay, mixed relations |
| S11 | `audited_variance_renewal_structural_space_invariants` | `0`, `0.5`, `1`; silence, duplicate, length, total M/D spectral power |
| S12 | `audited_variance_renewal_structural_sustained_support` | every non-zero sustained row, all ratios, non-zero energy, no exact-zero `H` block |
| S13 | `audited_variance_renewal_structural_forbidden_mechanisms` | source-token and type scan for forbidden carriers, recurrence, detectors, layers, limiters, post-gain |
| Y01 | `audited_variance_renewal_synthetic_crest_reference` | all five crest rows at all ratios |
| Y02 | `audited_variance_renewal_synthetic_pitch_reference` | low tone, mid tone, every chord partial at all ratios |
| Y03 | `audited_variance_renewal_synthetic_impulse_distribution` | shortest 95% interval and exact-map centroid at all ratios |
| Y04 | `audited_variance_renewal_synthetic_replica_regions` | impulse and impulse-train region count and typed optional secondary peak |
| Y05 | `audited_variance_renewal_synthetic_noise_periodicity` | every integer lag from 20 ms through 1 s |
| Y06 | `audited_variance_renewal_synthetic_rms_modulation` | uniform-noise and mid-tone block-RMS CV |
| Y07 | `audited_variance_renewal_synthetic_silence_gap` | mapped gap RMS against full mapped active support |
| Y08 | `audited_variance_renewal_synthetic_integrity_discontinuity` | every mono row; length, finiteness, dropout, first-difference crest |
| Y09 | `audited_variance_renewal_synthetic_linked_stereo_inventory` | all five stereo relations, all ratios, frozen `space` schedule |

Structural admission command:

`effigy test cargo-nextest audited_variance_renewal_structural_`

Require exactly `13` selected tests and `13` passes.

Synthetic admission command:

`effigy test cargo-nextest audited_variance_renewal_synthetic_`

Require exactly `9` selected tests and `9` passes. Every synthetic owner
renders and records all of its rows before one final assertion. A failed row
does not short-circuit later rows inside that owner.

## Frozen Synthetic Sources

Use `F=48000`, `L=96000`, and `T` of `4L`, `8L`, and `16L`. Sustained support
is `24000<=n<72000`. Entrance frames `24000..26047` use
`0.5-0.5*cos(pi*(n-24000)/2047)`. Exit frames `69952..71999` use
`0.5-0.5*cos(pi*(71999-n)/2047)`. Other supported samples have weight `1`;
samples outside support are zero.

Mono sources:

- low tone: `0.5*sin(2*pi*110*n/F)`
- mid tone: `0.5*sin(2*pi*440*n/F)`
- chord: amplitude `0.1` at `110`, `164.813778`, `220`, `277.182631`, and
  `329.627557` Hz
- harmonic pad: partials `k=1..8` at `110*k` Hz with amplitude `0.35/k`
- impulse: `1` at `n=48000`
- impulse train: `1`, `-0.8`, `0.65`, `-0.5` at `19200`, `38937`, `58103`,
  `77797`
- silence gap: harmonic pad with exact zero at `42000<=n<54000`
- uniform, Rademacher, and amplitude-modulated noise below

Noise tag is little-endian `RNWTEST0`. For active `n`, set
`r=mix64(n xor TEST)`:

- uniform: `0.5*(2*((r>>11)/2^53)-1)`
- Rademacher: `+0.5` for high bit one, otherwise `-0.5`
- amplitude-modulated: Rademacher times
  `0.5+0.375*sin(2*pi*1.7*n/F)`

Stereo relations:

- duplicate: identical mid tone
- common polarity: duplicate and its exact two-channel negation
- anti-phase: mid tone left, exact negation right
- delay: harmonic pad left, zero-padded `37`-sample delay right
- mixed: chord plus `0.2` uniform noise left; delayed chord minus the same
  `0.2` noise right

`Y09` renders duplicate at all three `space` values and all ratios. It renders
common polarity, swap, and mixed at `space=0.5`; anti-phase at `space=1`; and
delay at `space=0` and `1`, all at every ratio.

## Exact Measurements

Crest factor is `20*log10(max(abs(x))/rms(x))`. Crest growth subtracts source
crest. Use mapped support `[ratio*24000,ratio*72000)` except impulse train,
which uses `[ratio*19200,ratio*77798)`. Candidate must not exceed the matching
PaulX row plus `2 dB`.

| Crest row | `4x` | `8x` | `16x` |
| --- | ---: | ---: | ---: |
| uniform noise | 9.931823324 | 11.898668127 | 10.432303004 |
| Rademacher noise | 14.809189700 | 15.525287997 | 15.710575458 |
| amplitude-modulated noise | 13.019591155 | 12.553111650 | 14.090964151 |
| harmonic pad | 6.348431050 | 6.745264841 | 6.313515214 |
| impulse train | -27.569237149 | -27.301419353 | -26.883996048 |

Pitch uses the central half of mapped active support. Apply a periodic Hann,
zero-pad to the next power of two at least eight times the measured length,
search expected frequency plus or minus `4 Hz`, choose the maximum magnitude,
and apply three-bin log-magnitude parabolic interpolation. Candidate absolute
error must not exceed the matching PaulX row plus `2` cents.

| Pitch row | `4x` | `8x` | `16x` |
| --- | ---: | ---: | ---: |
| 110 Hz | 7.410431632 | 8.816034233 | 7.666572548 |
| 440 Hz | 2.461974128 | 5.373507937 | 4.838384698 |
| chord max partial | 7.976134703 | 9.331375778 | 13.456683592 |

Impulse width is the shortest inclusive interval whose `f64` squared-energy
sum is at least `95%` of total. Two-pointer ties choose the earliest interval.
Candidate width must be `0.75` through `1.50` times reference.

Impulse centroid uses `f64` squared energy and expected mapped event
`(48000.5)*ratio-0.5`. Candidate absolute error must not exceed reference plus
`10%` of reference width.

| Impulse row | `4x` | `8x` | `16x` |
| --- | ---: | ---: | ---: |
| shortest 95% width | 79,469 | 155,953 | 309,239 |
| centroid error | 49,188.649257221 | 114,695.538853499 | 246,065.455355601 |

Replica envelope uses `480`-frame RMS windows at starts `0,240,...` through
the last complete window. Active windows are at least `-30 dB` relative to the
global envelope peak. Start a new region only when the distance from the last
active window start is at least `2400` frames. The region containing the
global peak is primary. Secondary peak is the largest other region peak in dB
relative to primary, represented as explicit `Option<f64>`.

PaulX has one region and `None` secondary peak for both impulse and impulse
train at all ratios. Candidate must also have one region and `None`; the
`+3 dB` secondary comparison is therefore vacuous, not missing.

Noise periodicity subtracts the active-support global mean, computes linear
autocorrelation by zero-padding to the next power of two at least `2*M-1`, and
divides every lag by lag-zero energy. Evaluate every integer lag `960..=48000`.
Candidate maximum must not exceed reference plus `0.05`.

Block-RMS CV uses `2400`-frame windows, `1200`-frame hop, and population
standard deviation over mapped active support. Candidate maximum must not
exceed reference plus `0.05`.

Gap RMS uses `[ratio*42000,ratio*54000)` divided by RMS over
`[ratio*24000,ratio*72000)`, expressed in dB. Candidate must not exceed
reference plus `3 dB`.

| Metric | `4x` | `8x` | `16x` |
| --- | ---: | ---: | ---: |
| uniform-noise autocorrelation | 0.017218163 | 0.017727693 | 0.017090511 |
| uniform-noise RMS CV | 0.387747959 | 0.460013282 | 0.492971808 |
| mid-tone RMS CV | 0.617268653 | 0.679139581 | 0.708639858 |
| silence-gap relative dB | 2.565308664 | 2.752688694 | 3.061967868 |

Interior discontinuity uses first-difference crest factor over the same mapped
support as crest; impulse uses the complete output. Candidate must not exceed
the matching PaulX row plus `3 dB`. This is a comparator-relative spike and
click control, not a claim that discrete audio is mathematically continuous.

| Difference crest row | `4x` | `8x` | `16x` |
| --- | ---: | ---: | ---: |
| low tone | 9.905726 | 10.894208 | 10.519063 |
| mid tone | 9.556341 | 10.436868 | 10.905863 |
| chord | 11.802457 | 12.936199 | 12.552822 |
| harmonic pad | 11.915677 | 13.552276 | 14.040544 |
| impulse | 21.672820 | 21.668892 | 21.489501 |
| impulse train | 16.312956 | 16.540906 | 17.186745 |
| silence gap | 13.453803 | 15.084147 | 15.790229 |
| uniform noise | 14.905336 | 15.539239 | 15.680264 |
| Rademacher noise | 14.348783 | 15.440176 | 15.456703 |
| amplitude-modulated noise | 16.083822 | 16.292147 | 16.221062 |

`Y08` also requires exact `T`, finite samples, and no exact-zero run of length
`H` inside mapped non-zero support for every mono source and ratio.

## Structural Pass

The 13 structural owners jointly require:

- every rejection before output allocation
- exact length, finite output, exact-zero silence
- byte-identical repeats and active changed-seed difference
- exact transform, map, interpolation, phase, blend, envelope, and crop laws
- one strictly increasing schedule and no second timeline
- actual working state at most `32 MiB`, duration-independent, with no
  allocation or reallocation after processing starts
- duplicate stereo equals duplicated mono within `1e-6`
- swap swaps, common polarity negates, anti-phase remains anti-phase within
  `1e-6`
- `space` preserves silence, duplicate stereo, total frame M/D spectral power,
  and length
- exact-zero first and last samples for non-empty `T>=8`
- non-zero interior energy and no exact-zero `H` block on every sustained row
- no forbidden carrier, recurrence, detector, layer, limiter, or post-gain

Any miss rejects the candidate before synthetic execution.

## Long-Form Mono Gate

Only after all nine synthetic owners pass, render the retained mono
percussion, bass, vocals, pads/sustains, and full-mix sources at `4x`, `8x`,
and `16x`. Review all five `8x` rows first.

Compare candidate mono with the left channel of retained PaulXStretch 1.6.0
default / FFT `16384`. Conceal A/B identity. For each row, source, A, and B
share one RMS target bounded so peak is at most `0.95`. Use the frozen seed and
`space=0.5`; `space` has no mono effect.

Record smoothness, musical usefulness, source identity, evolution, grain,
atonal ringing, periodicity, event stability, crest distraction, exterior
behavior, preference, and usability.

Pass requires:

- no unusable candidate row
- candidate preferred or tied on at least `12` of `15` rows
- no source family loses all three ratios
- no exposed vocoder colour, rough periodicity, cyclic repetition, doubled
  attack, stutter, or static freeze

Listening is promotion authority. Metrics cannot waive a loss.

## Linked-Stereo Gate

Only after mono passes, render the retained stereo pack at every ratio and
`space` value. An eligible listener independent of the operator assesses
centre stability, width, width pumping, one-sided texture, channel echo,
low-frequency image, and monotonic `space`. The operator's one-ear hearing does
not satisfy this gate.

Pass requires no unusable row, no mono or duplicate instability, and image
stability preferred or tied with PaulXStretch on at least `12` of `15`
neutral-space rows. Missing independent review blocks admission.

## Rejection, Cleanup, And Minimal Admission

Any admission miss rejects the complete candidate. Record one dominant cause
and stopped gate. Delete the worktree, branch, checkpoint, module, tests, build
state, and candidate listening assembly. Retain only external comparator
evidence and the docs closeout.

Do not tune, repair, or rerun a failed checkpoint. Another implementation
requires another complete brief. Failure does not close the PaulX-like product
target unless the operator explicitly stops it.

Only a complete pass may admit:

- private `creative_audited_variance_renewal`
- fixed-ratio neutral-`Dream` request and renderer
- structural and synthetic regression tests
- one internal creative-engine version identifier

Do not admit a public character enum, `motion`, `detail`, cache schema,
artifact surface, report mode, runtime route, pitch path, dynamic ratio,
another character, router, Loophole, or Chorus integration.

## Sources

- [Creative Stretch Source Triangulation](../research/specimen-dossiers/creative-stretch-source-triangulation.md)
- [Rejected Batch 31.23 brief](./offline-creative-variance-compensated-renewal-spectral-brief.md)
- [PaulX reference recovery](../logs/2026-07/20-g10-031-paulx-reference-recovery.md)
- [PaulXStretch pinned magnitude and phase path](https://github.com/essej/paulxstretch/blob/8ec191fdd7203354c79391cbc04c9fd83fa30ea0/Source/PS_Source/Stretch.cpp#L109-L263)
- [PaulXStretch pinned frame blend and source accumulator](https://github.com/essej/paulxstretch/blob/8ec191fdd7203354c79391cbc04c9fd83fa30ea0/Source/PS_Source/Stretch.cpp#L320-L563)

## Next Task

Complete every row in the concealed Batch 31.25 mono listening pack without
opening its key. Listen to all five `8x` rows first, then `4x` and `16x`.
