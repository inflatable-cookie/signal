# Offline Creative VerifiedSourceRelativeRenewalSpectral Renderer Brief

Status: frozen; candidate rejected at synthetic admission
Owner: dsp
Updated: 2026-07-20
Contract: `085`
Roadmap: `g10.031`, Batches 31.28-31.29

## Decision

Build one fresh Signal-owned `VerifiedSourceRelativeRenewalSpectral` candidate
for neutral `Dream` at `4x`, `8x`, and `16x`.

Retain the source-relative architecture frozen in Batch 31.26. Batch 31.27
produced no synthetic, mono-listening, or stereo result. Its sole miss was an
incorrect handwritten `mix64(1)` assertion. This brief replaces that evidence
construction under a new identity. It does not recover, patch, or rerun
checkpoint `1f05cc33`.

No mid/side magnitude synthesis, per-component orientation, post-render gain,
limiter, compressor, phase propagation, magnitude recurrence, transient
detector, onset reset, or component layer is present.

## Outcome

Fresh checkpoint `d94612dd9f4ca9ba51724c826cac1d9375c27ff8`
passed compile, construction `1/1`, and structural admission `15/15` without
post-checkpoint repair or rerun. Synthetic admission completed all nine owners:
seven passed and two failed.

`Y04` found two active replica regions instead of one in one `16x` row. Its
failure message did not distinguish impulse from impulse train. `Y02` also
missed two `4x` pitch rows: `10.960881380` and `10.960712818` cents against
PaulX-relative ceilings of `9.410431632` and `4.461974128` cents.

The dominant cause is incomplete range ownership in one fixed-resolution
renewal topology: its frozen `32768/16384` transform and source-relative map
do not jointly retain the low-ratio tonal estimate and single-region
high-ratio transient distribution. This is an architectural diagnosis from
the paired ratio-end failures, not proof that transform size alone is causal.
Do not turn it into a window, hop, or threshold sweep.

No listening gate ran. Cleanup deleted the worktree, branch, checkpoint,
module, tests, build state, and candidate artifacts. No candidate DSP entered
`main`.

## Supported Request

The private `CandidateRequest<'a>` contains exactly finite mono or interleaved
stereo `input`, `channels` equal to `1` or `2`, `sample_rate` from `8000`
through `192000`, exact `target_frames`, explicit `seed`, and finite `space` in
`[0,1]`.

Let source frames be `L` and target frames be `T`. Require `4L<=T<=16L` with
checked multiplication. Reject values above `2^53-1`, partial stereo frames,
non-finite samples, unsupported rates or channels, range misses, overflow, and
every non-empty zero-target request before output allocation. Empty input with
`T=0` returns empty.

The only entry is
`render(CandidateRequest<'_>) -> Result<Vec<f32>, CandidateError>`. The closed
error enum owns request, size, allocation-bound, and non-finite-processing
failure. Motion, detail, pitch, dynamic ratio, reverse, other characters,
cache, artifacts, realtime execution, and public exposure are absent.

## Transform, Map, And Analysis

- `N=clamp(nearest_power_of_two(round(2F/3)),8192,131072)`; integer halves
  round upward and power-of-two distance ties choose the larger power
- `H=N/2`; `N=32768` and `H=16384` at `44.1` and `48 kHz`
- periodic Hann `w[n]=0.5-0.5*cos(2*pi*n/N)`
- energy gain `G=sqrt(N/sum(w[n]^2))`
- output block `j` begins at `y_j=jH`
- sole source centre `x_j=((y_j+0.5)L/T)-0.5`, evaluated from checked `u128`
  numerator `(2y_j+1)L` and denominator `2T`
- four-point cubic Lagrange interpolation over `i-1..i+2`, with exact zero
  outside `[0,L)`

One `FrameSchedule` computes `x_j` once. Mono analyzes one native component.
Stereo analyzes native left and right with the same centre, interpolation,
window, and forward transform. Retain complex non-negative coefficients
`X_L[j,b]` and `X_R[j,b]`; never reduce stereo to magnitudes or mid/side.

Mono orientation is the sign of the first exactly non-zero input sample, or
`+1` for silence. Stereo has one common polarity orientation. Scan
`M=(L+R)/sqrt(2)` first, then `D=(L-R)/sqrt(2)` only when `M` is exact silence.
Use `+1` and mark silence when both are exact silence. Orientation may change
common polarity only; it never chooses balance or relative phase.

## Counter Phase And Audited Vectors

Define wrapping `mix64(z)`:

1. `z=(z xor (z>>30))*0xBF58476D1CE4E5B9`
2. `z=(z xor (z>>27))*0x94D049BB133111EB`
3. return `z xor (z>>31)`

For frame `j`, bin `b`, and stream tag `s`:

`z=mix64(seed xor mix64(j xor FRAME) xor rotate_left(mix64(b xor BIN),21) xor s)`.

`FRAME`, `BIN`, `BASE`, and synthetic `TEST` are little-endian ASCII tags.
Convert the high `53` bits with `u=(z>>11)/2^53`, then
`theta=2*pi*u-pi`.

Python integer arithmetic and Ruby integer arithmetic independently produced
the same complete table. This table is the only handwritten exact counter
literal owner in candidate tests:

| Owner | Exact `u64` value |
| --- | --- |
| `FRAME = RNWFRAME` | `0x454d415246574e52` |
| `BIN = RNWBIN00` | `0x30304e4942574e52` |
| `BASE = RNWBASE0` | `0x3045534142574e52` |
| `TEST = RNWTEST0` | `0x3054534554574e52` |
| `mix64(0)` | `0x0000000000000000` |
| `mix64(1)` after round one | `0xbf58476d1ce4e5b9` |
| `mix64(1)` after round two | `0x5692161dbd2f29de` |
| `mix64(1)` final | `0x5692161d100b05e5` |
| `mix64(u64::MAX)` final | `0xb4d055fcf2cbbd7b` |

One complete address vector uses `seed=0x0123456789abcdef`, `j=7`, `b=11`,
and `s=BASE`:

| Address stage | Exact `u64` value |
| --- | --- |
| `mix64(j xor FRAME)` | `0xbeec58feebe20517` |
| `mix64(b xor BIN)` | `0x1faf24ea33da9322` |
| bin hash rotated left `21` | `0x9d467b526443f5e4` |
| outer `mix64` input | `0x12cc358a445d734e` |
| final `z` | `0xaefa063073d5e350` |
| high-`53` numerator `z>>11` | `0x0015df40c60e7abc` |

`tests.rs` owns one `COUNTER_VECTORS` constant containing these values. Every
construction and structural assertion refers to its named fields. Duplicate
hexadecimal counter literals elsewhere in the candidate are forbidden.

For active mono bin `b`, emit `Y=o*A*exp(i*theta)`. DC is real `o*A[0]`,
Nyquist is zero, negative bins are conjugate mirrors, and exact silence stays
zero.

## Source-Relative Stereo Pair Law

For positive non-Nyquist bin magnitudes `A_L=|X_L|` and `A_R=|X_R|`, when both
are non-zero compute in `f64`:

`C=(X_R*conj(X_L))/(A_R*A_L)`.

Normalize `C` once. Reject non-finite or zero norm. Let
`delta=atan2(im(C),re(C))` in `(-pi,pi]`; exact negative-real ties use `+pi`.
No previous-frame or dormant relation exists.

Frequency weight `h` is zero at and below `250 Hz`, one at and above
`1500 Hz`, and `t*t*(3-2*t)` between them for
`t=(f-250)/(1500-250)`.

Let `d=abs(delta)`:

- `delta'=0` when `delta=0`
- `delta'=delta` when `d>=pi/2`
- otherwise
  `delta'=sign(delta)*(d+space*h*(pi/2-d))`

With common orientation `o` and counter phase `theta`:

- `Y_L=o*A_L*exp(i*(theta-delta'/2))`
- `Y_R=o*A_R*exp(i*(theta+delta'/2))`

An inactive channel bin stays zero; the active bin uses `o*A*exp(i*theta)`.
DC preserves both channel magnitudes and relative sign. Nyquist is zero.
Complete both spectra by conjugate mirroring.

This preserves per-frame channel magnitudes at every `space`, the complete
non-zero-bin source relation at `space=0`, duplicate mono samplewise, common
polarity samplewise, and anti-phase samplewise within the frozen tolerance.
Swap owns magnitudes, conjugate relation, and decoded image; exact time-domain
swap is not claimed at the negative-real half-angle branch.

## Synthesis, Boundaries, State, And Cost

Inverse-transform with `1/N` and no synthesis window. For `0<=n<H`:

- `u=(n+0.5)/H`
- `a=0.5+0.5*cos(pi*u)`
- `b=1-a`
- `c=1/sqrt(a*a+b*b)`
- `q[j,n]=G*c*(a*z[j,H+n]+b*z[j+1,n])`

Cache two adjacent inverse frames per active channel. Channels share schedule,
frame index, counter, blend, envelope, and crop. There is no decode matrix or
channel-local gain.

Exterior envelope `E=min(H,floor(T/4))`. For `E>=2`, multiply head frame
`y<E` by `sin((pi/2)y/(E-1))` and tail frame `y>=T-E` by
`sin((pi/2)(T-1-y)/(E-1))`; apply both when they overlap. Otherwise use one.
Emit exactly `[0,T)`. Never append, wrap, reflect, resize-fill, or repair.

Allocate output, window, analysis spectra, inverse workspace, adjacent frames,
FFT plans, and scratch before frame one. Excluding input and required output,
actual peak working state is at most `32 MiB` at `N<=131072` and independent
of duration. The counting allocator includes plans and scratch, subtracts only
returned output capacity, and requires no allocation after processing starts.

Mono performs one forward and inverse FFT per new frame; stereo performs two
of each. Cost is `O((T/H)*N*log N)`. Fixed traversal, counter phase, explicit
ties, and no parallel reduction require byte-identical repeats. Offline only.

## Candidate Isolation And Construction

Use exactly:

- worktree: `signal-candidate-31-29`
- branch: `candidate/g10-031-verified-source-relative-renewal`
- module:
  `crates/signal-dsp-stretch/src/creative_verified_source_relative_renewal/`
- files: `mod.rs`, `plan.rs`, `analysis.rs`, `relation.rs`, `synthesis.rs`,
  `tests.rs`

The isolated `lib.rs` may declare the module privately. No public API,
production tier, dependency, feature, report, binary, fixture, cache, artifact
schema, route, Loophole, or Chorus change is allowed. Listening assembly stays
ignored under `target/`.

Test prefixes are only:

- `verified_source_relative_renewal_construction_`
- `verified_source_relative_renewal_structural_`
- `verified_source_relative_renewal_synthetic_`

`tests.rs` owns one compile-linked `GATE_OWNERS` table with exactly `24` unique
IDs, names, and function pointers: `15` structural and `9` synthetic. The sole
construction owner validates that manifest and every `COUNTER_VECTORS` field.
No required test is ignored. Every accumulator has an explicit type.

Construction order:

1. `effigy test compile`
2. run the construction prefix and require exactly `1/1`
3. create one immutable local checkpoint and record its hash
4. freeze source, tests, assertions, manifest, and checkpoint

Only compiler, type, visibility, ownership, or manifest assembly repairs may
occur before the clean construction receipt when they change no formula,
literal, source, measurement, threshold, or assertion. A vector mismatch is
not an assembly repair; it rejects the candidate. Any later miss is terminal.

## Gate Ownership

Structural owners are:

| ID | Exact owner suffix | Boundary |
| --- | --- | --- |
| S01 | `request_preallocation` | every request decision before output allocation |
| S02 | `transform_map` | `N`, `H`, checked map, counts, monotonicity |
| S03 | `window_interpolation_gain` | Hann, cubic reads, zero exterior, `G` |
| S04 | `mono_renewal` | named counter-table use, orientation, spectrum, DC, Nyquist |
| S05 | `native_stereo_analysis` | one linked L/R read and retained complex coefficients |
| S06 | `relation_space` | `C`, ties, widening law, magnitudes, source relation |
| S07 | `stereo_dc_nyquist` | DC magnitude/sign, zero Nyquist, Hermitian completion |
| S08 | `blend_envelope_crop` | pairwise blend, envelope, endpoints, exact crop |
| S09 | `edge_source_matrix` | edges, silence, DC, impulses, tones, chords, noise |
| S10 | `determinism_seed` | byte repeat, active seed change, finiteness |
| S11 | `duplicate_polarity_antiphase` | samplewise duplicate, polarity, anti-phase |
| S12 | `swap_relationship` | swapped magnitudes, conjugate relation, decoded image |
| S13 | `channel_balance` | per-bin magnitude and band-energy preservation at all spaces |
| S14 | `allocation_memory` | actual `32 MiB`, duration independence, no processing allocation |
| S15 | `forbidden_mechanisms` | forbidden token and type inventory |

Each exact owner name is
`verified_source_relative_renewal_structural_<suffix>`. `S04` contains no
independent handwritten counter literal. Run structural admission once and
require exactly `15/15`.

Synthetic owners are the audited predecessor's `Y01` through `Y09` meanings,
renamed with the verified prefix. Its `Frozen Synthetic Sources` and
`Exact Measurements` sections remain normative unchanged. `Y09` additionally
exercises source relation, channel magnitude, whole/band/window balance, and
all ratios at `space=0`, `0.5`, and `1`. Every owner renders all rows before
one final assertion. Run synthetic admission once and require exactly `9/9`.

## Listening And Stereo Admission

Only after objective admission, repeat the retained concealed mono pack:
percussion, bass, vocals, pads/sustains, and full mix at `4x`, `8x`, and `16x`
against PaulXStretch 1.6.0 default / FFT `16384`. Use the frozen seed,
`space=0.5`, exact crop, common row RMS, and `0.95` peak ceiling. Pass requires
no unusable row, preferred or tied on at least `12/15`, no family loss, and no
forbidden vocoder, periodic, cyclic, doubled-attack, stutter, or freeze
character. Batch 31.25's mono pass is evidence, not a waiver.

Then capture PaulX from the same retained stereo originals and render all five
at every ratio and `space`. The rejected source-relative brief's exact whole,
three-band, mapped-window, dominance, spread, duplicate, relation, and
side-energy limits remain normative unchanged.

The operator may reject during speaker pre-screen. Promotion requires an
eligible independent listener. Neutral image must be preferred or tied with
PaulX on at least `12/15` rows with no unusable row. Missing independent review
blocks admission.

## Rejection, Cleanup, And Minimal Admission

Any miss rejects the complete candidate. Record one dominant cause and stopped
gate. Delete worktree, branch, checkpoint, module, tests, build state, and
listening assembly. Do not tune, repair, or rerun a failed checkpoint.

Only a complete pass may admit the private module, fixed-ratio neutral-`Dream`
request and renderer, structural/synthetic regressions, and one internal
creative-engine version. Do not admit public character, motion, detail, cache,
artifact, report, route, pitch, dynamic ratio, other character, router,
Loophole, or Chorus integration.

## Sources

- [Rejected source-relative predecessor](./offline-creative-source-relative-renewal-spectral-brief.md)
- [Audited synthetic authority](./offline-creative-audited-variance-compensated-renewal-spectral-brief.md)
- [Batch 31.27 vector rejection](../logs/2026-07/20-g10-031-source-relative-vector-rejection.md)
- [Creative product contract](../contracts/085-creative-time-stretch-product-and-routing-contract.md)

## Next Task

Run Batch 31.30 only. Reassess ratio-range ownership against the pinned
PaulXStretch render path and the complete Batch 31.29 receipt. Either freeze
one materially different, range-aware complete successor brief or close this
source-relative topology. Do not implement DSP in the same batch, reopen a
parameter sweep, or close the PaulX-like product target without explicit
operator direction.
