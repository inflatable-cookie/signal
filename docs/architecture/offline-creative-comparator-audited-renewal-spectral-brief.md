# Offline Creative ComparatorAuditedRenewalSpectral Renderer Brief

Status: rejected at synthetic admission; candidate deleted
Owner: dsp
Updated: 2026-07-21
Contract: `085`
Roadmap: `g10.031`, Batch 31.38

## Decision

Build one fresh Signal-owned `ComparatorAuditedRenewalSpectral` candidate for
neutral `Dream` at exact `4x`, `8x`, and `16x` expansion.

The operator explicitly changed creative-stereo promotion policy after Batch
31.37. Local mapped-window source-relative balance and dominance remain
mandatory diagnostics, but no longer reject a creative renderer by numeric
threshold. Hard stereo admission still owns structural relationships,
whole-render and three-band source balance, space consistency, and complete
evidence. Promotion still requires comparator-relative review by an eligible
independent stereo listener.

Retain the complete Batch 31.36 renderer formulas. This is a new candidate,
not a reinterpretation or repair of checkpoint `5d8eaf45`. Do not recover its
source, tests, helpers, build state, or listening assembly. Implement this
brief from the retained documentation in one new disposable worktree.

## Supported Request

The private `CandidateRequest<'a>` contains exactly finite mono or interleaved
stereo `input`, `channels` equal to `1` or `2`, `sample_rate` from `8000`
through `192000`, exact `target_frames`, explicit `seed`, and finite `space`
in `[0,1]`.

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

There is no transient detector, reset, reassignment, phase propagation,
magnitude recurrence, dormant relation, peak tracker, material split, or
second source timeline. Smooth phase-forgotten smear is the selected complete
creative representation, not a transparent transient claim.

## Counter Phase

Define wrapping `mix64(z)`:

1. `z=(z xor (z>>30))*0xBF58476D1CE4E5B9`
2. `z=(z xor (z>>27))*0x94D049BB133111EB`
3. return `z xor (z>>31)`

For frame `j`, bin `b`, and stream tag `s`:

`z=mix64(seed xor mix64(j xor FRAME) xor rotate_left(mix64(b xor BIN),21) xor s)`.

`FRAME`, `BIN`, `BASE`, and synthetic `TEST` are little-endian ASCII tags.
Convert the high `53` bits with `u=(z>>11)/2^53`, then
`theta=2*pi*u-pi`.

The exact counter table and address vector in
[SupportAuditedListeningLedSourceRelativeRenewalSpectral](./offline-creative-verified-source-relative-renewal-spectral-brief.md#counter-phase-and-audited-vectors)
are incorporated unchanged and are the only handwritten exact literal owner.
`ADMISSION_SEED=0x0123456789abcdef` remains the sole synthetic and listening
seed. A changed-seed determinism control uses `mix64(ADMISSION_SEED)`.

For active mono bin `b`, emit `Y=o*A*exp(i*theta)`. DC is real `o*A[0]`,
Nyquist is zero, negative bins are conjugate mirrors, and exact silence stays
zero.

## Linked Stereo Pair Law

For positive non-Nyquist bin magnitudes `A_L=|X_L|` and `A_R=|X_R|`, when both
are non-zero compute in `f64`:

`C=(X_R*conj(X_L))/(A_R*A_L)`.

Normalize `C` once. Reject non-finite or zero norm. Let
`delta=atan2(im(C),re(C))` in `(-pi,pi]`; exact negative-real ties use `+pi`.

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

Analysis decisions, source schedule, counter phase, blend, normalization, and
crop are shared across linked channels. Channels never draw unrelated random
trajectories. The law preserves per-frame channel magnitudes at every `space`,
the complete active-bin source relation at `space=0`, duplicate mono,
common-polarity behavior, and anti-phase behavior within the frozen structural
tolerance. It does not claim mapped-window waveform balance after adjacent
independently renewed frames blend; that observed quantity is diagnostic.

## Synthesis, Boundaries, State, And Cost

Inverse-transform with `1/N` and no synthesis window. For `0<=n<H`:

- `u=(n+0.5)/H`
- `a=0.5+0.5*cos(pi*u)`
- `b=1-a`
- `c=1/sqrt(a*a+b*b)`
- `q[j,n]=G*c*(a*z[j,H+n]+b*z[j+1,n])`

Cache two adjacent inverse frames per active channel. Channels share schedule,
frame index, counter, blend, envelope, and crop. There is no decode matrix,
channel-local gain, post-render balance correction, limiter, or compressor.

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

- worktree: `signal-candidate-31-39`
- branch: `candidate/g10-031-comparator-audited-renewal`
- module:
  `crates/signal-dsp-stretch/src/creative_comparator_audited_renewal/`
- files: `mod.rs`, `plan.rs`, `analysis.rs`, `relation.rs`, `synthesis.rs`,
  `tests.rs`

The isolated `lib.rs` may declare the module privately. No public API,
production tier, dependency, feature, report, binary, fixture, cache, artifact
schema, route, Loophole, or Chorus change is allowed. Listening assembly stays
ignored under `target/`.

Test prefixes are only:

- `comparator_audited_renewal_construction_`
- `comparator_audited_renewal_structural_`
- `comparator_audited_renewal_synthetic_`

`tests.rs` owns one compile-linked `GATE_OWNERS` table with exactly `24`
unique IDs, names, and function pointers: `15` structural and `9` synthetic.
It owns the predecessor's exact immutable `SYNTHETIC_SUPPORTS` table. The sole
construction owner validates the manifest, counter table, support table,
mapped intervals, range-type separation, and sole admission seed.

Construction order:

1. `effigy test compile`
2. run the construction prefix and require exactly `1/1`
3. create one immutable local checkpoint and record its hash
4. freeze source, tests, assertions, manifest, and checkpoint

Only compiler, type, visibility, ownership, or manifest assembly repairs may
occur before the clean construction receipt when they change no formula,
literal, source, measurement, threshold, or assertion. A vector mismatch is
not an assembly repair. Any later miss is terminal.

## Structural And Synthetic Gates

The `15` structural owners retain IDs `S01` through `S15`, meanings, exact
fixtures, and tolerances from the rejected support-audited brief. Rename only
their prefix. They jointly own request preallocation, transform/map, window and
gain, mono renewal, native stereo analysis, relation/space, DC/Nyquist,
blend/envelope/crop, edge sources, deterministic seed behavior, duplicate /
polarity / anti-phase, swap relationship, channel spectral balance, actual
allocation bound, and forbidden mechanisms. Require exactly `15/15` once.

The `9` synthetic owners retain IDs `Y01` through `Y09`, sources, estimators,
PaulX measurements, support mapping, thresholds, and exact range ownership
from the same brief. Rename only their prefix. `Y02` must complete the full
finite PaulX-relative pitch diagnostic without a comparator ceiling. `Y08`
uses complete output for discontinuity and only complete `H`-sample windows
inside mapped authored support for dropout. `Y09` retains duplicate, relation,
spectral balance, and all `space` rows; mapped long-form stereo policy is owned
only by the later stereo gate. Require exactly `9/9` once.

Every owner renders all rows before one final assertion. Every candidate row
uses `ADMISSION_SEED`. No helper accepts an implicit or local seed.

## Mono Listening Gate

After objective admission, repeat the retained concealed mono pack:
percussion, bass, vocals, pads/sustains, and full mix at `4x`, `8x`, and `16x`
against PaulXStretch 1.6.0 default / FFT `16384`. Use `ADMISSION_SEED`,
`space=0.5`, exact crop, common row RMS, and `0.95` peak ceiling.

Pass requires no unusable row, preferred or tied on at least `12/15`, no
family loss, and no forbidden vocoder, periodic, cyclic, doubled-attack,
stutter, or freeze character. The scorecard must explicitly record
low-frequency noise/haze, entry energy, and tail energy. Batch 31.36's `15/15`
mono receipt is architecture evidence, not a waiver for the fresh candidate.

## Creative Stereo Admission

Use the exact retained stereo originals whose mid downmixes produced `M001`
through `M005`:

- `0000-drums_percussion-000002.wav`
- `0004-bass-000236.wav`
- `0008-vocals-000010.wav`
- `0012-pads_sustains-000423.wav`
- `0016-full_mix-000144.wav`

Render all five at `4x`, `8x`, and `16x`, and at `space=0`, `0.5`, and `1`.
Capture PaulXStretch 1.6.0 default / FFT `16384` from the same originals.
Exact-crop every file. Neutral A/B uses `space=0.5`; source, candidate, and
PaulX share one row RMS target under peak `0.95`.

Hard objective stereo controls are:

- finite samples, exact target length, deterministic repeat, and no clipping
- the structural duplicate, mono, common-polarity, anti-phase, swap,
  per-bin-magnitude, and non-decreasing side-energy rules
- candidate-source whole-render and `0..250`, `250..1500`, and
  `1500..Nyquist` balance error at most `0.75 dB` for every `space`
- balance spread across the three `space` renders at most `0.50 dB`
- no whole-render or band channel-dominance reversal when source balance
  magnitude is at least `0.50 dB`

Mapped-window diagnostics use `4`-second output windows with `2`-second hops.
Map each edge through the sole source/output map. Omit a window only when both
source channels are below `-60 dBFS` RMS; omit a final clipped window shorter
than `2` seconds. Record candidate-source and PaulX-source balance error and
dominance for every eligible window. Missing, non-finite, or incomplete
diagnostics reject the evidence receipt. No numeric mapped-window error or
dominance threshold rejects this creative candidate.

After hard objective admission, the operator may reject during speaker
pre-screen. Promotion then requires one eligible independent stereo listener;
the operator's one-ear hearing cannot satisfy this pass.

Concealed neutral review covers the `15` `space=0.5` candidate/PaulX rows.
Pass requires no unusable candidate row, preferred or tied on at least
`12/15`, and no source family losing every ratio. The listener must score
centre stability, width, pumping, one-sided texture, channel echo,
low-frequency image/noise, entry energy, tail energy, and musical usefulness.

The same listener reviews each candidate `space=0`, `0.5`, `1` trio. Every
trio must move in the declared preserve-to-widen direction without an image
jump, unrelated channel motion, low-frequency pull, or unusable setting.
Ambiguous or unavailable independent review blocks promotion; it does not
become a pass.

## Rejection, Cleanup, And Minimal Admission

Any terminal construction, objective, mono-listening, speaker, or independent
stereo miss rejects the complete candidate. A large but finite mapped-window
diagnostic does not. Record one dominant cause and stopped gate. Delete the
worktree, branch, checkpoint, module, tests, build state, and listening
assembly. Do not tune, repair, or rerun a failed checkpoint.

Only a complete pass may authorize a later admission batch for the private
module, fixed-ratio neutral-`Dream` request and renderer,
structural/synthetic regressions, hard stereo regression owners, diagnostic
schema, and one internal creative-engine version. Batch 31.39 does not merge
the candidate to `main`. Do not admit public character, motion, detail, cache,
artifact, report, route, pitch, dynamic ratio, other character, router,
seed/reroll control, Loophole, or Chorus integration. Product seed variation
requires a later frozen multi-seed character review.

Batch 31.39 outcome: fresh checkpoint
`c0cd943f5a5e8499540d5e759aac7a1586579d0a` passed compile, construction
`1/1`, and structural `15/15`. Synthetic admission finished `7/9`. `Y04`
failed the `16x` impulse replica row with a second region at
`-29.801787859 dB`; `Y09` failed linked-stereo swap at `4x` and `8x`.
Listening did not open. Cleanup deleted the candidate without repair or
rerun, and no candidate code entered `main`.

Batch 31.36 passed both failed owners under the nominally same renderer and
seed. Do not infer a parameter fix or authorize another implementation from
the conflicting receipts. Reconcile their evidence authority first.

## Sources

- [Rejected support-audited predecessor](./offline-creative-verified-source-relative-renewal-spectral-brief.md)
- [Creative source triangulation](../research/specimen-dossiers/creative-stretch-source-triangulation.md)
- [Batch 31.36 stereo rejection](../logs/2026-07/21-g10-031-support-audited-renewal-stereo-rejection.md)
- [Batch 31.37 stereo-ownership closure](../logs/2026-07/21-g10-031-renewal-stereo-ownership-closure.md)
- [Batch 31.39 rejection](../logs/2026-07/21-g10-031-comparator-audited-renewal-rejection.md)
- [Creative product contract](../contracts/085-creative-time-stretch-product-and-routing-contract.md)

## Next Task

Run `g10.031` Batch 31.40 only as docs and evidence-authority reassessment.
Reconcile the contradictory Batch 31.36 and Batch 31.39 synthetic receipts.
Do not recover deleted code, implement DSP, change a formula or gate, rerun a
candidate, expose product surfaces, start another character, or push.
