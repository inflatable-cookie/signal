# Offline Creative DirectRenewalDream Renderer Brief

Status: private fixed-ratio renderer admitted
Owner: dsp
Updated: 2026-07-22
Contract: `085`
Roadmap: `g10.031`, Batch 31.67

## Decision

Build one fresh Signal-owned `DirectRenewalDream` candidate for fixed-ratio
neutral `Dream` at exact `4x`, `8x`, and `16x` expansion.

This is the operator-authorized product reset from Batch 31.64. It does not
reverse a historical checkpoint or recover deleted source. The renderer keeps
the direct PaulX-like mechanism that already reached accepted mono sound and
replaces mismatched creative vetoes with hard integrity plus listening
authority.

The complete renderer is:

- one long-window native-channel analysis
- one exact sample-centred source map
- independent stochastic rotation for every output frame and positive bin
- one shared linked-stereo rotation plus symmetric `space` decorrelation
- one compensated adjacent-frame blend
- a short entry guard, longer terminal release, and exact target crop
- bounded deterministic offline state

There is no phase propagation, peak tracking, transient detector, material
separator, source alignment search, recurrence, limiter, or post-render gain.

## Supported Request

The private request contains exactly:

- finite mono or interleaved stereo `input: &[f32]`
- `channels` equal to `1` or `2`
- `sample_rate` from `8000` through `192000`
- exact `target_frames`
- explicit `seed: u64`
- finite `space` in `[0,1]`

Let source frames be `L` and target frames be `T`. Require checked
`T` equal to `4L`, `8L`, or `16L`, plus `L<=2^53-1` and `T<=2^53-1`. Empty
input with `T=0` returns empty. Every other empty, zero-target, partial-frame,
non-finite, unsupported, ratio-miss, size-overflow, or allocation-overflow
request fails before output allocation. Values are rejected, never clamped.

The only entry is
`render(CandidateRequest<'_>) -> Result<Vec<f32>, CandidateError>`. The closed
private error enum owns request, size, allocation, and non-finite-processing
failure.

Only fixed-ratio neutral `Dream` exists. `motion`, `detail`, dynamic ratio,
pitch, reverse, other characters, routing, cache, artifacts, realtime use, and
public exposure are absent.

## Transform And Source Map

For sample rate `F`, use:

`N=clamp(nearest_power_of_two(round_half_up(2F/3)),8192,131072)`

Power-of-two distance ties select the larger value. `H=N/2`. At `44.1` and
`48 kHz`, `N=32768` and `H=16384`.

Use periodic Hann analysis:

`w[n]=0.5-0.5*cos(2*pi*n/N)`, `0<=n<N`

with energy gain:

`G=sqrt(N/sum(w[n]^2))`.

Output block `j` begins at `y_j=jH`. Let `B=ceil(T/H)`. Analysis/inverse
frames are `j=0..B`; output blocks are `j=0..B-1` and the last block is cropped
to `T`.

The sole source centre is:

`x_j=((y_j+0.5)L/T)-0.5`.

Evaluate numerator `(2y_j+1)L` and denominator `2T` with checked `u128`,
convert once to `f64`, then subtract `0.5`. `x_j` is strictly increasing for
non-empty input. One `FrameSchedule` computes it once for every linked channel.

For analysis index `n`, read:

`p=x_j+n-(N-1)/2`.

Let `i=floor(p)` and `u=p-i`. Four-point cubic Lagrange interpolation over
`i-1..i+2` uses:

- `c_-1=-u(u-1)(u-2)/6`
- `c_0=(u+1)(u-1)(u-2)/2`
- `c_1=-(u+1)u(u-2)/2`
- `c_2=(u+1)u(u-1)/6`

Samples outside `[0,L)` are exact positive zero. Multiply the interpolated
sample by `w[n]`, cast once to `f32`, and transform with `rustfft` `Complex32`.
No other source cursor, event position, or frame schedule exists.

## Deterministic Renewal

Use wrapping `u64` `mix64`:

1. `z=(z xor (z>>30))*0xBF58476D1CE4E5B9`
2. `z=(z xor (z>>27))*0x94D049BB133111EB`
3. return `z xor (z>>31)`

For frame `j`, positive bin `b`, and stream tag `s`:

`z=mix64(seed xor mix64(j xor FRAME) xor rotl(mix64(b xor BIN),21) xor s)`.

Little-endian tags are:

| Tag | Value |
| --- | --- |
| `FRAME = RNWFRAME` | `0x454d415246574e52` |
| `BIN = RNWBIN00` | `0x30304e4942574e52` |
| `BASE = RNWBASE0` | `0x3045534142574e52` |
| `SPACE = RNWSPACE` | `0x4543415053574e52` |
| `TEST = RNWTEST0` | `0x3054534554574e52` |

Convert the high `53` bits with `u=(z>>11)/2^53`, then
`phase(z)=2*pi*u-pi`.

`ADMISSION_SEED=0x0123456789abcdef`. Construction owns the existing exact
`mix64(0)`, `mix64(1)`, and `mix64(u64::MAX)` vectors plus these two complete
addresses at `j=7`, `b=11`:

| Stream | final `z` | high-53 numerator |
| --- | --- | --- |
| `BASE` | `0xaefa063073d5e350` | `0x0015df40c60e7abc` |
| `SPACE` | `0x4afa52608a58dffb` | `0x00095f4a4c114b1b` |

No mutable random generator exists. Frame, bin, stream, and seed fully own
every phase draw.

## Mono And Linked-Stereo Spectra

Retain each native channel's complex positive-frequency coefficient `X_c`.
Input phase is never propagated between output frames. Current-frame complex
phase is retained only so one common stochastic rotation preserves the linked
channel relationship.

For mono positive non-Nyquist bin:

`Y=X*exp(i*theta)`

where `theta=phase(address(BASE))`.

For stereo, let `zeta=phase(address(SPACE))` and bin frequency `f=bF/N`.
Define `h=0` at and below `250 Hz`, `h=1` at and above `1500 Hz`, and
`h=t^2(3-2t)` between them for `t=(f-250)/1250`. Then:

- `d=0.5*space*h*zeta`
- `Y_L=X_L*exp(i*(theta-d))`
- `Y_R=X_R*exp(i*(theta+d))`

Exact-zero bins stay zero. DC retains the real input coefficient. Nyquist is
also retained as its real input coefficient. Negative bins are conjugate
mirrors.

This law owns:

- native channel magnitudes for every bin at every `space`
- exact current-frame interchannel relation at `space=0`
- exact zero and common-polarity behavior
- symmetric, frequency-protected decorrelation as `space` increases
- one shared source schedule and random address field

Exact rendered swap, anti-phase, and duplicate algebra at non-zero `space` are
diagnostics, not product invariants. Centre stability, image usefulness, and
monotonic perceived widening remain terminal through independent listening.

## Synthesis And Boundaries

Inverse-transform with `1/N` scale and no synthesis window. Let real inverse
frame be `z_c[j,n]`. For `0<=n<H`:

- `u=(n+0.5)/H`
- `a=0.5+0.5*cos(pi*u)`
- `b=1-a`
- `c=1/sqrt(a^2+b^2)`
- `q_c[j,n]=G*c*(a*z_c[j,H+n]+b*z_c[j+1,n])`

Cache exactly two adjacent inverse frames per channel. There is no OLA
denominator, adaptive gain, limiter, compressor, or normalization pass.

The entry guard is intentionally short:

`E_head=min(ceil(F/200),floor(T/4))`.

The release is intentionally longer:

`E_tail=min(2H,floor(T/4))`.

For an extent `E>=2`, head factor is
`sin((pi/2)y/(E-1))` and tail factor is
`sin((pi/2)(T-1-y)/(E-1))`. An extent of one sets its sole endpoint to zero;
an extent of zero applies no factor. Multiply overlapping factors. Emit exactly
`[0,T)`, canonicalize exact-zero endpoints, and never append, wrap, reflect,
resize-fill, or repair.

The asymmetric envelope directly owns the accepted operator target: solid
entry, longer release. The listening receipt records both energy regions.

There is no transient state machine. A long magnitude view deliberately smears
attacks. Separated audible replicas, micro-echo, stutter, static freeze, or
clicks reject through the frozen gates below.

## State, Determinism, And Cost

Allocate output, window, interpolation buffer, two native spectra, two inverse
frames per channel, FFT plans, and scratch before frame zero. Reuse all working
storage. Excluding borrowed input and returned output capacity, actual peak
working state is at most `32 MiB` for stereo at `N<=131072` and independent of
duration. No allocation or reallocation occurs after processing starts.

Mono performs one forward and inverse FFT per new frame. Stereo performs two
of each. Cost is `O((T/H)N log N)`. Traversal is single-threaded, reductions use
fixed order, phase is counter-addressed, and ties are explicit. Identical
requests are byte-identical on the frozen platform/toolchain identity.

Offline only. No audio-thread source fill, execution, synchronization, I/O, or
allocation is authorized.

## Candidate Isolation

Batch 31.66 starts from the exact Batch 31.65 closeout commit and creates only:

- worktree: `signal-candidate-31-66`
- branch: `candidate/g10-031-direct-renewal-dream`
- module: `crates/signal-dsp-stretch/src/creative_direct_renewal_dream/`
- files: `mod.rs`, `plan.rs`, `analysis.rs`, `stereo.rs`, `synthesis.rs`,
  `tests.rs`
- tracked ledger: `candidate-evidence/g10-031/31-66/conformance.tsv`
- tracked runner config: `.config/nextest.toml`
- ignored evidence root: `target/creative-stretch-direct-renewal-31-66/`
- acoustic ref:
  `refs/signal-evidence/creative/direct-renewal/31-66-acoustic`

The isolated `lib.rs` may declare the module privately. Existing dependencies
only. No old candidate file, commit, helper, output, or checkpoint may be
recovered. No public API, production tier, feature, report, binary, fixture,
cache, artifact schema, route, Loophole, or Chorus change is allowed.

Test prefixes are exactly:

- `direct_renewal_dream_construction_`
- `direct_renewal_dream_structural_`
- `direct_renewal_dream_synthetic_`

## Executable Evidence Authority

`tests.rs` owns compile-linked `RENDER_SPEC`, `EVIDENCE_SPEC`, `MEMORY_SPEC`,
and `RUN_SPEC` values. No helper contains a second seed, source endpoint,
threshold, ratio, source hash, comparator hash, row count, render count,
assertion mask, or receipt field.

One `GATE_OWNERS` table contains exactly `15` entries: `S01..S10` and
`Y01..Y05`. Every entry contains ID, exact test name, function pointer, ordered
row slice, row count, render count, worst-case output frames, assertion mask,
receipt-field mask, and `600 s` owner deadline. Construction verifies every
field and calls every non-rendering oracle against the renderer authority.

Frozen owner shape:

| ID | Owner suffix | Rows | Renders | Boundary |
| --- | --- | ---: | ---: | --- |
| S01 | `request_preallocation` | 16 | 2 | valid/invalid matrix; rejection before output allocation |
| S02 | `transform_map` | 30 | 0 | rates, geometry, counts, exact rational map, monotonicity |
| S03 | `window_interpolation_gain` | 18 | 0 | Hann, cubic coefficients, exterior zero, gain |
| S04 | `counter_mono_spectrum` | 20 | 0 | tags, vectors, addresses, mono rotation, DC/Nyquist/Hermitian |
| S05 | `linked_stereo_space` | 24 | 0 | common rotation, magnitude, relation, frequency weight, symmetric decorrelation |
| S06 | `blend_boundary_crop` | 18 | 18 | frame halves, compensation, entry/release, endpoints, exact crop |
| S07 | `edge_silence_matrix` | 24 | 24 | silence, one-sample, sub/exact/over-window, DC, impulse, tone |
| S08 | `determinism_seed` | 3 | 3 | byte repeat, active changed seed, finite output |
| S09 | `allocation_memory` | 4 | 2 | actual peak, plan duration independence, no processing allocation |
| S10 | `single_timeline_private_surface` | 12 | 0 | one schedule; forbidden state, mechanisms, dependencies, exposure |
| Y01 | `integrity_crest_discontinuity` | 30 | 30 | all mono sources and ratios; hard integrity, diagnostic crest |
| Y02 | `pitch_diagnostic` | 21 | 9 | two tones and five chord partials at all ratios |
| Y03 | `impulse_diagnostic` | 6 | 6 | impulse/train spread, placement, region inventory |
| Y04 | `periodicity_modulation_gap` | 9 | 9 | uniform noise, mid tone, silence gap at all ratios |
| Y05 | `linked_stereo_inventory` | 22 | 22 | exact frozen relation/ratio/space matrix |

Structural totals are `169` rows and `49` renders. Synthetic totals are `88`
rows and `76` renders. `Y01` owns at most `26,880,000` output frames; `Y05`
owns at most `18,816,000` stereo output frames. No owner may add a row or render
implicitly.

Structural row slices are exact cross-products, in the written order:

- `S01`: empty success; valid `L=4096` mono `4x` and stereo `16x`; channels
  `0` and `3`; one partial stereo frame; rates `7999` and `192001`; one input
  containing both non-finite classes; `space=NaN`, `-f32::EPSILON`, and
  `next_up(1)`; empty with non-zero target; non-empty with zero target; `5x`;
  and the direct dimension oracle at `L=2^53`. Only the two materialized valid
  requests render.
- `S02`: geometry at rates `8000`, `11025`, `44100`, `48000`, `96000`, and
  `192000`; `round_half_up` below/exact half and nearest-power tie-to-larger
  helper rows; block counts at three ratios; exact map values at
  `j=0,1,floor(B/2),B-1,B` for all ratios; monotonicity at all ratios. Map rows
  use `F=48000`, `L=96000`.
- `S03`: Hann at `n=0,1,N/4,N/2,N-1`; gain at `F=8000,48000,192000`; all
  four cubic coefficients plus coefficient sum at
  `u=0,0.25,0.5,0.75,next_down(1)`; integer-node reproduction for each of the
  four source taps; and one exterior-zero interpolation row.
- `S04`: three `mix64` vectors; five tag values; two complete addresses; two
  high-53/phase conversions; zero, active, DC, and Nyquist mono bins; two
  Hermitian mirror rows; active-seed change; and frame/bin address separation.
- `S05`: frequency weight at `0`, `250`, `875`, `1500`, and Nyquist Hz;
  decorrelation at `h=0,0.5,1`; duplicate, common-negated, anti-phase, and
  swapped complex fixtures at `space=0,1`; active-bin left/right magnitude at
  `space=0,0.5,1`; duplicate, negation, and swap relation at `space=0`; and
  side-energy direction over `0->0.5` and `0.5->1`.
- `S06`: ratios `4x,8x,16x` crossed with source lengths
  `1,H-1,H,H+1,2H,2H+1` at `F=48000`. Every render asserts the frozen blend,
  compensation, envelope, endpoint, block count, and crop rules.
- `S07`: exact silence, one-sample DC, sub-window DC, sub-window impulse,
  exact-window DC, exact-window tone, over-window impulse, and over-window
  tone, crossed with all ratios at `F=48000`.
- `S08`: byte-identical repeat, active changed-seed inequality, and finite
  extreme-seed output, each on the same `L=4096`, `8x`, mono tone request.
- `S09`: plan-accounting rows for mono `N=8192` and stereo `N=131072`, then
  instrumented stereo `4x` renders at `L=N` and `L=8N`. The renders prove no
  processing allocation; both duration rows prove identical planned capacity.
- `S10`: one source/static owner each for the absence of phase propagation,
  peak tracking, transient state, material separation, source alignment,
  recurrence, limiter, post-render gain, public module/API, feature flag,
  binary/report mode, and dependency change.

`next_up(1)` and `next_down(1)` mean the adjacent IEEE-754 `f32` values. The
direct dimension oracle executes the same checked validation function as the
public candidate entry without materializing the impossible slice.

The tracked nextest profile is exactly:

```toml
[profile.direct-renewal]
retries = 0
fail-fast = false
test-threads = 1
slow-timeout = { period = "300s", terminate-after = 2 }
failure-output = "immediate-final"
success-output = "never"
status-level = "all"
final-status-level = "all"
```

The tracked `conformance.tsv` is a static owner/row manifest and is never
written by a test. Each invocation gets a fresh ignored receipt directory
named by checkpoint commit, stage, round, and owner. Every owner traverses its
frozen row slice serially. After each row it appends one canonical JSON line,
flushes, and `sync_all`s its owner receipt. Keys, in order, are `schema`,
`checkpoint`, `stage`, `round`, `owner`, `row_index`, `row_id`, `status`,
`render_count`, `output_frames`, `input_sha256`, `output_sha256`, `assertions`,
and `diagnostics`. Arrays preserve spec order; numeric values use finite
decimal strings or `null` when the field mask excludes them. `status` is only
`pass` or `fail`. A terminal summary with expected/complete row and render
counts is written and synced only after every row completes. Timeout or process
loss therefore retains all completed rows. Audio stays under the ignored
evidence root; receipts contain hashes, not embedded samples.

### Conformance and checkpoint

From one clean local commit, run:

1. `effigy test cargo-nextest -- -P direct-renewal -r --no-run -E 'test(/^direct_renewal_dream_/)'`
2. `effigy test cargo-nextest -- -P direct-renewal -r -j 1 --no-fail-fast --failure-output immediate-final -E 'test(/^direct_renewal_dream_construction_/)'`; require `1/1`
3. `effigy test cargo-nextest -- -P direct-renewal -r -j 1 --no-fail-fast --failure-output immediate-final -E 'test(/^direct_renewal_dream_structural_/)'`; require `10/10`

Repeat all three unchanged from the same clean tree and require the same
counts. Record commit/tree, every candidate and evidence SHA-256,
`RENDER_SPEC`, `EVIDENCE_SPEC`, `MEMORY_SPEC`, `RUN_SPEC`, `Cargo.lock`,
`rustc -vV`, Effigy/nextest versions, OS, architecture, and ledger SHA-256.
Create the acoustic ref directly at that commit.

Compiler, type, visibility, ownership, allocation, and exact-conformance fixes
may iterate before checkpoint only when authority in this brief already fixes
the answer. Any new DSP formula, scalar, source, helper algorithm, metric,
threshold, assertion, comparator, or listening choice stops for docs-level
reassessment. No acoustic owner or inspectable audio may run before the ref.

After the ref, run `Y01` through `Y05` individually in numeric order with the
same profile, `-r`, `-j 1`, failure capture, and exact-name filter. Require one
selected test and one pass each. Stop later owners on failure. Every acoustic
stage uses only the ref.

## Synthetic Sources

Use `F=48000`, `L=96000`, and `T=4L`, `8L`, or `16L`. Sustained support is
`[24000,72000)`. The first and last `2048` supported samples use complementary
half-cosine entry/exit ramps; the interior weight is one and exterior is zero.

Sources are:

- low tone: `0.5 sin(2*pi*110n/F)`
- mid tone: `0.5 sin(2*pi*440n/F)`
- chord: amplitude `0.1` at `110`, `164.813778`, `220`, `277.182631`, and
  `329.627557 Hz`
- harmonic pad: partials `k=1..8` at `110k Hz`, amplitude `0.35/k`
- impulse: `1` at `48000`
- impulse train: `1,-0.8,0.65,-0.5` at `19200,38937,58103,77797`
- silence gap: harmonic pad with exact zero over `[42000,54000)`
- uniform noise: `0.5*(2*((mix64(n xor TEST)>>11)/2^53)-1)`
- Rademacher noise: `+0.5` for high bit one, otherwise `-0.5`
- amplitude-modulated noise: Rademacher times
  `0.5+0.375 sin(2*pi*1.7n/F)`

The ten little-endian `f32` input SHA-256 values are:

| Source | SHA-256 |
| --- | --- |
| amplitude-modulated noise | `ba6b9c244618939769e7283fac92f198690238db0c96d99c280892ee358ab31b` |
| chord | `b7c85b6faed8d670fd7eefa66f7be6a89df0f7c5c3a4444146a2d083a70792e7` |
| harmonic pad | `732895709a05fa724d9dd76a03bc22c64865b84ba93ba351e49354b31f95e96c` |
| impulse train | `47314d3121745479660fb0d0350b41aec987074f75a503181805e5f4545e8138` |
| impulse | `fc73433e0fab2786572b6a98bd0cc9f86145960581d77e3dbc7d1bfa6abca57b` |
| low tone | `2c6d1c766ce73ac75000f8e9cbd6238fafbf180c64baa33d62a55fb9517f32e1` |
| mid tone | `36397e016a1d00a5bf1884d049a1454ab7342965ffd3cf21179474610a218b33` |
| Rademacher noise | `c1ae606691767937990e38a314ceadeee6c7cb0a9da63c7ed3d3a3ef31b838b5` |
| silence gap | `1c17fdc3cecd09cfcc403c39a9c7aadb75c41239433c20863cb967fbcef0013e` |
| uniform noise | `cde1917d6afdfe3dfb260da2a6273a243e261032bb9fe6624e49020089ee9923` |

Stereo fixtures are duplicate mid tone; exact common negation; anti-phase mid
tone; pad with a zero-padded `37`-sample right delay; and mixed stereo with
left chord plus `0.2` uniform noise and right delayed chord minus the same
noise.

`Y05` renders duplicate at all ratios and `space=0,0.5,1` (`9` rows); base and
common-negated at `8x/0.5` (`2`); mixed and swapped mixed at `8x/0.5` (`2`);
anti-phase at `8x` and all spaces (`3`); delayed pad at all ratios and
`space=0,1` (`6`). Total: `22` rows/renders.

## Synthetic Pass And Diagnostics

Hard for every synthetic output:

- exact `T`, finite samples, deterministic repeat where named, exact-zero
  first/last sample, and no output sample above absolute `8`
- no exact-zero run of `H` complete frames wholly inside mapped authored
  non-zero support
- uniform-noise autocorrelation and uniform/mid block-RMS CV no greater than
  matching PaulX plus `0.05`
- complete row receipt with all required hashes and fields

First-difference reference values at `4x/8x/16x` are:

| Source | `4x` | `8x` | `16x` |
| --- | ---: | ---: | ---: |
| low tone | `9.905726` | `10.894208` | `10.519063` |
| mid tone | `9.556341` | `10.436868` | `10.905863` |
| chord | `11.802457` | `12.936199` | `12.552822` |
| harmonic pad | `11.915677` | `13.552276` | `14.040544` |
| impulse | `21.672820` | `21.668892` | `21.489501` |
| impulse train | `16.312956` | `16.540906` | `17.186745` |
| silence gap | `13.453803` | `15.084147` | `15.790229` |
| uniform noise | `14.905336` | `15.539239` | `15.680264` |
| Rademacher noise | `14.348783` | `15.440176` | `15.456703` |
| amplitude-modulated noise | `16.083822` | `16.292147` | `16.221062` |

Uniform autocorrelation references are `0.017218163`, `0.017727693`, and
`0.017090511`. Uniform block-RMS CV references are `0.387747959`,
`0.460013282`, and `0.492971808`; mid-tone references are `0.617268653`,
`0.679139581`, and `0.708639858`.

Mandatory finite diagnostics, never numeric vetoes:

- first-difference crest and delta from the frozen PaulX value for all `Y01`
  rows
- pitch error for `110`, `440`, and all five chord partials using the central
  half of mapped support, periodic Hann, at least `8x` zero-padding, `+/-4 Hz`
  search, and three-bin log-magnitude parabolic interpolation
- impulse `95%` shortest-energy width, centroid error against
  `(48000.5)T/L-0.5`, active-region count, and every secondary-region level
- silence-gap RMS, low-frequency `0..80 Hz` energy ratio, exact swap/polarity/
  anti-phase/duplicate residuals, and `space` side-energy direction

Impulse active regions use `480`-frame RMS windows, `240`-frame hop, `-30 dB`
relative activity, and `2400`-frame region separation. The previous exact
one-region result is diagnostic only.

For every `Y05` row, hard stereo controls are exact length/finiteness, native
per-bin magnitude at synthesis, source relation at `space=0`, candidate-source
whole and `0..250`, `250..1500`, `1500..Nyquist` balance error at most
`0.75 dB`, balance spread across complete `space` trios at most `0.50 dB`, and
no dominance reversal when source balance magnitude is at least `0.50 dB`.
Time-domain algebra and local image remain diagnostics.

## Long-Form Mono Gate

Only after `Y01..Y05` pass, render the five retained `44.1 kHz`, `220500`-
frame sources at `4x`, `8x`, and `16x`. Mono is samplewise
`f32((f64(L)+f64(R))/2)`. Use `ADMISSION_SEED`, `space=0.5`, exact crop, and
the retained PaulXStretch `1.6.0` default / FFT `16384` comparator.

| Family/file | Source SHA-256 | PaulX `4x / 8x / 16x` SHA-256 |
| --- | --- | --- |
| percussion `0000-drums_percussion-000002.wav` | `89e55b28c6ed36e26bf73f2024d301aaeedd07cca30b315f5530051c28f4e1e7` | `57d00180a71d6bdc31d96f9d064c10d669505aef9755a9a06a50d11f74ba0fa6` / `63f244af10e645efefb74371050608c1ecc8b8cc1b1148f1d8bb231a97166c97` / `bb2f505859358ddefcd81f0b8492188020f1f7a826c08308ad7e80df95b8c1ab` |
| bass `0004-bass-000236.wav` | `3d587007a5d9a683e82e14a184530a9a0f953e58fbc2fe3712a42aa86ecf9ad8` | `6f4a2d9cad0fc83c367e30af8bea1a88b3d982c627fbf4994059374cf2d5f148` / `5af21debcef116c3ebbc6d99c901874076e26d62db3c63d069256a0612bffb5a` / `56f0016ae0c3ac4f72b6fde81d60f56e4142b5bfe4f14f036fbd066a19fd6516` |
| vocals `0008-vocals-000010.wav` | `3d74a686dccf6dcdfedc57e0fc2b76a0d29374ba28afa1bb497172cc441f7ee9` | `3a33b35f5080c315c5a78ceee3666f7b313629e1c2a28b121b8e6332549ece4f` / `442f03cb03432f3e1ccea11c4f13fb542ce811a69aff6b21e34ca18a2d0d58ea` / `922fe84bf48e242e5cf00f891fd61a8d84735f96fa05452ce8e3fafc0608007a` |
| pads `0012-pads_sustains-000423.wav` | `a736a0e04ade9e879db954069c8c0b68842bd4d364eec69e62dfef1447763131` | `2a403fb5bc6bc7359b6dd1111b6899c4c269d575fc958dd896e1396503360718` / `c9408a43f1a7478afa40b31998c5b7615860456558db0749167f12467aa07d22` / `1fd1416e9fa5771076654da07d4c0fcc4accf80dd66c44cf5e53518b2cca0465` |
| full mix `0016-full_mix-000144.wav` | `caa5d0d7c51bc7e2d537d3d13dbe32055f0c2032c69bd8e3f28a38df96fafbf1` | `198230814079c57561aa09908875734daa14ac04c0cc8c612d35f8696024265d` / `56a48397e8dfa43bdbf6f100bd273802786d9a260e21b275760c27c8b0eb9174` / `c643146fe667731f7eeb77964a1d834bba1423940c64cf2516815c46230dba65` |

The little-endian decoded-mono source SHA-256 values in the same family order
are:

| Family | Decoded mono SHA-256 |
| --- | --- |
| percussion | `da20fe2d616be4f2e589b46390cd85ecff3c323a870bd35d36d3432fbc0ad68f` |
| bass | `7bf03e3ba0c3ab0c197c881a106767d46f26a1d6a11bf84449b2f500aa2f0687` |
| vocals | `64c96ca0ba72778def43940d963e5467e0c62f48e429cead53d7f08841bfc417` |
| pads | `5ac7f7b3650b02f2ed160d682c56b0359d8972c8b8f77d671552b57e56da8a2e` |
| full mix | `637b60f3b0e0230f871d48b804507a1388ece6570abcfa58d4a74610ea84c96b` |

The retained PaulX files are stereo containers created from those mono
sources. Verify the full-file hashes above, keep the first exact `T` frames,
then downmix with the same samplewise formula. The cropped decoded-mono
SHA-256 values at `4x / 8x / 16x` are:

| Family | Cropped PaulX decoded-mono SHA-256 |
| --- | --- |
| percussion | `8aa2323537f7b39de586877b863f2ea1b31c24112ded4335437991cb40247ded` / `bca4af496b5d0b332653c1330bfbeefb63a443685e3ebd32ca453489b706e196` / `4b291a8fbd5b5b543a6693be3a0cfc2c0e32222743fe4dc608e5d81b8d13f547` |
| bass | `3d1ff31ed417ff1df9c7acb82847eff75c3fce244b0b1d73c9946a566fa9f2a1` / `bf7503f4448aaa41a987b2bb6a92fb130290e09ff0c37067c894116eda3fd3ff` / `e46a7903fe5eaf2f81d4b13f6e0c6407937a899bbd7ffc381ea43aba646081cf` |
| vocals | `fd42bde9c14e2fa520f0ae708052117fd98ed4dac36705fb10e003635ff2dd8e` / `a1b02a866602b096c4b5ad3a18350107a7325eae14cb3712f2701646a5700ad9` / `00b3e9583fb15ef38c6ecaedfb07afb1775292f62cd969287a630d724a79e44c` |
| pads | `5e04f47e7bc899705873079c33d4861b5ce0eea46d8c411099878c0d4ca4574f` / `b5a5d0af9f49b36a317920b9c6675786acb34a33fefe140a73e839410fd8a9cd` / `9d2c340cfc5a5acebe262493c668d74547a8e5f2f4b20c1412ad145dd074febe` |
| full mix | `40ba471c9ad0a4f4198ce771091e844e4d582696ca1d9098a78bd5d88a4edf12` / `71fe8f07f0229f81fa7709bde40d256a067ac7e9260d0b414bde7f0d3d3b4357` / `42b6828b052c8d10fae5cccd24ee76af57c048963e07e52edae56bd64b14a962` |

Within each row, source, candidate, and PaulX share one RMS target reduced only
to keep peak at or below `0.95`. Conceal A/B identity. Review all five `8x`
rows first, then the remaining ten.

Pass requires no unusable candidate row, candidate preferred or tied on at
least `12/15`, and no source family losing every ratio. Exposed vocoder colour,
rough periodicity, cyclic repetition, audible replica, doubled attack,
micro-echo, stutter, static freeze, arbitrary loudness, or click rejects.

The scorecard records smoothness, musical usefulness, tonal focus/ringing,
transient softness/spike, low-frequency noise, event placement, width, entry
energy over the first `5%`, and tail energy over the last `5%`. Metrics cannot
waive listening.

## Independent Stereo Gate

Use the same five original source hashes. Render all five sources at all three
ratios and `space=0,0.5,1`: `45` candidate renders. The `15` retained PaulX
files above are the neutral comparators. Exact-crop and level-match as in mono.

Hard objective controls are exact length, finiteness, deterministic repeat,
whole/three-band balance within `0.75 dB`, trio balance spread within
`0.50 dB`, and no dominance reversal above the `0.50 dB` source floor.

Four-second windows with two-second hops record complete candidate-source and
PaulX-source balance, dominance, correlation, width, `0..80 Hz` energy, entry
energy, and tail energy. Omit a window only when both mapped source channels
are below `-60 dBFS` RMS or the final clipped window is shorter than two
seconds. Missing/non-finite evidence rejects; finite local error is diagnostic.

The operator may reject during speaker pre-screen. Promotion requires an
eligible independent listener. Concealed `space=0.5` review passes only with no
unusable row, preferred or tied on at least `12/15`, and no family losing every
ratio. Every `space` trio must move preserve-to-widen without an image jump,
unrelated channel motion, low-frequency pull, balance shift, or unusable
setting. Missing or ambiguous independent review blocks promotion.

## Rejection, Cleanup, And Minimal Admission

Any hard objective, mono-listening, speaker, or independent-stereo miss rejects
the checkpoint. A finite diagnostic alone does not. Record the dominant cause,
stopped gate, complete persisted rows, and exact checkpoint identity. Do not
tune, repair, or rerun after acoustic identity.

Delete the worktree, branch, build state, private module, test/config surface,
and rendered audio after the decision. Retain the local evidence ref through
one required docs-only reassessment, then delete it when that evidence question
closes. Nothing from a rejected candidate enters `main`.

Only a complete pass may open a separate minimal-admission batch for:

- private `creative_direct_renewal_dream`
- fixed-ratio neutral-`Dream` request and renderer
- structural/synthetic regression owners and diagnostic receipt schema
- one internal creative-engine version

Do not admit public character/seed controls, `motion`, `detail`, cache,
artifact/report surfaces, dynamic ratio, other characters, routing, Loophole,
or Chorus. Multi-seed character review and product integration remain later
decisions.

Two fresh acoustic checkpoints failing the same dominant cause close this
architecture for reassessment. No scalar, seed, window, phase, threshold, or
assertion sweep follows a failure.

## Batch 31.66 Outcome

Checkpoint `760da32d2c87b2838bda48f32af90ae4ae51f8d9` implemented this brief
without changing its renderer formulas.

- two unchanged clean conformance rounds passed compile, construction `1/1`,
  and structural `10/10`
- `Y01..Y05` passed `88/88` synthetic rows and `76/76` renders
- concealed mono passed as `15/15` usable ties; L001 and L006 retained a
  non-terminal slower-entry/faster-tail caveat
- long-form stereo passed all `45` hard rows, all `15` trio-spread rows, and
  all `1400` mapped diagnostics
- worst whole/three-band balance error was `0.138039758 dB`; worst `space`
  trio spread was `0.064465954 dB`
- the largest finite local image diagnostic was `9.253537932 dB` on the
  `16x` full-mix tail; PaulX measured `8.219403964 dB` in the same region

The operator found the stereo effect satisfactory on speakers, reported no
detectable stereo issue, and explicitly waived eligible independent review
for this effect. Contract `085` records this as a scoped creative-product
decision for this checkpoint. The operator's one-ear hearing limitation is
not erased, and no independent-listening claim is made.

The candidate passes fixed-ratio admission. This outcome opens only the
minimal private production surface below; it does not admit routing, dynamic
ratio, public controls, other characters, cache behavior, Loophole, or Chorus.

## Batch 31.67 Admission

The private renderer, fixed-ratio request, closed error boundary, regression
owners, diagnostic receipt schemas, and internal engine version
`signal-creative-direct-renewal-dream-v1` now compile in normal Signal builds.
The module remains private and unrouted.

Analysis, plan, stereo, and synthesis source are byte-identical to checkpoint
`760da32d`. Integrated construction passed `1/1`, structural passed `10/10`,
and `Y01..Y05` passed `88/88` rows with `76/76` renders. Synthetic hashes,
assertions, and diagnostics match the checkpoint row-for-row after excluding
checkpoint, stage, and round labels.

No candidate evidence directory, nextest profile, listening audio, public API,
route, cache identity, dynamic ratio, other character, Loophole, or Chorus
surface was admitted.

## Sources

- [Direct-renewal owner study](./offline-creative-direct-renewal-owner-study.md)
- [Creative source triangulation](../research/specimen-dossiers/creative-stretch-source-triangulation.md)
- [Creative product contract](../contracts/085-creative-time-stretch-product-and-routing-contract.md)
- [PaulXStretch pinned core](https://github.com/essej/paulxstretch/blob/8ec191fdd7203354c79391cbc04c9fd83fa30ea0/Source/PS_Source/Stretch.cpp)

## Next Task

Run Batch 31.68 only. Reassess the paused `2x..4x` coherent/`Dream` overlap as
docs and architecture work. Either freeze one complete shared-map blend
architecture or retain the pause. Do not change DSP or product surfaces.
