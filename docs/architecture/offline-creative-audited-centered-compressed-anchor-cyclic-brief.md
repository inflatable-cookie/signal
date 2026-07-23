# Offline Creative AuditedCenteredCompressedAnchorCyclic Brief

Status: frozen; Batch 32.9 ready
Owner: dsp
Updated: 2026-07-23
Contracts: `046`, `085` Rule 11
Roadmap: `g10.032`, Batch 32.8
Behavior:
[Offline Creative Cyclic Behavioral Synthesis](./offline-creative-cyclic-behavioral-synthesis.md)

## Decision

Build one fresh isolated `AuditedCenteredCompressedAnchorCyclic` candidate.
The renderer is byte-for-byte architectural continuity with the
evidence-invalid `CenteredCompressedAnchorCyclic` checkpoint:

- one exact rational source/output map
- one fixed render-wide cycle clock
- two neighbouring forward unit-rate source reads per output sample
- one complementary raised-cosine crossfade
- one independently computed commanded-replica ledger
- one schedule shared by linked channels
- direct exact-length evaluation with exterior zero
- duration-independent state and deterministic scalar traversal

No DSP formula, scalar, source, metric, threshold, or listening policy is
derived from the unreceipted `Y01` output. The old checkpoint is rejected and
deleted. Its source, tests, helpers, objects, branch, worktree, build state,
and ref may not be recovered.

This fresh identity changes only executable evidence ownership. Every
structural, synthetic, rejection, render, and listening row is its own
one-shot process and append-only receipt. A process persists `started` before
loading audio or invoking the renderer. It persists `pass`, `fail`, or
`panic` before nextest reports the result. A timeout or external kill leaves a
durable incomplete receipt and rejects the checkpoint.

This is the one fresh audited identity allowed by Contract `085` Rule 11. A
second incomplete-evidence checkpoint closes the centred compressed-anchor
identity as protocol churn.

## Clean-Room And Scope Boundary

This is clean-room Signal architecture. Public source establishes mechanism
families only. Signal copies no external implementation expression, constant,
threshold, table, mask, or control flow. ReaReaRea is a behavioral and
listening comparator, not a production dependency.

The candidate is private, offline, whole-buffer, and fixed-ratio. It does not
change:

- public API, feature flags, report modes, or binaries
- CreativeStretch routing, cache, or artifact contracts
- the admitted `Dream` renderer
- transparent stretch or RealtimePreview
- runtime DTOs, Loophole, or Chorus

No grain list, free cursor, repeat counter, similarity search, pitch or
transient detector, FFT, phase vocoder, stochastic state, feedback, post-gain,
limiter, width stage, tail repair, or automatic cycle selector exists.

## Supported Request

The private request contains:

- borrowed mono or interleaved linked-stereo finite `f32`
- channel count `C` equal to `1` or `2`
- sample rate `F` in `8,000..=192,000` Hz
- input frame count `L`
- exact target frame count `T`
- integer `cycle_us` in `5,000..=90,000`

The neutral cycle is `48,000 us`. Direction review uses `5,000 us` and
`90,000 us`. Invalid values reject; nothing clamps.

Checked validation occurs before output allocation:

- empty input is valid only with `T=0`
- non-empty identity requires `T=L`
- expansion requires `L<T<=8L`
- `L` and `T` must not exceed `2^53-1`

Exact `16x`, compression, invalid empty/target pairs, partial interleaved
frames, invalid rate or channels, invalid cycle, non-finite input, arithmetic
overflow, and allocation overflow return a closed private `CandidateError`
before candidate output allocation.

Identity is byte-exact passthrough after full validation. It does not create a
cycle plan. Target frames are the sole duration owner; no independent ratio
field exists.

Dynamic ratio, reverse, transpose, Auto, `INTELL`, seed, `motion`, `detail`,
`space`, cache, artifacts, public routing, audio-thread execution, and
RealtimePreview are unsupported.

## Complete Renderer

### Cycle Clock

Convert duration once with checked `u128`:

`H=floor((F*cycle_us+500000)/1000000)`.

This is positive half-up rounding. No clamp or power-of-two conversion
follows. At `44.1 kHz`, `5`, `48`, and `90 ms` produce `H=221`, `2117`, and
`3969`. At the maximum request, `H=17280`.

`H` is the output launch clock. For ratio `q=T/L`, adjacent appearances of one
source event have planned spacing:

`D=H*(T-L)/T=H*(1-1/q)`.

There is no integer repeat count.

### Map, Anchors, And Reads

The sole ideal map at output sample centre `y` is:

`a(y)=((y+0.5)L/T)-0.5`.

For anchor output `y_k=kH`:

`den=2T`

`anchor_num(k)=(2kH+1)L-T`

`x_k=anchor_num(k)/den`.

Use checked signed `i128` numerators and positive `i128` denominator. The path
is strictly increasing and extends unchanged beyond the target crop.

For output index `y`:

`k0=floor(y/H)`

`k1=k0+1`

`r=y-k0H`, with `0<=r<H`.

Only `k0` and `k1` contribute. Their unit-rate read positions are:

`p_num(k,y)=anchor_num(k)+den*(y-kH)`

`p(k,y)=p_num(k,y)/den`.

Use Euclidean division:

`i=floor(p_num/den)`

`u=(p_num-i*den)/den`, with `0<=u<1`.

For native channel `c`, linear interpolation is:

`s_c(p)=(1-u)input_c[i]+u input_c[i+1]`.

Indices outside `[0,L)` return exact positive zero. Convert input `f32` to
`f64`, evaluate interpolation and crossfade in `f64`, and cast once at output.
No floating cursor owns a later sample.

### Crossfade And Traversal

Build one `H+1` `f64` table:

`c[r]=0.5-0.5*cos(pi*r/H)`, for `0<=r<=H`.

Assign `c[0]=0` and `c[H]=1` exactly. Then:

`w1=c[r]`

`w0=1-w1`

`o_c[y]=f32(w0*s_c(p(k0,y))+w1*s_c(p(k1,y)))`.

Traverse output-major, channel-minor, both ascending. Weights are
non-negative and complementary. Every sample has exactly two scheduled reads,
including a zero-weight read at a cycle boundary. The raw peak cannot exceed
the largest input peak except the frozen `2e-6` floating tolerance.

### Commanded Replica Ledger

The independent oracle includes output `y` for source event `e` and anchor
`k` only when:

- `k` is `k0(y)` or `k1(y)`
- the matching crossfade weight is positive
- `floor(p(k,y))=e` or `floor(p(k,y))+1=e`
- the matching interpolation coefficient is positive

The oracle uses checked rational numerators and may not call the renderer's
position, interpolation, or weight helpers.

Hard rules:

- output outside the ledger is at most `1e-7` absolute
- every authored event has a positive ledger appearance
- anchors and source events remain ordered
- two interior events retain ordered weighted centres
- a fully interior cluster centre is within `D/2+1` output frame of its ideal
  mapped centre

Boundary-crossing clusters use the exact cropped ledger and record lost
leading or trailing appearances. They do not inherit the interior centre
bound.

Receipts record appearance count, spacing median and range, weighted centre,
extent, peak, energy, cropped appearances, and comparator deltas. Finite
character differences diagnose; they do not independently reject.

### Linked Stereo

Both channels share request validation, `H`, anchors, rational positions,
fractions, weights, traversal, ledger, and boundaries. Only native source
samples differ.

Samplewise mechanics have `1e-6` maximum absolute error:

- duplicate remains duplicate
- anti-phase remains anti-phase
- common negation commutes
- channel swap commutes

An exact right gain `0x3f004dce` (`0.5011872053146362f32`) retains whole and
three-band right/left ratio within `0.01 dB`. A right channel delayed by `13`
source frames retains strongest interior cross-correlation lag in `12..=14`;
its normalized peak may not fall more than `0.02` below ReaReaRea.

Long-form whole-render and `0..250`, `250..1500`, and `1500..Nyquist`
candidate-source balance error may not exceed the matching
ReaReaRea-source error by more than `0.50 dB`. Four-second mapped windows with
two-second hops record the same source, candidate, comparator, and delta
values. Missing or non-finite evidence rejects.

### Boundaries, Length, Memory, And Cost

Evaluate exactly `[0,T)`. Exterior reads are exact zero. Do not wrap, reflect,
edge-clamp, append a tail, resize-fill, normalize, or apply an implicit fade.
No terminal repair exists.

Exterior discontinuity includes exterior zero to `o[0]`, every interior first
difference, and `o[T-1]` to exterior zero. After the common level policy, the
candidate maximum may not exceed matching ReaReaRea by more than `1.50 dB`.
Exact silence remains exact zero.

Renderer working state is the `H+1` table, one checked plan, two rational
positions, and fixed scalar accumulators. It is at most `256 KiB`, excluding
borrowed input and required output, and independent of `L` and `T`. No
allocation or reallocation occurs after sample processing begins.

Cost is `O(H+C*T)`: at most four source loads, two linear interpolations, two
weighted products, one addition, and one final cast per channel/sample. No
analysis pass, source copy, output-length state, search, FFT, thread, task,
lock, or I/O exists.

The same request, checkpoint, target, toolchain, build profile, OS, and
architecture produces byte-identical output. The checkpoint records all of
those identities plus `Cargo.lock`.

## Fresh Isolation

Batch 32.9 starts from the exact Batch 32.8 closeout commit and creates only:

- worktree: `/Users/tom/Dev/projects/signal-candidate-32-9`
- branch:
  `candidate/g10-032-audited-centered-compressed-anchor-cyclic`
- private module:
  `crates/signal-dsp-stretch/src/creative_audited_centered_compressed_anchor_cyclic/`
- module files: `mod.rs`, `plan.rs`, `schedule.rs`, `interpolate.rs`,
  `synthesis.rs`, `evidence.rs`, `tests.rs`
- tracked evidence authority:
  `candidate-evidence/g10-032/32-9/`
- tracked nextest config: `.config/nextest.toml`
- ignored build/evidence root:
  `target/creative-stretch-audited-centered-compressed-anchor-cyclic-32-9/`
- acoustic ref:
  `refs/signal-evidence/creative/audited-centered-compressed-anchor-cyclic/32-9-acoustic`

Existing dependencies only. The isolated `lib.rs` may declare the module
privately. No rejected file, helper, commit, branch, ref, object, output,
receipt, build artifact, or generated comparator copy may be recovered.

The tracked evidence directory contains:

- `row-manifest.tsv`
- `comparator.tsv`
- `run-audited-centered-cyclic.sh`
- immutable source/comparator recipes and hash expectations

Generated audio and receipts stay ignored. The runner is candidate-local
evidence scaffolding, not a product surface.

Test names use only:

- `audited_centered_cyclic_construction_row`
- `audited_centered_cyclic_structural_row`
- `audited_centered_cyclic_synthetic_row`
- `audited_centered_cyclic_exact16_row`
- `audited_centered_cyclic_listening_row`
- `audited_centered_cyclic_summary`

An environment row ID selects exactly one manifest row. A test process that
would select zero or multiple rows fails before renderer execution.

## One-Shot Row Protocol

Every row is a separate nextest invocation and OS process. Owners do not loop
over multiple DSP rows. The tracked runner follows manifest order and stops
after the first non-pass terminal state.

Before source or comparator load, output allocation, candidate render, or
metric work, the wrapper:

1. creates `{receipt_root}/{row_id}.jsonl` with `create_new`
2. writes the canonical `started` record
3. calls `flush` and `sync_all`
4. runs the row as `Result` inside `catch_unwind`
5. writes one terminal `pass`, `fail`, or `panic` record
6. calls `flush` and `sync_all`
7. only then returns success or causes the test process to fail

The row body contains no assertion panic or `unwrap`. All expected failures
are `Result` values. The wrapper converts unexpected panic payloads to one
escaped error string after the durable `panic` record is written.

A timeout, signal, crash, or kill leaves only `started`. That is terminal
evidence failure. It is never rerun under the same identity. Existing row
receipt files block execution, including prior partial files. Receipt roots
are unique by checkpoint, phase, and gate; no test deletes or truncates them.

The fresh nextest profile is:

```toml
[profile.audited-centered-cyclic]
retries = 0
fail-fast = true
test-threads = 1
slow-timeout = { period = "300s", terminate-after = 2 }
failure-output = "immediate-final"
success-output = "never"
status-level = "all"
final-status-level = "all"
```

Each nextest invocation is also bounded externally by the runner at `600 s`.
Any test-runner or shell retry is prohibited.

## Canonical Receipt

Each JSON line uses UTF-8, one trailing newline, no insignificant whitespace,
and the following key order:

1. `schema`
2. `identity`
3. `checkpoint`
4. `tree`
5. `phase`
6. `row_id`
7. `sequence`
8. `status`
9. `source_id`
10. `ratio_num`
11. `ratio_den`
12. `target_frames`
13. `cycle_us`
14. `channels`
15. `planned_render_count`
16. `completed_render_count`
17. `output_frames`
18. `input_container_sha256`
19. `input_pcm_sha256`
20. `comparator_container_sha256`
21. `comparator_pcm_sha256`
22. `output_pcm_sha256`
23. `artifact_sha256`
24. `assertions`
25. `diagnostics`
26. `error`

`schema` is `signal.audited-centered-cyclic.row.v1`; `identity` is
`AuditedCenteredCompressedAnchorCyclic`. Excluded fields are JSON `null`.
Hashes are lowercase hex SHA-256. PCM hashes cover little-endian interleaved
`f32` frames after decode or before deterministic WAV encoding. Container
hashes cover exact files.

`assertions` is an ordered array of objects with exact keys `id`, `status`,
`expected`, and `actual`. `diagnostics` is an ordered array with exact keys
`id`, `value`, and `unit`. Finite numbers are locale-independent decimal
strings. Every hard assertion in the manifest appears exactly once in the
terminal record. No generic assertion label is allowed.

Terminal assertion status is exactly `pass`, `fail`, or `not_run`. Assertions
after the first failure remain present as `not_run` with null actual value.
Terminal diagnostics retain every manifest ID; a value not reached after a
hard failure is null. A terminal `pass` requires every assertion to pass and
every mandatory diagnostic value to be present and finite.

The `started` record uses `sequence=0`, `status=started`, zero completed
renders, null result hashes, empty assertion/diagnostic arrays, and null
error. The terminal record uses `sequence=1`. `pass` has null error; `fail`
and `panic` have a non-empty escaped error.

A dedicated JSON string escaper owns quote, reverse-solidus, control
characters, and `U+0000..U+001F`. Construction exercises every escape class.
No error or path is interpolated into JSON without it.

## Construction Proof

Construction is one non-DSP row and must pass before conformance. It verifies:

- exact module, file, dependency, and private-surface inventory
- exact renderer constants, formulas, types, error variants, and state bounds
- exact ordered manifest, row IDs, row counts, render counts, and phase map
- exact assertion and diagnostic IDs for every row
- exact source formulas, discontinuity samples, container hashes, and PCM
  hashes
- exact comparator requests, project fields, row order, and hashes
- exact long-form renderer, level-match, fade, concealment, pack, decision,
  and summary executors
- exact nextest profile, runner order, timeout, no-retry, and stop behavior
- actual row-dispatch functions, not only names, masks, or pointers

Without invoking DSP, it then runs sentinel row bodies through the real
receipt wrapper:

- one `Result` failure persists `started` plus terminal `fail`
- one panic persists `started` plus terminal `panic`
- one child process enters the real wrapper, acknowledges its synced
  `started` record through a separate sentinel file, is killed by the
  construction parent, and leaves only `started`
- a second open of every sentinel row is blocked by `create_new`
- each file has exact schema, key order, sequence, status, assertion map,
  diagnostics map, newline, and receipt SHA-256

Sentinel files use a construction-only ignored receipt root and are never
reused. Construction rejects a wrapper that can reach test failure before
terminal `flush` and `sync_all`.

Construction independently calls all non-rendering oracles. It proves the
worst-case planned output and `600 s` envelope for every row. It does not
render candidate audio.

## Structural Conformance

Structural rows retain the frozen renderer coverage. Each row is one process:

| Gate | Rows | Renders | Ownership |
| --- | ---: | ---: | --- |
| S01 | 22 | 4 | request and pre-allocation rejection |
| S02 | 42 | 0 | cycle, rational map, monotonic anchors |
| S03 | 40 | 0 | crossfade, complement, interpolation, exterior zero |
| S04 | 63 | 0 | two reads, spacing law, independent ledger |
| S05 | 54 | 54 | short lengths and exact crop |
| S06 | 28 | 28 | identity, silence, active edges |
| S07 | 54 | 72 | linked algebra, gain, delay |
| S08 | 20 | 10 | capacity, allocation, determinism |
| S09 | 16 | 0 | single timeline and private surface |

Totals are `339` rows and `168` renders.

`S01` has four successes: empty, mono identity, stereo `2x`, and mono `8x`.
Its failures are channels `0` and `3`, partial stereo, rates `7999` and
`192001`, `NaN`, positive infinity, negative infinity, cycles `4999` and
`90001`, invalid empty/target pairs, compression, `8L+1`, exact `16x`,
direct `L>2^53-1`, direct `T>2^53-1`, and direct allocation overflow.

`S02` crosses rates `8000`, `44100`, `48000`, and `192000` with all three
cycle anchors; four map positions at `2x`, `4x`, and `8x`; anchor indices
`0`, `1`, `2`, `last`, and `last+1`; and three complete monotonicity rows.

`S03` uses `H=240`, `2304`, and `4320` at `48 kHz`. It checks the window at
`0`, `H/4`, `H/2`, `3H/4`, and `H`; complements at `0`, `1`, `H/4`, `H/2`,
and `H-1`; and ten interpolation rows for `u=0`, `0.25`, `0.5`, `0.75`,
`next_down(1)`, integer nodes, both exterior sides, coefficient sum, and peak
convexity.

`S04` checks two anchors at `0`, `H-1`, `H`, `H+1`, and `T-1` across every
ratio/cycle; nine spacing rows; and neutral-cycle ledger events `0`, `48000`,
and `95999` at every ratio.

`S05` uses `F=48000`, source lengths `1`, `H-1`, `H`, `H+1`, `2H`, and
`2H+1`, all ratios and cycles. The source is `0.25` plus `0.5` at its centre.

`S06` covers mono/stereo identity at lengths `0`, `1`, and `4096`; exact
mono/stereo silence at every ratio/cycle; and mono constant-`0.25` edges at
`2x` and `8x`, `5` and `90 ms`.

`S07` crosses duplicate, anti-phase, common-negated pair, swapped pair, exact
right gain, and right-delay-`13` with every ratio/cycle. Pair rows render both
transformations.

`S08` covers planned capacity at minimum/maximum rates and cycles in both
layouts; instrumented short/long stereo; mono/stereo byte repeats at `2x` and
`8x`; direct size, duration independence, no-processing-allocation, and
maximum-table oracles.

`S09` proves absence of every prohibited state/mechanism and every public,
dependency, binary, report, or audio-thread exposure.

Row IDs are the gate plus zero-padded manifest ordinal and semantic slug, for
example `S01-000-empty-success`. The tracked manifest freezes every expansion
and exact order; runtime row generation is prohibited.

## Conformance And Acoustic Ref

Comparator preparation completes before the candidate source checkpoint. From
one clean candidate commit:

1. compile every candidate test without running
2. run construction once; require `1/1`
3. invoke all `339` structural rows separately in manifest order
4. run the structural summary reader
5. repeat steps 1 through 4 unchanged from the same commit and tree

The two rounds use separate ignored directories but receipt field
`phase=conformance`; no round or path appears in receipt content. Every
corresponding row receipt must be byte-identical. The summary reader requires
exactly one `started` and one terminal `pass` per row, validates hashes and
ordered assertion maps, concatenates terminal lines in manifest order, and
records the aggregate SHA-256.

Record checkpoint/tree, source and evidence SHA-256 values, `Cargo.lock`,
`rustc -vV`, Effigy and nextest versions, OS, architecture, build profile,
comparator manifest, receipt aggregates, and all runner exits. Only then
create the acoustic ref directly at that commit.

Before the ref, corrections are allowed only when this brief already fixes the
answer. Any choice or change to DSP, source, scalar, helper algorithm, metric,
threshold, assertion, comparator, listening rule, or cleanup stops for
docs-level reassessment. No synthetic row or candidate listening output runs
before the ref.

## Synthetic Source Authority

Use `F=44100`, `L=88200`, and `T=2L`, `4L`, or `8L`. Sustained sources are
active over `[22050,66150)`. Their first and last `2048` active samples use
complementary half-cosine ramps; interior weight is one and exterior is zero.

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

Evaluate in `f64`, apply support, then cast once to little-endian `f32`.
Construction regenerates every source and proves samples at discontinuities
and `n=0`, `1`, `22050`, `44100`, `66149`, and `88199`.

Stereo fixtures use amplitude-modulated noise:

- duplicate
- anti-phase
- base/common-negated pair
- mixed pair and its channel swap
- exact right gain `0x3f004dce`
- right delay `13` with exterior zero

## Synthetic Gates

Each row is a separate process. Order and counts are:

| Gate | Expansion | Rows | Renders |
| --- | --- | ---: | ---: |
| Y01 | ten mono sources × `2x/4x/8x` × neutral cycle | 30 | 30 |
| Y02 | low/high tone, chord, pad × ratios × all cycles | 36 | 36 |
| Y03 | impulse/train × ratios × all cycles | 18 | 18 |
| Y04 | three noise sources × ratios × all cycles | 27 | 27 |
| Y05 | gap/pad × ratios × all cycles | 18 | 18 |
| Y06 | six stereo fixtures × ratios × all cycles | 54 | 72 |

Totals are `183` rows and `201` renders. Within each gate, order is source,
ratio `2x/4x/8x`, then cycle `5/48/90 ms` where applicable. Pair fixtures
render both transformations in one row. The manifest contains the expanded
rows; the executable may not infer extras.

Hard for every applicable output:

- exact `T`, finite samples, and complete durable receipt
- raw peak at most input peak plus `2e-6`
- exact-zero silence
- no complete `5 ms` output RMS window below `-80 dBFS` while its
  ideal-map source window exceeds `-40 dBFS`
- exterior-inclusive first difference within the comparator `+1.50 dB` rule
- every event appearance inside the independent ledger
- linked mechanics within frozen bounds
- every required diagnostic present and finite

Dropout uses `221` samples at `44.1 kHz`. Candidate output windows map through
the exact ideal-map endpoints. For output window `[b,b+221)`, the source
reference is the ascending integer range
`ceil(a(b))..=floor(a(b+220))`, clipped to `[0,L)`. Source activity is the
`f64` RMS of that exact range. An empty range or incomplete output window is
ineligible. A non-finite RMS rejects.

### Y02 Pitch

Use the mapped active hull, periodic Hann, next-power-of-two zero padding of at
least `262144`, magnitude spectrum, and three-point log-magnitude parabolic
interpolation. Search one octave around each authored tone; for chord/pad,
also record the strongest component within `+-15%` of each partial.

Record input-relative cents, comparator-relative cents, peak magnitude,
interpolated bin, and estimator-bin resolution. Every value must be finite.
Pitch is diagnostic, not an absolute character veto.

### Y03 Replicas

Apply the hard independent-ledger rules. Record per event the appearance
count, spacing median/range, centre, extent, peak, energy, crop loss, and
candidate/comparator delta.

### Y04 Cadence And Modulation

Build an RMS envelope with `2048`-sample windows and `512`-sample hops over the
mapped active hull. Subtract its mean, apply a periodic Hann, and zero-pad its
FFT to the next power of two at least `16384`.

Search `0.1..=20 Hz` for the strongest non-DC envelope component. For
amplitude-modulated noise, also search `1.45..=1.95 Hz` and record the nearest
and strongest components around authored `1.7 Hz`. Normalize envelope
autocorrelation by zero-lag energy and search lags
`round(D/512)-2..=round(D/512)+2`, clipped to positive valid lags.

Record planned `D`, strongest frequency and strength, authored sideband
frequency and strength, autocorrelation lag/value, and comparator deltas.
Measured cadence spacing is `F/strongest_frequency` output frames. Both
planned and measured spacing must increase strictly from short to neutral to
long cycle for the same source/ratio. Strength values are finite diagnostics.

### Y05 Gap, Tail, And Boundary

The gap hull is the set of output indices whose exact rational `a(y)` lies in
`[38588,49612]`. Inside that hull, measure the maximal contiguous exact-zero
output support and record its inclusive start, exclusive end, and length. Do
not substitute an expected label.

Record unexpected dropout count, exterior first difference, last active frame,
inactive terminal frames, exact-zero support, and candidate/comparator deltas.
The pad control must not report an intentional exact-zero gap. Missing support
measurement rejects.

### Y06 Linked Inventory

Whole-render energy uses `f64` sample sums. Three-band energy uses periodic
Hann `4096`, hop `2048`, and bands `0..250`, `250..1500`, and
`1500..Nyquist`. Width is mid/side energy ratio with
`M=(L+R)/2`, `S=(L-R)/2`, using the same whole/band partitions.

Local balance uses four-second output windows with two-second hops. Source
reference bounds are the exact ideal-map endpoints for each window.

Cross-correlation is normalized, uses interior active support, and searches
integer lags `-32..=32`; ties select smallest absolute lag, then negative lag.
Delay rows require `12..=14` and candidate peak no more than `0.02` below the
comparator. Gain rows require `0.01 dB`.

Record source, candidate, comparator, and both deltas for whole balance,
three-band balance, every mapped window, width, correlation lag, and
correlation peak. Hard long-form balance uses comparator `+0.50 dB`; local
balance and width remain mandatory finite diagnostics.

The `8x`, neutral-cycle versions of the six fixtures retain raw candidate
PCM/WAV hashes for the later stereo listening pack. No later rerender is
allowed.

## Comparator Preparation

The primary comparator is REAPER `7.69/macOS-arm64`, mode `983040`. Projects
use `44.1 kHz`, preserve pitch, item play rate `1/q`, mode field `0.0025`,
zero-length fade-in, `10 ms` fade-out, exact item length `qL/F`, and `24-bit`
stereo WAV. Only source path, play rate, item length, and render name vary.

The five retained `220500`-frame stereo sources are:

| Family/file | Source container SHA-256 |
| --- | --- |
| percussion `0000-drums_percussion-000002.wav` | `89e55b28c6ed36e26bf73f2024d301aaeedd07cca30b315f5530051c28f4e1e7` |
| bass `0004-bass-000236.wav` | `3d587007a5d9a683e82e14a184530a9a0f953e58fbc2fe3712a42aa86ecf9ad8` |
| vocals `0008-vocals-000010.wav` | `3d74a686dccf6dcdfedc57e0fc2b76a0d29374ba28afa1bb497172cc441f7ee9` |
| pads `0012-pads_sustains-000423.wav` | `a736a0e04ade9e879db954069c8c0b68842bd4d364eec69e62dfef1447763131` |
| full mix `0016-full_mix-000144.wav` | `caa5d0d7c51bc7e2d537d3d13dbe32055f0c2032c69bd8e3f28a38df96fafbf1` |

Retained comparator container hashes:

| Family | `4x` | `8x` |
| --- | --- | --- |
| percussion | `37db73f3b67b43eba491de1c084fe7d40429ed449f631d9107276b574661d90b` | `356723fe7d953ef119558590f30682f25de47f521be98c62a746fb69698a0c0a` |
| bass | `618830e45b3deb5ca07609c00d5cb7b8cb65b4cef64f242d47180e26e790a922` | `3fd429b76b95c41b5c869a6f9df646ec8cd2181c5fa3423272a596003c5dada6` |
| vocals | `af180619222408364022edda7fdb76819a8398a0e39da8bf0287448a808f154f` | `0ae69d4cf473f255c277047d39406ff089b42abc0ab747e8eeb973cf7e9e2764` |
| pads | `269b81b489df287e4dcb868f2966983da9d5c7cd03bb2c3e5ff7fe349d1c7660` | `77eb6b8f2314521543c6d4a161fa74b84a4af8b33bda706f3f5ee4f830a04f79` |
| full mix | `e07dc195460332fd8cd4088e1447de3added2122ed7c108418157b99cd6bcfae` | `64f8c450e400c589ac249a717d6c27750b7cf8aef41f501b1666643214fc3ce6` |

Before the first candidate source file is created:

1. regenerate the missing five `2x` long-form rows
2. regenerate every synthetic and stereo comparator row
3. write the exact ordered `63`-row `comparator.tsv`
4. bind source, project, container, decoded PCM, request, REAPER, and order
   hashes
5. require manifest SHA-256
   `eb5384681767dfd36e8daf81809a95d51a79f6cb178f0705fe4cffce9ecccacd`
6. bind historical `48`-row group receipt
   `5bb7b55456065d8f3d69c7229abc117eacb9280cf298a779b634598a19663e11`

A retained `4x/8x` mismatch, incomplete `2x`, missing synthetic row, changed
project field, or manifest mismatch stops before checkpoint creation.
Comparator audio stays ignored. Batch 32.8 itself performs no recapture.

## Exact `16x` Admission Control

After all synthetic rows pass, run five separate exact-`16x` rows in retained
source order. Each loads and hashes one source, requests `T=16L`, expects
`CandidateError::UnsupportedRatio`, proves zero candidate output allocation
and zero completed renders, and writes its durable receipt. Any output or
different error rejects.

## Long-Form Render And Level Policy

Long-form candidate rows are separate one-shot processes:

- mono neutral: five sources × `2x/4x/8x` = `15` renders
- mono short and long: five × three ratios × two cycles = `30` renders
- stereo neutral: five × three ratios = `15` renders

Total long-form candidate renders are `60`. Synthetic stereo fixture listening
reuses the six retained `Y06` `8x`, neutral raw outputs by verified PCM hash.

Candidate objective WAVs are deterministic IEEE `f32`, `44.1 kHz`, native
channel count, little-endian. Raw objective audio is never faded, normalized,
or overwritten.

For each listening row:

1. decode source, candidate, and comparator to `f32`
2. downmix mono as `f32((f64(left)+f64(right))/2)`
3. compute active whole-file RMS in `f64`
4. set `r_ref=RMS(source)`
5. set gains `g_source=1`, `g_candidate=r_ref/RMS(candidate)`,
   `g_comparator=r_ref/RMS(comparator)`
6. reject zero or non-finite RMS/gain
7. compute post-match peak over all three
8. set common safety `s=min(1,0.95/max_peak)`
9. apply `s*g_*` once in `f64`, then cast to `f32`

This preserves source level and RMS-matches both renders before one common
peak-safety reduction. No per-file peak normalization exists.

Listening copies apply the comparator's terminal `10 ms` fade only after level
matching. At `44.1 kHz`, `N=441`. Samples before `T-N` use gain one. For
`n=T-N..T-1`, gain is `(T-1-n)/(N-1)`, making the first fade sample one and
the last zero. Raw objective files remain unchanged.

Receipts bind raw PCM/WAV, matched PCM/WAV, gains, RMS, peaks, safety factor,
fade length, and final listening-copy hashes.

## Concealment And Decisions

For A/B row `row_id`, compute:

`h=SHA256(checkpoint || "|" || row_id || "|audited-centered-cyclic-conceal-v1")`.

If the low bit of `h[0]` is zero, A is candidate; otherwise B is candidate.
Write a public pack manifest containing only row, source family, ratio, cycle,
and A/B file names. Write the mapping to a separate ignored key file. The
operator and independent listener may not inspect the key or raw identity
paths before all decisions are locked.

Neutral mono review order is `8x`, `4x`, then `2x`. For each row, the decision
record freezes:

- `row_id`
- `recognizable_cyclic`: yes/no
- `musically_useful`: yes/no
- `preference`: A/B/tie
- `usable`: yes/no
- artifact flags: click, dropout, level step, buzz, lost event, doubled attack
- optional note

After all rows, reveal the key once and write the derived candidate decisions.
The decision file is append-only JSONL with the same started/terminal
durability rule. Missing, duplicate, edited-after-reveal, or ambiguous
decisions reject evidence.

Neutral pass requires:

- recognizable Cyclic character on at least `12/15`
- useful on at least `10/15`
- candidate preferred or tied on at least `10/15`
- candidate preferred or tied on at least `7/10` primary `4x/8x` rows
- no unusable row
- no family losing all three ratios
- no uncontrolled click, dropout, arbitrary level step, source-obscuring buzz,
  lost event, or attack outside the commanded grammar

Only after neutral passes, compare concealed short/neutral/long candidate
trios. Short must move toward metallic/ring-like motion on at least `12/15`;
long must move toward tremolo/echo-like motion on at least `12/15`. Each
direction must be useful on at least `10/15`. Any inverted direction or
unusable endpoint rejects.

Listening is promotion authority. Metrics cannot waive a listening failure.

## Stereo Admission

After mono passes, the `15` neutral stereo rows must first pass hard mechanics,
whole/three-band balance, and delay/gain controls. The operator then performs
a speaker pre-screen and may reject.

Promotion still requires one eligible independent listener. The scoped Dream
waiver does not apply to Cyclic. The listener reviews:

- `15` concealed candidate/comparator long-form rows
- six concealed `8x`, neutral-cycle fixtures: stable centre, wide sustain,
  transient relation, delay, unequal level, and anti-phase

Stereo pass requires:

- every hard relation and balance row passes
- no unusable row
- candidate preferred or tied on at least `10/15` long-form rows
- no family loses every ratio
- no centre pull, one-sided texture, width pumping, side inversion, detached
  echo, channel-local cycle, or arbitrary balance movement

The independent listener records the same immutable decision schema plus
stereo artifact flags. Missing or ambiguous eligible review blocks promotion.

## Gate Order

Every stage runs from the same acoustic ref and stops at first failure:

1. `Y01` through `Y06`, numeric gate and manifest row order
2. five-source exact `16x` typed rejection
3. concealed long-form mono neutral review
4. short/neutral/long direction review
5. long-form stereo hard objectives
6. operator speaker pre-screen
7. eligible independent stereo review
8. fixed-ratio promotion decision

No code, source, formula, constant, metric, threshold, comparator, fixture,
helper, listening rule, or runner change is allowed after the ref.

## Failure, Cleanup, And Admission

Every stop records:

- row and stage
- durable receipt state and nextest exit
- checkpoint commit/tree/ref
- source, comparator, output, artifact, and receipt hashes
- exact failed assertions or incomplete evidence edge
- last complete gate

Any structural, synthetic, long-form, stereo, or listening failure closes this
candidate. It does not authorize tuning or rerun. A finite diagnostic alone
cannot reject.

If this fresh checkpoint again lacks a frozen field, assertion, executor, or
durable failed-row boundary, it is the second incomplete-evidence checkpoint
and closes the centred compressed-anchor identity as protocol churn.

After the required docs-only reassessment, delete:

- isolated worktree and branch
- candidate module, tests, runner, and tracked candidate evidence
- build state and generated candidate/comparator/listening copies
- local acoustic ref

No rejected constant, helper, fixture, hidden API, report mode, partial
mechanism, or threshold enters `main`.

Only a complete pass may open a separate minimal admission batch for:

- private
  `creative_audited_centered_compressed_anchor_cyclic`
- its private exact-target manual-cycle request, errors, and renderer
- structural and synthetic regression owners required to preserve admission
- one frozen Cyclic engine-version identifier

Do not admit public `Cyclic`, Auto, `INTELL`, routing, cache, artifacts,
dynamic ratio, pitch, reverse, seed, Dream controls, runtime integration,
Loophole, or Chorus.

## Remaining Risks

- direct two-read crossfade may retain the surfaced dropout on some material
- compressed-anchor replicas may sound too pitch-shifted, metallic, or regular
- long cycles may become detached echo; short cycles may become buzz
- zero exterior reads may create material-dependent onset or tail energy
- one fixed linked schedule may preserve algebra while sounding narrow or
  spatially detached
- REAPER behavior and the Signal formula may diverge in musically important
  ways despite finite comparator-relative metrics
- the expanded one-row protocol has high process overhead and must still fit
  its frozen envelope
- independent eligible stereo review remains an external availability risk

These are admission risks, not open design choices.

## Next Task

Execute `g10.032` Batch 32.9 only. From the exact Batch 32.8 closeout commit,
create the fresh isolated identity, prepare and bind the comparator manifest,
implement this unchanged renderer and complete row protocol, then run compile,
construction, and two structural conformance rounds. Stop before every
synthetic, exact-`16x`, long-form, or listening row.
