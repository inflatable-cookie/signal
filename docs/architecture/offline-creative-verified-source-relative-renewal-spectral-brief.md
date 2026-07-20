# Offline Creative ListeningLedSourceRelativeRenewalSpectral Renderer Brief

Status: rejected at synthetic `Y08`; candidate deleted
Owner: dsp
Updated: 2026-07-20
Contract: `085`
Roadmap: `g10.031`, Batches 31.28-31.35

## Decision

Batch 31.33 freezes one fresh Signal-owned
`ListeningLedSourceRelativeRenewalSpectral` candidate for neutral `Dream` at
`4x`, `8x`, and `16x`.

Retain the complete source-relative renderer, audited seed, request, transform,
map, native-channel relation law, synthesis, boundaries, state, and terminal
gates below. Change one admission rule: `Y02` must measure and record every
PaulX-relative pitch row, but its old reference-plus-`2`-cent ceiling is a
diagnostic rather than a terminal assertion. Concealed listening owns whether
the measured tonal deviation is objectionable.

This is an operator-authorized Contract `085` boundary change, not a seed,
window, hop, transform, phase, threshold, or scalar repair. Checkpoints
`d94612dd` and `790119b7` remain rejected and deleted. Batch 31.34 must build
fresh source under the new identity and may not recover either implementation.

The correction follows the product evidence. Batch 31.25 passed concealed mono
as `15/15` ties against PaulXStretch. Operator speaker review found its stereo
output solid apart from the source-relative balance inversion. The later
native left/right law was designed to remove that defect, but its fresh
checkpoints stopped before listening on pitch deltas of about `1.55`, `6.50`,
and `2.02` cents beyond their reference-relative ceilings. Those deltas remain
mandatory diagnostics. They no longer outrank the creative listening authority.

No mid/side magnitude synthesis, per-component orientation, post-render gain,
limiter, compressor, phase propagation, magnitude recurrence, transient
detector, onset reset, component layer, or range switch is present.

## Batch 31.34 Outcome

Fresh checkpoint `f76d5bb7241cd27f3a897ff9cf1b8c7e678cc91c`
passed compile, construction `1/1`, and structural admission `15/15` without
post-checkpoint repair or rerun. Synthetic admission selected all nine owners:
eight passed and `Y08` failed.

`Y02` passed its complete listening-led pitch diagnostic. `Y08` found an
exact-zero run of at least one `H` block in the impulse row at `4x`, `8x`, and
`16x`. The frozen test applied that dropout assertion over the complete
impulse output. The normative text separately says first-difference crest uses
the complete impulse output and the dropout assertion uses mapped non-zero
support. Batch 31.25's otherwise matching mono topology also passed `Y08`.

The checkpoint is rejected under its frozen assertion. This receipt does not
yet establish whether the dominant cause is renderer support or an
over-broad executable interpretation of mapped non-zero support. Do not repair
or rerun it. Batch 31.35 must resolve that evidence boundary from retained
briefs and receipts before another candidate can exist.

Listening did not open. Cleanup deleted the worktree, branch, checkpoint,
module, tests, local build state, and candidate artifacts. The disposable
nextest cache was moved to Trash. No candidate DSP entered `main`.

## Batch 31.29 Predecessor Outcome

Fresh checkpoint `d94612dd9f4ca9ba51724c826cac1d9375c27ff8`
passed compile, construction `1/1`, and structural admission `15/15` without
post-checkpoint repair or rerun. Synthetic admission completed all nine owners:
seven passed and two failed.

`Y04` found two active replica regions instead of one in one `16x` row. Its
failure message did not distinguish impulse from impulse train. `Y02` also
missed two `4x` pitch rows: `10.960881380` and `10.960712818` cents against
PaulX-relative ceilings of `9.410431632` and `4.461974128` cents.

The rejection applies to that checkpoint. The evidence contradiction's
dominant cause is the unfrozen candidate seed: stochastic synthesis was judged
without one authoritative request identity. Seed variation may or may not
explain each failed row, but the receipt cannot attribute either miss to the
transform, map, or ratio range. Do not turn it into a seed, window, hop,
threshold, or range sweep.

No listening gate ran. Cleanup deleted the worktree, branch, checkpoint,
module, tests, build state, and candidate artifacts. No candidate DSP entered
`main`.

## Batch 31.31 Outcome

Fresh checkpoint `790119b7936d5166ffb814f9401ba1398d2d5db9`
passed compile, construction `1/1`, and structural admission `15/15`. The
single synthetic command selected all nine owners. Six passed before `Y02`
failed the complete pitch matrix; `Y08` and `Y09` were then cancelled by the
test runner.

The failed row was the `8x` chord: maximum partial error
`13.351828347` cents against an `11.331375778`-cent PaulX-relative ceiling.
`Y04` passed both impulse sources at all ratios, so the audited seed removed
the predecessor's replica miss. It did not supply robust tonal coherence.

Batch 31.29 failed two `4x` pitch rows under seed `17`; Batch 31.31 failed an
`8x` chord pitch row under `ADMISSION_SEED`. Two complete checkpoints now fail
the same tonal-pitch class across different seeds, material, and ratios.
Contract `084` Rule 7 requires architecture reassessment. This does not
authorize another seed, transform, phase, window, hop, threshold, or assertion
attempt.

Batch 31.32 completed that reassessment. No materially different complete
source-backed renderer satisfied the retained tonal, diffusive, linked-stereo,
exact-length, deterministic, and bounded-state boundaries without reopening a
rejected family. Renewal is closed without promotion. The PaulX-like product
target remains active evidence.

Batch 31.33 supersedes that closure for future Contract `085` execution after
the operator explicitly changed the governing gate. The repeated comparator
delta is no longer a terminal failure class. This does not alter either prior
receipt, revive deleted code, or claim tonal parity. It authorizes one fresh
listening-led candidate only.

Listening did not open. Cleanup deleted the worktree, branch, checkpoint,
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

`ADMISSION_SEED=0x0123456789abcdef`. One complete address vector uses
`seed=ADMISSION_SEED`, `j=7`, `b=11`, and `s=BASE`:

| Address stage | Exact `u64` value |
| --- | --- |
| `mix64(j xor FRAME)` | `0xbeec58feebe20517` |
| `mix64(b xor BIN)` | `0x1faf24ea33da9322` |
| bin hash rotated left `21` | `0x9d467b526443f5e4` |
| outer `mix64` input | `0x12cc358a445d734e` |
| final `z` | `0xaefa063073d5e350` |
| high-`53` numerator `z>>11` | `0x0015df40c60e7abc` |

`tests.rs` owns one `COUNTER_VECTORS` constant containing these values. Every
construction and structural assertion refers to its named fields. The complete
address vector's `seed` field is also `ADMISSION_SEED`. Every candidate render
in synthetic and listening admission uses that named field. No helper accepts
an implicit or locally chosen admission seed. The determinism owner derives
its changed-seed control with `mix64(ADMISSION_SEED)`. Duplicate hexadecimal
counter or admission-seed literals elsewhere in the candidate are forbidden.

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

- worktree: `signal-candidate-31-34`
- branch: `candidate/g10-031-listening-led-source-relative-renewal`
- module:
  `crates/signal-dsp-stretch/src/creative_listening_led_source_relative_renewal/`
- files: `mod.rs`, `plan.rs`, `analysis.rs`, `relation.rs`, `synthesis.rs`,
  `tests.rs`

The isolated `lib.rs` may declare the module privately. No public API,
production tier, dependency, feature, report, binary, fixture, cache, artifact
schema, route, Loophole, or Chorus change is allowed. Listening assembly stays
ignored under `target/`.

Test prefixes are only:

- `listening_led_source_relative_renewal_construction_`
- `listening_led_source_relative_renewal_structural_`
- `listening_led_source_relative_renewal_synthetic_`

`tests.rs` owns one compile-linked `GATE_OWNERS` table with exactly `24` unique
IDs, names, and function pointers: `15` structural and `9` synthetic. The sole
construction owner validates that manifest and every `COUNTER_VECTORS` field.
It also validates that the address seed is the sole `ADMISSION_SEED` used by
synthetic and listening request assembly. No required test is ignored. Every
accumulator has an explicit type.

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
`listening_led_source_relative_renewal_structural_<suffix>`. `S04` contains no
independent handwritten counter literal. Run structural admission once and
require exactly `15/15`.

Synthetic owners are the audited predecessor's `Y01` through `Y09` meanings,
renamed with the listening-led prefix. Its `Frozen Synthetic Sources` and
`Exact Measurements` sections remain normative unchanged except for `Y02`.
`Y09` additionally exercises source relation, channel magnitude,
whole/band/window balance, and all ratios at `space=0`, `0.5`, and `1`.

`Y02` renders the low tone, mid tone, and every chord partial at every ratio.
It must complete the frozen estimator, require a finite detected peak for each
partial, and record candidate absolute error, PaulX error, and signed delta in
cents. It has no pitch-error ceiling and cannot reject a finite measured row.
Every other synthetic threshold remains terminal and unchanged.

Every owner renders all rows before one final assertion. Every candidate row
uses `ADMISSION_SEED`; test helpers cannot supply another seed. Run synthetic
admission once and require exactly `9/9`. `Y02` passes only when its complete
diagnostic matrix was produced under the frozen estimator.

## Listening And Stereo Admission

Only after terminal objective admission and complete `Y02` diagnostics, repeat
the retained concealed mono pack:
percussion, bass, vocals, pads/sustains, and full mix at `4x`, `8x`, and `16x`
against PaulXStretch 1.6.0 default / FFT `16384`. Use `ADMISSION_SEED`,
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

Any terminal objective or listening miss rejects the complete candidate.
Exceeding the former `Y02` reference-relative ceiling does not. Record one
dominant cause and stopped gate. Delete worktree, branch, checkpoint, module,
tests, build state, and listening assembly. Do not tune, repair, or rerun a
failed checkpoint.

Only a complete pass may admit the private module, fixed-ratio neutral-`Dream`
request and renderer, structural/synthetic regressions, and one internal
creative-engine version. Do not admit public character, motion, detail, cache,
artifact, report, route, pitch, dynamic ratio, other character, router,
seed/reroll control, Loophole, or Chorus integration. Product seed variation
requires a later frozen multi-seed character review; it cannot reinterpret this
single-seed admission receipt.

## Sources

- [Rejected source-relative predecessor](./offline-creative-source-relative-renewal-spectral-brief.md)
- [Audited synthetic authority](./offline-creative-audited-variance-compensated-renewal-spectral-brief.md)
- [Batch 31.27 vector rejection](../logs/2026-07/20-g10-031-source-relative-vector-rejection.md)
- [Batch 31.30 seed-authority reassessment](../logs/2026-07/20-g10-031-seed-authority-reassessment.md)
- [Batch 31.31 seed-audited rejection](../logs/2026-07/20-g10-031-seed-audited-renewal-rejection.md)
- [Batch 31.32 tonal-coherence closure](../logs/2026-07/20-g10-031-renewal-tonal-coherence-closure.md)
- [Batch 31.33 listening-led reopening](../logs/2026-07/20-g10-031-listening-led-renewal-reopening.md)
- [Batch 31.34 listening-led rejection](../logs/2026-07/20-g10-031-listening-led-renewal-rejection.md)
- [Creative product contract](../contracts/085-creative-time-stretch-product-and-routing-contract.md)

## Next Task

Run Batch 31.35 only. Reconcile the `Y08` complete-impulse measurement range
with its mapped-non-zero-support dropout boundary and the passed Batch 31.25
receipt. Decide whether the miss is renderer evidence or gate-construction
evidence. Do not implement DSP, repair or rerun the checkpoint, or push.
