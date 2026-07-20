# Offline Creative SourceRelativeRenewalSpectral Renderer Brief

Status: rejected at structural exact-vector proof
Owner: dsp
Updated: 2026-07-20
Contract: `085`
Roadmap: `g10.031`, Batch 31.26

## Decision

Build one fresh Signal-owned `SourceRelativeRenewalSpectral` candidate for
neutral `Dream` at `4x`, `8x`, and `16x`.

Retain the Batch 31.25 mono renderer topology that passed every hard,
synthetic, and concealed listening gate. Replace only its rejected stereo
representation. Stereo analysis is native left/right complex analysis. One
counter-addressed phase rotation renews each linked spectral pair while the
source interchannel phase relation and each channel magnitude remain explicit.

This is not a patch to checkpoint `97ee7056`. Do not recover its source,
tests, helpers, build state, or listening assembly. Implement this brief
cleanly in one new disposable worktree.

No mid/side magnitude synthesis, per-component first-sample orientation,
post-render channel gain, limiter, compressor, instantaneous-frequency
carrier, phase propagation, magnitude recurrence, transient detector, onset
reset, or component layer is present.

## Outcome

Checkpoint `1f05cc33dcc57b5714f02bf71f05a44d4ff98f09` passed compile and
construction `1/1`. Structural admission selected exactly `15` owners. Fourteen
passed. `source_relative_renewal_structural_mono_renewal` failed its frozen
`mix64(1)` vector before synthetic execution.

The implementation of the normative wrapping finalizer returned
`0x5692161d100b05e5`. The frozen assertion expected
`0x569216d1009b05e5`: the assertion transposed the middle `1d10` into
`d100`. The formula and returned value agree; the executable evidence vector
does not. This is an evidence-construction failure, not a stereo or audible
renderer result.

The checkpoint is rejected without assertion repair or rerun. Synthetic,
mono-listening, and stereo gates did not open. The disposable worktree,
branch, checkpoint, module, tests, and build state were deleted. No candidate
DSP entered `main`.

## Supported Request

The private candidate accepts `CandidateRequest<'a>` with exactly:

- finite mono or interleaved stereo `input`
- `channels` equal to `1` or `2`
- `sample_rate` from `8000` through `192000` Hz
- exact `target_frames`
- explicit `seed`
- finite `space` in `[0,1]`

Let source frames be `L` and target frames be `T`. Require `4L<=T<=16L`
with checked multiplication. Reject values above `2^53-1`, partial stereo
frames, non-finite samples, unsupported rates or channels, compression, range
misses, overflow, and every non-empty zero-target request before output
allocation. Empty input with `T=0` returns empty.

The only entry is
`render(CandidateRequest<'_>) -> Result<Vec<f32>, CandidateError>`. The closed
error enum covers request, size, allocation-bound, and non-finite-processing
failure. Motion, detail, pitch, dynamic ratio, reverse, other characters,
cache, artifact routing, realtime execution, and public exposure remain out of
scope.

## Transform, Map, And Analysis

Use the passed mono geometry unchanged:

- `N=clamp(nearest_power_of_two(round(2F/3)),8192,131072)`; integer halves
  round upward and power-of-two distance ties choose the larger power
- `H=N/2`; at `44.1` and `48 kHz`, `N=32768`, `H=16384`
- periodic Hann `w[n]=0.5-0.5*cos(2*pi*n/N)`
- energy gain `G=sqrt(N/sum(w[n]^2))`
- output block `j` begins at `y_j=jH`
- sole source centre `x_j=((y_j+0.5)L/T)-0.5`, evaluated with checked `u128`
  numerator `(2y_j+1)L` and denominator `2T`
- four-point cubic Lagrange interpolation over `i-1..i+2`, with exact zero
  outside `[0,L)`

One `FrameSchedule` computes `x_j` once. Mono analyzes one native component.
Stereo analyzes native left and right with the same centre, interpolation,
window, and forward transform. Retain the complex non-negative coefficients
`X_L[j,b]` and `X_R[j,b]`; do not reduce stereo analysis to magnitudes or
mid/side components.

Mono orientation remains the sign of the first exactly non-zero input sample,
or `+1` for exact silence. This preserves the passed mono law.

Stereo has one common polarity orientation, never one per channel or
component. Scan `M=(L+R)/sqrt(2)` from source start and use the sign of its
first exactly non-zero sample. Only if `M` is exact silence, scan
`D=(L-R)/sqrt(2)`. If both are exact silence, use `+1` and mark the request
silent. This orientation may change common waveform polarity only. It never
chooses left/right balance or relative phase.

## Counter Phase And Mono Renewal

Retain wrapping `mix64`, frame/bin counter addressing, little-endian tags
`RNWFRAME`, `RNWBIN00`, and `RNWBASE0`, and conversion of the high `53` bits to
`theta=2*pi*u-pi` exactly as frozen in the rejected audited brief.

For active mono bin `b`, let `A=|X[b]|` and emit
`Y[b]=o*A*exp(i*theta)`. DC is real `o*A[0]`; Nyquist is exact zero; negative
bins are conjugate mirrors. Exact silence remains exact zero. This path must
match the frozen Batch 31.25 mono formulas and seed addressing exactly.

## Source-Relative Stereo Pair Law

For each positive non-Nyquist bin, let `A_L=|X_L|` and `A_R=|X_R|`.

If both magnitudes are non-zero, compute in `f64`:

`C=(X_R*conj(X_L))/(A_R*A_L)`.

Normalize `C` once to unit magnitude after rejecting non-finite or zero norm.
Let `delta=atan2(im(C),re(C))` in `(-pi,pi]`; exact negative-real ties use
`+pi`. `delta` is the source right-minus-left phase relation. No previous
frame, peak, threshold, or dormant relation exists.

Define frequency weight:

- `h=0` at and below `250 Hz`
- `h=1` at and above `1500 Hz`
- between them, `t=(f-250)/(1500-250)` and `h=t*t*(3-2*t)`

Let `d=abs(delta)`. The output relation is:

- `delta'=0` when `delta=0`
- `delta'=delta` when `d>=pi/2`
- otherwise
  `delta'=sign(delta)*(d+space*h*(pi/2-d))`

Thus `space=0` preserves the analyzed relation. Increasing `space` moves only
already non-zero coherent relations toward quadrature, never narrows an
already-wide relation, never changes a channel magnitude, and never creates
width from duplicate mono.

With common orientation `o` and base phase `theta`:

- `Y_L=o*A_L*exp(i*(theta-delta'/2))`
- `Y_R=o*A_R*exp(i*(theta+delta'/2))`

If exactly one channel magnitude is zero, that output bin remains zero and the
active channel uses `o*A*exp(i*theta)`. If both are zero, both remain zero.

DC preserves per-channel magnitude and relative sign. If both DC coefficients
are non-zero, set left to `o*A_L` and right to
`o*sign(X_L[0]*X_R[0])*A_R`. A single active DC coefficient uses `o*A`.
Nyquist is exact zero in both channels. Complete both spectra by conjugate
mirroring.

This law owns the failed boundary directly:

- per-frame left and right spectral magnitudes equal their analyzed source
  magnitudes at every `space`
- `space=0` preserves the complete non-zero-bin interchannel relation
- duplicate stereo is samplewise equal to duplicated mono
- common polarity negates output samplewise
- exact whole-source anti-phase remains anti-phase samplewise
- channel swap swaps per-frame magnitudes and conjugates the relation exactly;
  time-domain sample equality is not claimed at the negative-real half-angle
  branch and is gated as an image relationship

No sign from a near-silent side or channel sample can select render-wide
balance.

## Pairwise Synthesis And Boundaries

Inverse-transform with `1/N`. Apply no synthesis window. Blend the same two
adjacent frames independently for each native channel:

- `u=(n+0.5)/H`
- `a=0.5+0.5*cos(pi*u)`
- `b=1-a`
- `c=1/sqrt(a*a+b*b)`
- `q[j,n]=G*c*(a*z[j,H+n]+b*z[j+1,n])`

Cache exactly two adjacent inverse frames per active channel. Both channels
share schedule, frame index, counter address, blend coefficients, envelope,
and crop. There is no decode matrix or channel-local gain.

Retain exterior envelope `E=min(H,floor(T/4))`. For `E>=2`, multiply head
frame `y<E` by `sin((pi/2)y/(E-1))` and tail frame `y>=T-E` by
`sin((pi/2)(T-1-y)/(E-1))`; apply both when they overlap. Otherwise use one.
Emit exactly `[0,T)`. Never append, resize-fill, wrap, reflect, or repair.

The fixed envelope remains a known audible distinction from the PaulX capture,
not an admission failure. Every non-finite coefficient, relation, accumulator,
or output sample fails the render.

## State, Allocation, Determinism, And Cost

Allocate output, window, two stereo analysis spectra, inverse workspace, two
adjacent frames per active channel, FFT plans, and scratch before frame one.
Reuse them for the complete render. Mono may use one analysis spectrum.

Excluding input and required output, actual peak working state must be at most
`32 MiB` at `N<=131072` and independent of duration. A candidate-only counting
allocator includes FFT plans and scratch, subtracts only returned output
capacity, asserts no allocation or reallocation after processing begins, and
compares one-second with one-hour capacity plans.

Mono performs one forward and one inverse FFT per new frame. Stereo performs
two of each. Cost is `O((T/H)*N*log N)`. Counter phase, fixed traversal,
explicit ties, and no parallel reduction require byte-identical repeats on one
supported deterministic target. The renderer is offline only.

## Candidate Isolation

Use exactly:

- worktree: `signal-candidate-31-27`
- branch: `candidate/g10-031-source-relative-renewal`
- module: `crates/signal-dsp-stretch/src/creative_source_relative_renewal/`
- files: `mod.rs`, `plan.rs`, `analysis.rs`, `relation.rs`, `synthesis.rs`,
  `tests.rs`

The isolated `lib.rs` may declare the module privately. No public API,
production tier, dependency, feature, report, binary, fixture, cache, artifact
schema, route, Loophole, or Chorus change is allowed. Listening assembly stays
ignored under `target/`.

## Construction And Gate Ownership

Test prefixes are only:

- `source_relative_renewal_construction_`
- `source_relative_renewal_structural_`
- `source_relative_renewal_synthetic_`

`tests.rs` owns one compile-linked `GATE_OWNERS` table with exactly `24`
unique IDs, names, and non-null function pointers: `15` structural and `9`
synthetic. No required test is ignored. Every accumulator has an explicit type.

Construction order:

1. `effigy test compile`
2. run the construction prefix and require exactly `1/1`
3. create one local immutable checkpoint and record its hash
4. freeze source, tests, assertions, manifest, and checkpoint

Only compiler/type/visibility/manifest repairs may occur before the clean
construction receipt, and only when they change no DSP formula, source,
measurement, threshold, or assertion. Any later miss is terminal.

Structural owners are:

| ID | Exact owner | Boundary |
| --- | --- | --- |
| S01 | `source_relative_renewal_structural_request_preallocation` | all request decisions before output allocation |
| S02 | `source_relative_renewal_structural_transform_map` | `N`, `H`, checked map, counts, monotonicity |
| S03 | `source_relative_renewal_structural_window_interpolation_gain` | Hann, cubic reads, zero exterior, `G` |
| S04 | `source_relative_renewal_structural_mono_renewal` | old mono orientation, address, spectrum, DC, Nyquist |
| S05 | `source_relative_renewal_structural_native_stereo_analysis` | one linked L/R read and retained complex coefficients |
| S06 | `source_relative_renewal_structural_relation_space` | `C`, ties, widening law, channel magnitudes, source relation |
| S07 | `source_relative_renewal_structural_stereo_dc_nyquist` | DC magnitude/sign, zero Nyquist, Hermitian completion |
| S08 | `source_relative_renewal_structural_blend_envelope_crop` | pairwise blend, envelope, endpoints, exact crop |
| S09 | `source_relative_renewal_structural_edge_source_matrix` | edge sizes, silence, DC, impulses, tones, chords, noise |
| S10 | `source_relative_renewal_structural_determinism_seed` | byte repeat, active seed change, finiteness |
| S11 | `source_relative_renewal_structural_duplicate_polarity_antiphase` | samplewise duplicate, common polarity, anti-phase |
| S12 | `source_relative_renewal_structural_swap_relationship` | swapped magnitudes, conjugate relation, decoded image |
| S13 | `source_relative_renewal_structural_channel_balance` | per-bin L/R magnitude and band-energy preservation at all spaces |
| S14 | `source_relative_renewal_structural_allocation_memory` | actual `32 MiB`, duration independence, no processing allocation |
| S15 | `source_relative_renewal_structural_forbidden_mechanisms` | forbidden token/type inventory |

Synthetic owners remain the audited `Y01` through `Y09` meanings: crest,
pitch, impulse distribution, replica regions, noise periodicity, RMS
modulation, silence gap, integrity/discontinuity, and linked-stereo inventory.
Rename them with the new prefix. The exact sources, active support, formulas,
PaulX reference numbers, and tolerances in the predecessor brief's
`Frozen Synthetic Sources` and `Exact Measurements` sections are normative
unchanged. `Y09` additionally exercises the new relation and channel-balance
laws at every ratio and `space=0`, `0.5`, and `1`.

Run structural admission once and require exactly `15/15`. Then run synthetic
admission once and require exactly `9/9`. Every owner renders all its rows
before one final assertion. Do not stop inside an owner after the first row.

## Long-Form Mono Gate

Only after all structural and synthetic owners pass, repeat the retained
five-family concealed mono comparison against PaulXStretch 1.6.0 default / FFT
`16384` at `4x`, `8x`, and `16x`, with the frozen seed, `space=0.5`, exact crop,
one common RMS target, and `0.95` peak ceiling.

Pass remains: no unusable candidate row, preferred or tied on at least `12/15`,
no family losing all ratios, and no exposed vocoder colour, rough periodicity,
cyclic repetition, doubled attack, stutter, or static freeze. The Batch 31.25
`15/15` result is architecture evidence, not a waiver for a new implementation.

## Source-Relative Stereo Gate

Use the exact retained stereo originals whose mid downmixes produced `M001`
through `M005`:

- `0000-drums_percussion-000002.wav`
- `0004-bass-000236.wav`
- `0008-vocals-000010.wav`
- `0012-pads_sustains-000423.wav`
- `0016-full_mix-000144.wav`

Render all five at `4x`, `8x`, and `16x`, and at `space=0`, `0.5`, and `1`.
Capture PaulXStretch 1.6.0 default / FFT `16384` from the same stereo originals
before opening candidate listening. Exact-crop every file. Neutral A/B uses
`space=0.5`; source, candidate, and PaulX share one row RMS target under peak
`0.95`.

For source and render, channel balance is
`10*log10(sum(R^2)/sum(L^2))`. Measure whole active audio and bands `0..250`,
`250..1500`, and `1500..Nyquist`. Candidate-source error must be at most
`0.75 dB` in every whole-render and band row at every `space`. Balance spread
between the three `space` renders of one source/ratio must be at most
`0.50 dB`. Any channel-dominance reversal where source magnitude is at least
`0.50 dB` rejects. These are source-image integrity bounds, not character
metrics.

Also measure broadband balance in `4`-second output windows with `2`-second
hops, clipped to `[0,T)`. Map each window edge to the source with the sole
linear source/output map and measure the corresponding source interval. Omit a
window only when both source channels are below `-60 dBFS` RMS. Candidate-
source window error must be at most `1.50 dB`; a dominance reversal rejects
when source-window balance magnitude is at least `0.75 dB`. A final clipped
window shorter than `2` seconds is not measured. This gate prevents opposing
local image errors from cancelling in the whole-render total.

Structural analysis must also prove per-bin channel magnitude equality at
every `space`, exact source relation at `space=0`, exact duplicate mono, and
non-decreasing per-bin side energy as `space` increases.

After objective admission, the operator may pre-screen on speakers. Final
promotion still requires an eligible independent listener to assess centre,
width, pumping, one-sided texture, channel echo, low-frequency image, and
monotonic `space`. Pass requires no unusable row and candidate neutral image
preferred or tied with PaulX on at least `12/15` rows. The operator's one-ear
hearing cannot satisfy the independent pass, but an observed fault may reject.

## Rejection, Cleanup, And Minimal Admission

Any miss rejects the complete candidate. Record one dominant cause and stopped
gate. Delete the worktree, branch, checkpoint, module, tests, build state, and
candidate listening assembly. Do not tune, repair, or rerun a failed
checkpoint.

Only a complete pass may admit the private module, fixed-ratio neutral-`Dream`
request and renderer, structural/synthetic regressions, and one internal
creative-engine version. Do not admit a public character enum, motion, detail,
cache, artifact surface, report mode, runtime route, pitch path, dynamic ratio,
other character, router, Loophole, or Chorus integration.

## Sources

- [Rejected audited predecessor](./offline-creative-audited-variance-compensated-renewal-spectral-brief.md)
- [Creative source triangulation](../research/specimen-dossiers/creative-stretch-source-triangulation.md)
- [Batch 31.25 stereo rejection](../logs/2026-07/20-g10-031-audited-renewal-stereo-rejection.md)
- [Creative time-stretch product contract](../contracts/085-creative-time-stretch-product-and-routing-contract.md)

## Next Task

Run Batch 31.28 only. Reconcile the incorrect frozen vector with the normative
formula, audit every exact construction vector, and either freeze fresh
complete candidate authority under a new identity or close the topology. Do
not implement candidate DSP in the same batch.
