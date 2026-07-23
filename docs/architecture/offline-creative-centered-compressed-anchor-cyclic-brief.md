# Offline Creative CenteredCompressedAnchorCyclic Renderer Brief

Status: frozen at immutable acoustic ref; acoustic admission unrun
Owner: dsp
Updated: 2026-07-23
Contract: `085`, Rule 11
Roadmap: `g10.032`, Batch 32.4
Behavior:
[Offline Creative Cyclic Behavioral Synthesis](./offline-creative-cyclic-behavioral-synthesis.md)

## Decision

Build one isolated `CenteredCompressedAnchorCyclic` candidate for explicit
fixed-cycle `Cyclic` expansion above `1x` through `8x`.

It is one direct sample-domain renderer:

- one exact rational source/output map
- one fixed render-wide cycle clock
- two neighbouring forward unit-rate source reads per output sample
- one complementary raised-cosine crossfade
- one independently auditable commanded-replica ledger
- one schedule shared by every linked channel
- exact direct output evaluation with no synthesis tail
- duration-independent state and deterministic scalar traversal

No grain list, free-running cursor, integer repeat/jump clock, similarity
search, pitch detector, transient detector, phase vocoder, spectral correction,
randomness, feedback, post-gain, limiter, or boundary repair exists.

This is clean-room Signal architecture. Potenza and SickoCV establish mechanism
families only. Signal copies no GPL expression, constant, threshold, table, or
control flow. ReaReaRea supplies behavioral measurements and listening
comparison only.

## Prior Candidate Boundary

`CyclicGrain` and `SimilarityAlignedCyclic` remain rejected and deleted.
Nothing from either candidate is recovered.

This brief stays in the broad compressed-anchor family because Batch 32.2
proved that ReaReaRea follows that grammar. It changes the complete authority:

- one explicit cycle duration replaces inherited Dream macros
- a measured centred replica law replaces uncalibrated fixed-lattice intent
- direct two-read evaluation replaces rolling grain accumulation
- no seed, `motion`, `detail`, or `space` exists
- comparator-relative creative diagnostics replace the invalid absolute pitch
  veto
- Contract `085` Rule 11 owns construction, checkpoint, receipt, and cleanup

This is the explicit evidence-backed product-gate change allowed by Rule 11.
It does not overturn the old receipt or authorize scalar tuning.

## Supported Request

The private candidate request contains:

- borrowed mono or interleaved linked-stereo finite `f32`
- channel count `C` equal to `1` or `2`
- sample rate `F` from `8,000` through `192,000` Hz
- input frame count `L`
- exact target frame count `T`
- integer cycle duration `cycle_us`

Supported cycle values are `5,000..=90,000` microseconds. The frozen neutral
value is `48,000`; short and long review values are `5,000` and `90,000`.
Values are rejected, never clamped.

Target frames own duration. Use checked integer comparisons before output
allocation:

- empty input is valid only with `T=0`
- non-empty identity requires `T=L`
- expansion requires `L<T<=8L`
- `L` and `T` must not exceed `2^53-1`

Exact `16x`, compression, zero target with non-empty input, non-zero target
with empty input, partial interleaved frames, invalid sample rate, invalid
channels, invalid cycle, non-finite input, arithmetic overflow, and allocation
overflow return a closed private `CandidateError` before candidate output
allocation.

Identity is byte-exact passthrough after complete request validation. It does
not build the cycle plan. The candidate has no independent ratio field.

Dynamic ratio, reverse, transpose, Auto cycle, `INTELL`, seed, `motion`,
`detail`, `space`, cache, artifacts, public product integration,
RealtimePreview, and audio-thread execution are unsupported.

## Cycle Clock

Convert cycle duration once:

`H=floor((F*cycle_us+500000)/1000000)`.

Evaluate with checked `u128`. This is positive half-up rounding. No clamp or
power-of-two conversion follows.

At `44.1 kHz`:

| Cycle | `H` |
| --- | ---: |
| `5 ms` | `221` |
| `48 ms` | `2117` |
| `90 ms` | `3969` |

The neutral value is measurement-backed. ReaReaRea median impulse spacing
`D` gives `H=D/(1-1/q)` of about `2117`, `2117.3`, and `2116.6` frames at
`2x`, `4x`, and `8x`.

`H` is the output launch clock. The effective spacing between adjacent
appearances of one source event is:

`D=H*(T-L)/T=H*(1-1/q)`.

Cycle changes `H`. Ratio changes the compressed source-anchor advance and
therefore `D`. There is no integer repeat count.

## Exact Map And Anchors

For output sample-centre coordinate `y`, the sole ideal map is:

`a(y)=((y+0.5)L/T)-0.5`.

For every integer anchor output `y_k=kH`, represent its source coordinate by:

`den=2T`

`anchor_num(k)=(2y_k+1)L-T`

`x_k=anchor_num(k)/den`.

Use checked signed `i128` numerators and positive `i128` denominator. The
anchor path is strictly increasing. Anchors may lie beyond the target crop;
the same rational map extends them. No anchor is displaced or accumulated.

For output index `y`, define:

`k0=floor(y/H)`

`k1=k0+1`

`r=y-k0H`, where `0<=r<H`.

Only anchors `k0` and `k1` may contribute. For anchor `k`, its unit-rate source
read at `y` is:

`p_num(k,y)=anchor_num(k)+den*(y-y_k)`

`p(k,y)=p_num(k,y)/den`.

Use Euclidean division:

`i=floor(p_num/den)`

`u=(p_num-i*den)/den`, where `0<=u<1`.

The formula, integer division rule, and traversal order are the entire timeline
owner. No floating cursor feeds a later sample.

## Source Read And Crossfade

Read each native channel by linear interpolation:

`s_c(p)=(1-u)input_c[i]+u input_c[i+1]`.

Indices outside `[0,L)` return exact positive zero. Convert source `f32` to
`f64`, evaluate interpolation in `f64`, and do not cast between interpolation
and crossfade.

Build one `H+1` entry `f64` table before processing:

`c[r]=0.5-0.5*cos(pi*r/H)`, for `0<=r<=H`.

Assign `c[0]=0` and `c[H]=1` exactly. For output remainder `r`:

`w1=c[r]`

`w0=1-w1`.

Evaluate channels in ascending order:

`o_c[y]=f32(w0*s_c(p(k0,y))+w1*s_c(p(k1,y)))`.

Weights are non-negative and complementary. The output is a convex combination
of native samples after linear interpolation. A finite input therefore cannot
produce a raw peak above the largest input peak except the frozen `2e-6`
floating tolerance.

The conceptual local support is `2H`, but no grain object or overlap ring
exists. Every target sample receives exactly two scheduled reads, including a
zero-weight read at a cycle boundary.

## Commanded Replica Ledger

For an impulse at source frame `e`, the independent ledger includes output
index `y` and anchor `k` only when:

- `k` is `k0(y)` or `k1(y)`
- the corresponding weight is positive
- `floor(p(k,y))=e` or `floor(p(k,y))+1=e`
- the matching linear-interpolation coefficient is positive

The oracle computes this from checked rational numerators without calling the
renderer interpolation helper.

Hard rules:

- output outside the ledger is at most `1e-7` absolute
- every authored event has at least one positive ledger appearance
- anchor and source-event order remain increasing
- two interior authored events retain ordered weighted ledger centres
- an event whose full theoretical cluster lies inside `[0,T)` has weighted
  centre within `D/2+1` output frame of its ideal mapped centre

Events whose theoretical clusters cross an exterior boundary use the exact
cropped ledger and record the lost leading or trailing appearances. They do
not inherit the interior centre bound.

Record positive appearance count, median and range of spacing, weighted centre,
cluster extent, peak, energy, and matching ReaReaRea values. Those finite
differences diagnose character. They are not sample-matching gates.

No transient copy, attack repair, echo, or residual path may add another
appearance.

## Linked Stereo

Channels share:

- request and cycle plan
- `k0`, `k1`, anchors, rational positions, and interpolation fractions
- weights and traversal order
- ledger and boundary decisions

Only native source samples differ. No mid/side synthesis, channel sum,
dominant-channel analysis, channel-local cycle, random motion, width control,
or post-hoc balance repair exists.

Samplewise hard mechanics use `1e-6` maximum absolute error:

- duplicate input remains duplicate
- anti-phase input remains anti-phase
- common negation commutes
- channel swap commutes

An exact right gain `0x3f004dce` (`0.5011872053146362f32`) retains its
whole-render and three-band
right/left ratio within `0.01 dB`. A right channel delayed by `13` source
samples must retain its strongest interior cross-correlation lag in
`12..=14`; its normalized peak correlation may not fall more than `0.02`
below matching ReaReaRea.

For long-form stereo, whole-render and `0..250`, `250..1500`, and
`1500..Nyquist` candidate-source balance error may not exceed the matching
ReaReaRea-source error by more than `0.50 dB`. Four-second mapped windows with
two-second hops record the same values as diagnostics. Missing or non-finite
evidence rejects.

## Boundaries And Exact Length

Evaluate only output indices `[0,T)` in ascending order. Reads outside source
domain are exact zero. Do not wrap, reflect, clamp to an edge sample, append a
tail, resize-fill, normalize after rendering, or apply an implicit fade.

The first and last output samples may be non-zero only through the same two
scheduled reads as every interior sample. Direct evaluation has no unfinished
overlap state and no terminal crop repair.

Exterior discontinuity includes:

- exterior zero to `o[0]`
- every interior first difference
- `o[T-1]` to exterior zero

After the common row level policy, the candidate exterior-inclusive maximum
first difference may not exceed matching ReaReaRea by more than `1.50 dB`.
Exact silence is governed by exact zero instead.

## Memory, Determinism, And Cost

Renderer-owned working state contains:

- the `H+1` `f64` crossfade table
- one checked plan
- two rational positions and interpolation states
- fixed scalar accumulators

At maximum rate and cycle, `H=17280`; planned working allocation must stay at
or below `256 KiB`, excluding borrowed input and required output. Capacity is
independent of `L` and `T`. No allocation or reallocation occurs after output
processing starts.

Cost is `O(C*T)`. Per output channel/sample the bound is four source loads, two
linear interpolations, two weighted products, one addition, and one final
cast. Table construction is `O(H)`. No FFT, search, analysis pass, event list,
source copy, output-length state, thread, task, lock, I/O, or unbounded work
exists.

Use scalar output-major, channel-minor traversal. Same complete request,
candidate version, supported target, toolchain, and build profile must produce
byte-identical output. The supported deterministic contract is the acoustic
checkpoint's recorded OS, architecture, `rustc -vV`, `Cargo.lock`, build
profile, and candidate source hashes.

Offline execution only. This architecture makes no audio-thread support claim.

## Candidate Isolation

Batch 32.5 starts from the exact Batch 32.4 closeout commit and creates only:

- worktree: `signal-candidate-32-5`
- branch: `candidate/g10-032-centered-compressed-anchor-cyclic`
- module:
  `crates/signal-dsp-stretch/src/creative_centered_compressed_anchor_cyclic/`
- files: `mod.rs`, `plan.rs`, `schedule.rs`, `interpolate.rs`,
  `synthesis.rs`, `evidence.rs`, `tests.rs`
- tracked ledger: `candidate-evidence/g10-032/32-5/conformance.tsv`
- tracked runner config: `.config/nextest.toml`
- ignored evidence root:
  `target/creative-stretch-centered-compressed-anchor-cyclic-32-5/`
- acoustic ref:
  `refs/signal-evidence/creative/centered-compressed-anchor-cyclic/32-5-acoustic`

The isolated `lib.rs` may declare the module privately. Existing dependencies
only. No historical candidate file, commit, helper, output, build state, or
checkpoint may be recovered.

No public API, feature, report mode, binary, fixture family, cache, artifact
schema, product route, runtime DTO, Loophole, or Chorus change is allowed.

Test prefixes are exactly:

- `centered_cyclic_construction_`
- `centered_cyclic_structural_`
- `centered_cyclic_synthetic_`

## Rule 11 Executable Authority

`tests.rs` owns compile-linked `RENDER_SPEC`, `EVIDENCE_SPEC`,
`COMPARATOR_SPEC`, `MEMORY_SPEC`, and `RUN_SPEC` values. No helper contains a
second cycle, source endpoint, ratio, threshold, row, comparator setting,
assertion mask, or receipt field.

One `GATE_OWNERS` table has exactly `15` entries:

| ID | Owner suffix | Rows | Renders | Boundary |
| --- | --- | ---: | ---: | --- |
| S01 | `request_preallocation` | 22 | 4 | valid/invalid matrix; failure before output allocation |
| S02 | `cycle_map` | 42 | 0 | cycle rounding, rational map, anchor monotonicity |
| S03 | `window_interpolation` | 40 | 0 | raised cosine, complements, linear read, exterior zero |
| S04 | `schedule_ledger` | 63 | 0 | two reads, spacing law, independent event ledger |
| S05 | `boundary_crop` | 54 | 54 | short lengths, every ratio/cycle, exact crop |
| S06 | `identity_silence_edges` | 28 | 28 | passthrough, exact silence, active endpoints |
| S07 | `linked_algebra` | 54 | 72 | duplicate, polarity, swap, level, delay |
| S08 | `allocation_determinism` | 20 | 10 | plan bounds, actual peak, byte repeat |
| S09 | `single_timeline_private_surface` | 16 | 0 | forbidden state, mechanisms, dependencies, exposure |
| Y01 | `integrity_discontinuity` | 30 | 30 | ten mono sources at three ratios, neutral cycle |
| Y02 | `pitch_diagnostic` | 36 | 36 | tones/chord/pad, all ratios and cycles |
| Y03 | `replica_diagnostic` | 18 | 18 | impulse/train, all ratios and cycles |
| Y04 | `cadence_modulation` | 27 | 27 | three noise families, all ratios and cycles |
| Y05 | `gap_tail_boundary` | 18 | 18 | gap/pad, all ratios and cycles |
| Y06 | `linked_stereo_inventory` | 54 | 72 | six linked fixtures, all ratios and cycles |

Structural totals are `339` rows and `168` renders. Synthetic totals are
`183` rows and `201` renders. No owner may add a row or render implicitly.
Each owner deadline is `600 s`.

### Structural Rows

`S01` contains four successes: empty, mono identity, stereo `2x`, and mono
`8x`. Its eighteen failures are channels `0` and `3`, partial stereo, rates
`7999` and `192001`, `NaN`, positive infinity, negative infinity, cycles
`4999` and `90001`, empty/non-zero target, non-empty/zero target, compression,
`8L+1`, exact `16x`, direct `L>2^53-1`, direct `T>2^53-1`, and direct
allocation overflow.

`S02` contains `12` cycle conversions from rates `8000`, `44100`, `48000`,
and `192000` crossed with `5`, `48`, and `90 ms`; `12` map rows from four
output positions at each ratio; `15` anchor rows from indices
`0`, `1`, `2`, `last`, and `last+1` at each ratio; and three complete
monotonicity rows.

`S03` uses `H=240`, `2304`, and `4320` at `48 kHz`. It evaluates `c` at
`0`, `H/4`, `H/2`, `3H/4`, and `H`, then complement pairs at
`0`, `1`, `H/4`, `H/2`, and `H-1` for every `H`. Ten interpolation rows cover
`u=0`, `0.25`, `0.5`, `0.75`, `next_down(1)`, integer-node reproduction,
negative exterior, right exterior, coefficient sum, and peak convexity.

`S04` proves exactly two scheduled anchors at output positions
`0`, `H-1`, `H`, `H+1`, and `T-1` across all ratios and cycles (`45` rows).
Nine rows prove `D=H(T-L)/T`. Nine default-cycle rows cover ledger events
`e=0`, `48000`, and `95999` at every ratio.

`S05` uses `F=48000` and source lengths
`1`, `H-1`, `H`, `H+1`, `2H`, and `2H+1`, crossed with every ratio and cycle.
Each deterministic source is `0.25` with `+0.5` added at its centre.

`S06` contains mono/stereo identity at lengths `0`, `1`, and `4096`; exact
mono/stereo silence at every ratio and cycle; and mono constant-`0.25` edge
rows at ratios `2x` and `8x` with cycles `5` and `90 ms`.

`S07` crosses duplicate, anti-phase, common-negated pair, swapped pair, exact
`-6 dB` right level, and right-delay-`13` fixtures with every ratio and cycle.
Pair fixtures render both transformations, giving `72` renders.

`S08` has eight planned-capacity rows from rates `8000` and `192000`, cycles
`5` and `90 ms`, and both layouts; two instrumented short/long stereo renders;
four mono/stereo byte-repeat rows at `2x` and `8x`; and six direct size,
capacity-independence, no-processing-allocation, and maximum-table oracles.

`S09` proves the absence of a free cursor, grain list, repeat counter,
similarity search, detector, FFT, stochastic state, feedback, post-gain,
limiter, width stage, public module/API, feature, binary/report mode, new
dependency, and audio-thread claim.

### Receipt And Runner

The tracked nextest profile is:

```toml
[profile.centered-cyclic]
retries = 0
fail-fast = false
test-threads = 1
slow-timeout = { period = "300s", terminate-after = 2 }
failure-output = "immediate-final"
success-output = "never"
status-level = "all"
final-status-level = "all"
```

`conformance.tsv` is static and never test-written. Every invocation gets a
fresh ignored receipt directory named by commit, stage, round, and owner.
Rows execute serially. After each row, append one canonical JSON line, flush,
and `sync_all`. Keys in order are:

`schema`, `checkpoint`, `stage`, `round`, `owner`, `row_index`, `row_id`,
`status`, `render_count`, `output_frames`, `cycle_us`, `input_sha256`,
`comparator_sha256`, `output_sha256`, `assertions`, `diagnostics`.

Arrays retain specification order. Finite numbers are decimal strings.
Excluded fields are `null`. `status` is `pass` or `fail`. Write and sync the
terminal expected/completed row/render summary only after the last row.

Construction verifies every owner, function pointer, ordered row slice, row
and render count, maximum output, deadline, assertion mask, receipt mask,
source formula, source/container hash where already retained, comparator
request, and execution boundary. It calls every non-rendering oracle.

### Conformance And Checkpoint

From one clean local commit:

1. compile all three test prefixes without running
2. run exactly one construction test; require `1/1`
3. run `S01..S09` serially; require `9/9`

Use the tracked profile, release build, one test thread, no retry, no
fail-fast, and immediate final failure output. Repeat all three steps unchanged
from the same clean tree and require identical counts and receipt hashes.

Record commit/tree, candidate and evidence file SHA-256 values, all five spec
values, `Cargo.lock`, `rustc -vV`, Effigy/nextest versions, OS, architecture,
build profile, comparator manifest, and ledger SHA-256. Create the acoustic
ref directly at that commit.

Before the ref, corrections are allowed only when this brief already fixes the
answer. A new DSP formula, scalar, source, helper, metric, threshold,
assertion, comparator, listening rule, or cleanup choice stops for docs-level
reassessment. No candidate acoustic owner or candidate audio may run before
the ref.

Actual checkpoint:

- commit `4600d228286797d22e4f4d5ca4efa997835fc4b2`
- tree `fa1fc8031a4aab4302b778474702e658784d8a64`
- ref
  `refs/signal-evidence/creative/centered-compressed-anchor-cyclic/32-5-acoustic`
- release compile `2/2`, construction `1/1` twice, structural `9/9` twice
- byte-identical `S01..S09` receipt hashes across both rounds
- no `Y01..Y06` owner or candidate listening render run

## Synthetic Source Authority

Use `F=44100`, `L=88200`, and `T=2L`, `4L`, or `8L`. Sustained sources are
active over `[22050,66150)`. Their first and last `2048` active samples use
complementary half-cosine ramps; the interior weight is one and exterior is
zero.

Use wrapping `u64` `mix64`:

1. `z=(z xor (z>>30))*0xBF58476D1CE4E5B9`
2. `z=(z xor (z>>27))*0x94D049BB133111EB`
3. return `z xor (z>>31)`

The test tag is `0x3054534554574e52`. Sources are:

- low tone: `0.5 sin(2*pi*110n/F)`
- high tone: `0.5 sin(2*pi*1760n/F)`
- chord: amplitude `0.1` at `110`, `164.813778`, `220`, `277.182631`, and
  `329.627557 Hz`
- harmonic pad: partials `k=1..8` at `110k Hz`, amplitude `0.35/k`
- impulse: `1` at `44100`
- impulse train: `1,-0.8,0.65,-0.5` at
  `17640,35787,53383,71437`
- silence gap: harmonic pad with exact zero over `[38588,49613)`
- uniform noise:
  `0.5*(2*((mix64(n xor TEST)>>11)/2^53)-1)`
- Rademacher noise: `+0.5` for high bit one, otherwise `-0.5`
- amplitude-modulated noise: Rademacher times
  `0.5+0.375 sin(2*pi*1.7n/F)`

Evaluate formula arithmetic in `f64`, apply the support ramp, then cast once
to little-endian `f32`. Construction independently regenerates every source,
records its SHA-256, and proves formula samples at all discontinuities and
`n=0`, `1`, `22050`, `44100`, `66149`, and `88199`.

Stereo fixtures use amplitude-modulated noise as their base:

- duplicate
- anti-phase
- base/common-negated pair
- mixed pair and channel-swapped pair
- exact right gain `0x3f004dce` (`0.5011872053146362f32`)
- right channel delayed `13` source frames with exterior zero

## Synthetic Gate

After the acoustic ref, run `Y01` through `Y06` individually in numeric order.
Every owner runs from the ref and stops later owners on hard failure.

Hard for every output:

- exact `T`, finite samples, and complete receipt
- raw peak no greater than input peak plus `2e-6`
- exact-zero silence
- no complete `5 ms` output RMS window below `-80 dBFS` where the mapped
  source window is above `-40 dBFS`
- exterior-inclusive first-difference result within the comparator rule
- every event appearance inside the independent ledger
- all linked mechanics within their frozen bounds

Missing or non-finite diagnostics reject evidence.

`Y02` records strongest tone component over one octave around the authored
frequency. For every chord/pad partial, record the strongest component within
`+-15%`. Use the mapped active hull, periodic Hann, next power-of-two
zero-padding to at least `262144`, magnitude spectrum, and three-point
log-magnitude parabolic interpolation. Record input-relative cents,
ReaReaRea-relative cents, and estimator-bin resolution. No finite pitch value
is a character rejection.

`Y03` applies the hard ledger rules and records appearance count, spacing,
centre, extent, peak, energy, and comparator delta per event.

`Y04` uses `2048`-sample RMS envelopes with `512`-sample hops. Record strongest
non-DC modulation peaks, sidebands around authored components, envelope
autocorrelation, and the exact planned `D`. Higher cycle must increase planned
and measured event spacing. Finite modulation strength is diagnostic.

`Y05` records mapped gap support, unexpected dropout count, exterior
first-difference, last active frame, inactive tail, and comparator delta.

`Y06` applies all linked rules and records whole/three-band balance,
cross-correlation lag and peak, local mapped-window balance, width, and
candidate/ReaReaRea deltas.

## Comparator Authority

REAPER `7.69/macOS-arm64` mode `983040` is the primary comparator. Projects use
`44.1 kHz`, preserve pitch, item play rate `1/q`, the retained mode field
`0.0025`, zero-length item fade-in, `10 ms` item fade-out, exact item length
`qL/F`, and `24-bit` stereo WAV output. Only source path, play rate, item
length, and render name vary.

The five retained stereo sources are exactly `220500` frames:

| Family/file | Source SHA-256 |
| --- | --- |
| percussion `0000-drums_percussion-000002.wav` | `89e55b28c6ed36e26bf73f2024d301aaeedd07cca30b315f5530051c28f4e1e7` |
| bass `0004-bass-000236.wav` | `3d587007a5d9a683e82e14a184530a9a0f953e58fbc2fe3712a42aa86ecf9ad8` |
| vocals `0008-vocals-000010.wav` | `3d74a686dccf6dcdfedc57e0fc2b76a0d29374ba28afa1bb497172cc441f7ee9` |
| pads `0012-pads_sustains-000423.wav` | `a736a0e04ade9e879db954069c8c0b68842bd4d364eec69e62dfef1447763131` |
| full mix `0016-full_mix-000144.wav` | `caa5d0d7c51bc7e2d537d3d13dbe32055f0c2032c69bd8e3f28a38df96fafbf1` |

Retained ReaReaRea container hashes:

| Family | `4x` | `8x` |
| --- | --- | --- |
| percussion | `37db73f3b67b43eba491de1c084fe7d40429ed449f631d9107276b574661d90b` | `356723fe7d953ef119558590f30682f25de47f521be98c62a746fb69698a0c0a` |
| bass | `618830e45b3deb5ca07609c00d5cb7b8cb65b4cef64f242d47180e26e790a922` | `3fd429b76b95c41b5c869a6f9df646ec8cd2181c5fa3423272a596003c5dada6` |
| vocals | `af180619222408364022edda7fdb76819a8398a0e39da8bf0287448a808f154f` | `0ae69d4cf473f255c277047d39406ff089b42abc0ab747e8eeb973cf7e9e2764` |
| pads | `269b81b489df287e4dcb868f2966983da9d5c7cd03bb2c3e5ff7fe349d1c7660` | `77eb6b8f2314521543c6d4a161fa74b84a4af8b33bda706f3f5ee4f830a04f79` |
| full mix | `e07dc195460332fd8cd4088e1447de3added2122ed7c108418157b99cd6bcfae` | `64f8c450e400c589ac249a717d6c27750b7cf8aef41f501b1666643214fc3ce6` |

Recreate the five missing `2x` rows and matching synthetic/stereo rows before
the acoustic ref. This is comparator preparation, not candidate execution.
Bind each output hash, source hash, project hash, REAPER identity, row order,
and the historical `48`-row group receipt
`5bb7b55456065d8f3d69c7229abc117eacb9280cf298a779b634598a19663e11`
into `COMPARATOR_SPEC`. A retained `4x` or `8x` mismatch, incomplete `2x`
capture, missing synthetic row, or changed project field stops before the ref.

Comparator audio remains ignored. No external production dependency enters
Signal.

## Long-Form Mono Listening

Only after every synthetic owner passes, downmix the five original sources and
ReaReaRea renders samplewise:

`mono=f32((f64(left)+f64(right))/2)`.

Render candidate neutral `48 ms` at `2x`, `4x`, and `8x`: `15` rows. Render
short `5 ms` and long `90 ms` directions for the same matrix: `30` more
candidate rows.

Within each neutral A/B row, source, candidate, and comparator share one RMS
target reduced only to keep every peak at or below `0.95`. Apply the
comparator's exact `10 ms` fade-out to the listening copy of the candidate,
never to raw objective evidence. Conceal identity.

Review neutral `8x`, then `4x`, then `2x`. Pass requires:

- recognizable Cyclic character on at least `12/15`
- musically useful output on at least `10/15`
- candidate preferred or tied on at least `10/15`
- preferred or tied on at least `7/10` primary `4x`/`8x` rows
- no unusable row
- no source family losing all three ratios
- no uncontrolled click, dropout, arbitrary level step, source-obscuring buzz,
  lost event, or doubled attack outside the commanded grammar

Only after neutral passes, compare short/neutral/long candidate trios. Short
must move recognizably toward metallic/ring-like motion on at least `12/15`;
long must move toward tremolo/echo-like motion on at least `12/15`. Each
direction must be musically useful on at least `10/15`. Any inversion or
unusable endpoint rejects.

Operator listening is promotion authority. Objective proximity cannot waive a
failed row.

## Independent Stereo Listening

Only after mono passes, render the five retained stereo originals at neutral
`48 ms` and all three ratios. Hard mechanics and comparator-relative
whole/three-band balance run before listening. Local mapped-window values stay
diagnostic.

The operator may reject during speaker pre-screen. Promotion requires one
eligible independent listener. Review the `15` concealed candidate/comparator
rows and the synthetic stable-centre, wide-sustain, transient, delay,
unequal-level, and anti-phase fixtures.

Pass requires:

- no hard relation or balance failure
- no unusable row
- candidate preferred or tied on at least `10/15`
- no family losing every ratio
- no centre pull, one-sided texture, width pumping, side inversion, detached
  echo, channel-local cycle, or arbitrary balance movement

The scoped Dream stereo waiver does not apply. Missing or ambiguous independent
review blocks promotion.

## Gate Order

From the same acoustic ref:

1. `Y01..Y06` synthetic and linked integrity
2. exact `16x` typed rejection on all five retained sources
3. concealed long-form mono neutral review
4. short/neutral/long cycle-direction review
5. hard long-form stereo objectives
6. operator speaker pre-screen
7. eligible independent stereo review
8. fixed-ratio promotion decision

Stop later gates on the first hard or listening failure. No code, source,
formula, constant, metric, threshold, comparator, or helper changes after the
ref.

## Rejection, Cleanup, And Minimal Admission

On failure, record the dominant cause, stopped gate, complete persisted rows,
checkpoint commit/tree, ref, source/comparator/candidate hashes, and receipt
hashes. A finite diagnostic alone cannot reject.

Delete:

- isolated worktree and branch
- candidate module and tests
- tracked candidate ledger/config
- build state and generated candidate/comparator copies
- local evidence ref after the required docs-only reassessment

No rejected constant, helper, fixture, report mode, hidden API, partial
mechanism, or threshold enters `main`. No tuning or rerun follows an acoustic
failure. Another candidate requires the Contract `085` reassessment rule.

Only a complete pass may open a separate admission batch for:

- the single private `creative_centered_compressed_anchor_cyclic` family
- its private exact-target, manual-cycle request and renderer
- the structural/synthetic regression owners required to preserve admission
- one frozen Cyclic engine-version identifier

Do not admit public `Cyclic`, Auto, `INTELL`, routing, cache, artifacts,
dynamic ratio, pitch, reverse, seed, Dream controls, runtime integration,
Loophole, or Chorus.

## Next Task

Execute `g10.032` Batch 32.6 only from the immutable acoustic ref. Run
`Y01..Y06` individually in numeric order and stop on the first hard failure.
Run exact `16x` typed rejection only after all six owners pass. Do not change
candidate code or evidence authority.
