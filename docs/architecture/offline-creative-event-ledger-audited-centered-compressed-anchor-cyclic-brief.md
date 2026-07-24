# Offline Creative EventLedgerAuditedCenteredCompressedAnchorCyclic Brief

Status: admitted privately at exact fixed `2x`, `4x`, and `8x`
Owner: dsp
Updated: 2026-07-24
Contracts: `046`, `085` Rule 11
Roadmap: `g10.032`, Batch 32.18
Behavior:
[Offline Creative Cyclic Behavioral Synthesis](./offline-creative-cyclic-behavioral-synthesis.md)

## Decision

Build one fresh isolated `EventLedgerAuditedCenteredCompressedAnchorCyclic` candidate.
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

No DSP formula, scalar, source, threshold, or listening policy is derived from
the rejected output. The evidence-ownership correction comes from the valid
Y01 receipt and independent static reconstruction recorded in Batch 32.17.
The old checkpoint is rejected and deleted. Its source, tests, helpers,
objects, branch, worktree, build state, and ref may not be recovered.

This fresh identity changes only executable evidence ownership. The six
neutral-cycle Y01 rows for `impulse` and `impulse-train` use the commanded
event ledger, not continuous mapped-window dropout. The other `24` Y01 rows
and every Y02, Y04, and Y05 row retain the `221`-frame, source-above-`-40`
dBFS, output-below-`-80` dBFS dropout rule unchanged.

Every structural, synthetic, rejection, render, and listening row is its own
one-shot process and append-only receipt. A process persists `started` before
loading audio or invoking the renderer. It persists `pass`, `fail`, or
`panic` before nextest reports the result. A timeout or external kill leaves a
durable incomplete receipt and rejects the checkpoint.

This is the one fresh evidence-backed product-gate identity allowed by
Contract `085` Rule 11. No rejected candidate source, evidence helper,
manifest generator, comparator helper, receipt writer, object, or test may be
recovered. The implementation must be written from this brief. Any missing,
placeholder, or non-executable owner closes the identity before acoustic
work.

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

Batch 32.19 starts from the exact pre-source authority-correction closeout
commit and creates only:

- worktree: `/Users/tom/Dev/projects/signal-candidate-32-19`
- branch:
  `candidate/g10-032-event-ledger-audited-centered-compressed-anchor-cyclic`
- private module:
  `crates/signal-dsp-stretch/src/creative_event_ledger_audited_centered_compressed_anchor_cyclic/`
- module files: `mod.rs`, `plan.rs`, `schedule.rs`, `interpolate.rs`,
  `synthesis.rs`, `evidence.rs`, `tests.rs`
- tracked evidence authority:
  `candidate-evidence/g10-032/32-19/`
- tracked nextest config: `.config/nextest.toml`
- ignored build/evidence root:
  `target/creative-stretch-event-ledger-audited-centered-compressed-anchor-cyclic-32-19/`
- acoustic ref:
  `refs/signal-evidence/creative/event-ledger-audited-centered-compressed-anchor-cyclic/32-19-acoustic`

Existing dependencies only. The isolated `lib.rs` may declare the module
privately. No rejected file, helper, commit, branch, ref, object, output,
receipt, build artifact, or generated comparator copy may be recovered.

The tracked evidence directory contains:

- `row-manifest.tsv`
- `comparator.tsv`
- `listening-manifest.tsv`
- `run-event-ledger-audited-centered-cyclic.sh`
- `generate-event-ledger-audited-centered-cyclic-evidence.py`
- `generate-event-ledger-audited-centered-cyclic-reaper.py`

Generated audio and receipts stay ignored. The runner is candidate-local
evidence scaffolding, not a product surface.

Test names use only:

- `event_ledger_audited_centered_cyclic_construction_row`
- `event_ledger_audited_centered_cyclic_structural_row`
- `event_ledger_audited_centered_cyclic_synthetic_row`
- `event_ledger_audited_centered_cyclic_exact16_row`
- `event_ledger_audited_centered_cyclic_listening_row`
- `event_ledger_audited_centered_cyclic_summary`

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
[profile.event-ledger-audited-centered-cyclic]
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

`schema` is `signal.event-ledger-audited-centered-cyclic.row.v1`; `identity` is
`EventLedgerAuditedCenteredCompressedAnchorCyclic`. Excluded fields are JSON `null`.
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

It also executes the seven frozen owner-known-answer groups below through the
same functions later used by synthetic and comparator rows. A label, non-null
function pointer, authored-value substitution, self-comparison, or helper
which bypasses the production evidence path fails construction.

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
complementary half-cosine ramps. For active offset `j=n-22050`:

- `0<=j<2048`: `w=0.5-0.5*cos(pi*j/2047)`
- `2048<=j<42052`: `w=1`
- `42052<=j<44100`: `w=0.5+0.5*cos(pi*(j-42052)/2047)`
- otherwise: exact positive `0f32`

Thus the first and last active-lattice samples are exact zero and the two
interior ramp endpoints are exact one. Evaluate the named source formula in
`f64`, multiply by `w` in `f64`, and cast once to `f32`. Impulse and impulse
train bypass the sustained ramp and contain only their named non-zero samples.

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
and `n=0`, `1`, `22050`, `22051`, `24097`, `24098`, `44100`, `64101`,
`64102`, `66148`, `66149`, and `88199`. Outside-support and explicit gap
samples are written as positive-zero bits.

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
- exterior-inclusive first difference within the comparator `+1.50 dB` rule
- linked mechanics within frozen bounds
- every required diagnostic present and finite

Continuous-source rows additionally require no complete `5 ms` output RMS
window below `-80 dBFS` while its ideal-map source window exceeds `-40`
dBFS. This applies to Y01 sources `low-tone`, `high-tone`, `chord`,
`harmonic-pad`, `silence-gap`, `uniform-noise`, `rademacher-noise`, and
`am-noise`, and to every Y02, Y04, and Y05 row. It does not apply to
`impulse` or `impulse-train`.

Dropout uses `221` samples at `44.1 kHz`. Candidate output windows map through
the exact ideal-map endpoints. For output window `[b,b+221)`, the source
reference is the ascending integer range
`ceil(a(b))..=floor(a(b+220))`, clipped to `[0,L)`. Source activity is the
`f64` RMS of that exact range. An empty range or incomplete output window is
ineligible. A non-finite RMS rejects.

Sparse-event rows instead require every event appearance inside the
independent ledger, no output above `1e-7` outside the union ledger, a
positive appearance for every authored event, ordered event centres, and the
frozen centre bound for every fully interior event.

### Numeric Evidence Primitives

All evidence uses these functions. No gate-local alternative exists:

- mono absolute peak is `max(abs(sample))`; linked peak is the maximum across
  channels
- RMS is `sqrt(sum(sample^2)/sample_count)` with `f64` accumulation
- amplitude dB is
  `20*log10(max(amplitude,f64::MIN_POSITIVE))`
- power ratio dB is
  `10*log10(max(numerator,f64::MIN_POSITIVE)/max(denominator,f64::MIN_POSITIVE))`
- an exact-zero test accepts `+0f32` or `-0f32`; positive-zero-bit assertions
  additionally require bits `0x00000000`
- exterior first difference is the largest absolute difference across
  exterior zero to frame `0`, every adjacent frame, and the last frame to
  exterior zero, taking the maximum across channels before amplitude-dB
  conversion
- an active tail frame has any channel with absolute sample above `1e-7`;
  `last-active-frame` is its final zero-based index and `inactive-frames` is
  `T-1-last_active`; no active frame rejects the tail measurement

Candidate-source and comparator-source balance errors are absolute dB
differences. Candidate-comparator delta is candidate error minus comparator
error. Frequency and support deltas are candidate minus comparator. These
directions never vary by gate.

### Y02 Pitch

For each authored component, run the same estimator independently on source,
candidate, and comparator. The source interval is `[22050,66150)`. Candidate
and comparator use the exact output hull whose ideal-map centres fall in that
interval.

For a signal of hull length `M`, multiply sample `n` by
`0.5-0.5*cos(2*pi*n/M)`. Zero-pad to the least power of two `N` with
`N>=max(M,262144)`. Run one unnormalized real-to-complex `f64` FFT and set
`m[k]=hypot(re[k],im[k])`. Search bins whose centre frequencies are within
one octave of a tone, or within `+-15%` of the named chord/pad partial. Ties
select the lower bin.

For the winning interior bin `k`, set
`l[j]=ln(max(m[j],f64::MIN_POSITIVE))` and
`delta=0.5*(l[k-1]-l[k+1])/(l[k-1]-2*l[k]+l[k+1])`. A zero denominator,
non-finite value, or `delta` outside `[-0.5,0.5]` rejects. Estimated frequency
is `(k+delta)F/N`. `input-cents` is
`1200*log2(candidate_frequency/source_frequency)`;
`comparator-cents` is
`1200*log2(candidate_frequency/comparator_frequency)`.
`resolution-cents` is
`1200*log2((candidate_frequency+F/N)/candidate_frequency)`.

Record candidate frequency, both cents values, candidate peak magnitude,
interpolated bin, and resolution. Every value must be measured and finite.
Pitch is diagnostic, not an absolute character veto.

### Y03 Replicas

The independent oracle evaluates each authored event separately. For event
`e` and anchor `k`, its positive contribution at output `y` is the matching
positive raised-cosine weight multiplied by the positive linear-interpolation
coefficient of source sample `e`. The oracle uses the brief's rational
formula directly and may not call renderer schedule, position, interpolation,
or weight helpers.

An appearance is one anchor's maximal contiguous output run with positive
oracle contribution for that event. Overlapping anchor appearances remain
distinct, while the event ledger is their union. Measure each centre from that
run's absolute candidate-sample weights. Aggregate event centre uses absolute
candidate-sample weights across the union ledger.
Spacing is the difference between adjacent measured appearance centres;
median averages the two middle values for an even count. Extent is half-open.
Peak is maximum absolute candidate sample and energy is its `f64` sum of
squares inside the event ledger.

For crop accounting, enumerate every anchor whose rational `x_k` lies in
`(e-H-1,e+H+1)`, then intersect its positive weight interval
`[(k-1)H+1,(k+1)H)` with the open interpolation interval where
`e-1<p(k,y)<e+1`. This finite exact intersection is one unbounded appearance.
Clip it to `[0,T)` for the measured ledger. Count an appearance as leading or
trailing cropped when any of its positive integer support lies below `0` or
at or above `T`; an appearance crossing both sides is invalid. Construction
proves the enumeration bounds against a direct wider scan.

Comparator event centre uses absolute mono sample weights inside the event's
Voronoi cell. Cell boundaries are midpoints between adjacent ideal mapped
authored centres; the first and last cells extend to `0` and `T`. A zero or
non-finite candidate or comparator weight sum rejects. Record appearance count,
spacing median/range, aggregate centre, extent, peak, energy, leading and
trailing cropped appearance counts, and candidate-minus-comparator centre.
Hard ledger assertions execute for both Y03 and sparse Y01 rows.

### Y04 Cadence And Modulation

Build an RMS envelope with complete `2048`-sample windows and `512`-sample
hops starting at the first sample of the mapped active hull. Use `f64`
sum-of-squares. Subtract the envelope mean, multiply element `j` of length
`M` by `0.5-0.5*cos(2*pi*j/M)`, and zero-pad to the least power of two
`N>=max(M,16384)`. Run the same unnormalized real `f64` FFT owner as Y02.

Search `0.1..=20 Hz` for the largest magnitude non-DC envelope bin; ties
select the lower bin. Frequency is `k*F/(512*N)`. Strength is the winning
bin's squared magnitude divided by the sum of squared magnitudes across the
search band. For amplitude-modulated noise, repeat the same choice within
`1.45..=1.95 Hz`; a magnitude tie selects the bin closest to `1.7 Hz`, then
the lower bin.

Autocorrelation uses the unwindowed mean-subtracted envelope:
`R(l)=sum(e[j]*e[j+l])/sum(e[j]^2)`. Search lags
`round(D/512)-2..=round(D/512)+2`, clipped to positive valid lags.
Select the largest `R`; ties select the lag closest to `D/512`, then the
smaller lag. Zero energy or any missing/non-finite result rejects.

Record planned `D`, strongest frequency and strength, authored sideband
frequency and strength, autocorrelation lag/value, and comparator deltas.
Measured cadence spacing is the selected autocorrelation lag times the
`512`-frame envelope hop. Exact planned spacing must increase strictly from
short to neutral to long cycle for the same source/ratio. Measured spacing,
spectral frequency, and strength remain finite diagnostics: the frozen
`0.1..20 Hz` search cannot resolve cadence above `20 Hz`, including the
shortest cycle settings.

### Y05 Gap, Tail, And Boundary

The gap hull is the set of output indices whose exact rational `a(y)` lies in
`[38588,49612]`. Inside that hull, measure the maximal contiguous exact-zero
output support. Runs are half-open; longest wins, then earliest start. Record
its inclusive start, exclusive end, and length. No run is a measured
`none`, not `(0,0)`. Do not substitute an expected label.

The `silence-gap` row requires one measured run and records its bounds. The
`harmonic-pad` control instead counts exact-zero runs of at least `221`
samples and records the maximum exact-zero run length, using zero when no run
exists. `pad.no-false-gap` requires that count to be zero. This avoids
encoding an absent run as fictional support bounds.

Record unexpected dropout count, exterior first difference, last active frame,
inactive terminal frames, the source-valid gap fields, and
candidate/comparator deltas. Missing required measurement rejects.

### Y06 Linked Inventory

Whole-render energy uses `f64` sample sums. Three-band energy uses periodic
Hann `4096`, hop `2048`, and bands `0..250`, `250..1500`, and
`1500..Nyquist`. Width is mid/side energy ratio with
`M=(L+R)/2`, `S=(L-R)/2`, using the same whole/band partitions.
Every energy ratio in dB is
`10*log10(max(numerator,f64::MIN_POSITIVE)/max(denominator,f64::MIN_POSITIVE))`;
this keeps exact duplicate and anti-phase fixtures finite without weakening
their samplewise hard assertions.

Whole energy is `sum(sample^2)`. Band analysis starts at frame zero and uses
only complete `4096`-frame windows. Each sample is multiplied by
`0.5-0.5*cos(2*pi*n/4096)`. Run an unnormalized `4096`-point real `f64` FFT.
One-sided power is `re^2+im^2`, doubled for bins other than DC and Nyquist.
A bin belongs to the half-open band containing `kF/4096`; Nyquist belongs to
the high band. Sum power across bins and windows. A row without one complete
window rejects.

Local balance uses four-second output windows with two-second hops. Source
reference bounds are the exact ideal-map endpoints for each window.

Cross-correlation uses the exact candidate/comparator output hull whose ideal
map lies in source support `[22050,66150)`. For lag `l`, pair left frame `n`
with right frame `n+l` where both are inside the hull. Set
`rho(l)=sum(L*R)/sqrt(sum(L^2)*sum(R^2))`. Zero denominator rejects. Search
integer lags `-32..=32` by largest `abs(rho)`; ties select smallest absolute
lag, then negative lag. Record the selected lag and `abs(rho)`.
Delay rows require `12..=14` and candidate peak no more than `0.02` below the
comparator. Gain rows require `0.01 dB`.

Record source, candidate, comparator, and both deltas for whole balance,
three-band balance, every mapped window, width, correlation lag, and
correlation peak. Hard long-form balance uses comparator `+0.50 dB`; local
balance and width remain mandatory finite diagnostics.

Local output windows are exactly `4F` frames starting at `0,2F,4F,...` while
complete. Candidate and comparator use that output interval. Source uses
`ceil(a(b))..=floor(a(b+4F-1))`, clipped to `[0,L)`. Empty source reference
or zero channel energy rejects. Window order defines `wWW`.

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

Before the first candidate source file is created, generate the native-stereo
musical and canonical synthetic inputs, recapture all `63` Comparator Manifest
V2 rows, and bind source, normalized project, project container, output
container, and decoded PCM hashes. Historical mono comparator outputs are
provenance only and do not gate this fresh native-stereo authority. Comparator
audio stays ignored.

The comparator root is exactly
`<ignored-root>/comparator/{sources,projects,outputs}`. Copy the five retained
musical containers byte-for-byte into `sources/musical/<frozen-filename>`.
Generate synthetic files as
`sources/synthetic/<source_id>.wav` and
`sources/stereo/<fixture>.wav`. Project and output names are
`projects/<comparator-row-id>.rpp` and
`outputs/<comparator-row-id>.wav`. The generator requires every destination
to be absent and never overwrites. A second source construction required by
`common-negation` or `swap` is derived in memory by the evidence row; it does
not add a comparator-manifest row.

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
3. compute whole-file RMS across every frame in `f64`
4. set `r_ref=RMS(source)`
5. set gains `g_source=1`, `g_candidate=r_ref/RMS(candidate)`,
   `g_comparator=r_ref/RMS(comparator)`
6. reject zero or non-finite RMS/gain
7. compute post-match peak over all three
8. set common safety `s=min(1,0.95/max_peak)`
9. apply `s*g_*` once in `f64`, then cast to `f32`

This preserves source level and RMS-matches both renders before one common
peak-safety reduction. No per-file peak normalization exists.

Source RMS uses native source frames. Candidate and comparator RMS use all
exactly `T` output frames. Stereo level matching computes one RMS across all
interleaved samples and applies one shared gain to both channels.

Listening copies apply the comparator's terminal `10 ms` fade only after level
matching. At `44.1 kHz`, `N=441`. Samples before `T-N` use gain one. For
`n=T-N..T-1`, gain is `(T-1-n)/(N-1)`, making the first fade sample one and
the last zero. Raw objective files remain unchanged.

Receipts bind raw PCM/WAV, matched PCM/WAV, gains, RMS, peaks, safety factor,
fade length, and final listening-copy hashes.

## Concealment And Decisions

For A/B row `row_id`, compute:

`h=SHA256(checkpoint || "|" || row_id || "|event-ledger-audited-centered-cyclic-conceal-v1")`.

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

Promotion defaults to one eligible independent listener. Contract `085` Rule
5 permits one explicit operator waiver scoped to this exact fixed-ratio
creative candidate after every hard stereo control passes. Without that
waiver, the listener reviews:

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
7. eligible independent stereo review or explicit Contract `085` Rule 5 waiver
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
  `creative_event_ledger_audited_centered_compressed_anchor_cyclic`
- its private exact-target manual-cycle request, errors, and renderer
- structural and synthetic regression owners required to preserve admission
- one frozen Cyclic engine-version identifier

Do not admit public `Cyclic`, Auto, `INTELL`, routing, cache, artifacts,
dynamic ratio, pitch, reverse, seed, Dream controls, runtime integration,
Loophole, or Chorus.

## Pre-Source Execution Audit

Batch 32.9 stopped before worktree creation, comparator recapture, or
candidate source. The implementation audit found authority the brief requires
construction to verify but does not itself freeze:

- exact bytes and schema of the required `63`-row comparator manifest
- exact synthetic/stereo comparator row membership and order
- exact REAPER project and source-container generator bytes
- exact expanded row IDs and per-row assertion/diagnostic IDs
- exact summary schema and aggregate receipt bytes
- exact runner environment, receipt-root, and child-sentinel handshake
- exact long-form pack, decision, reveal, and stereo-review manifest schemas

The deleted checkpoint is the only known local source for some of these
values, and this brief prohibits recovering it. Its manifest hash is therefore
a verifier without a reproducible generator.

No candidate identity exists. This is a docs-authority stop, not an acoustic
or structural result and not a second incomplete-evidence checkpoint.

## Executable Manifest Authority V2

This section closes Batch 32.9's gaps and supersedes earlier manifest,
comparator-hash, runner, summary, sentinel, and decision-file wording where
they differ. Renderer, sources, metrics, thresholds, and gate order do not
change.

### Canonical Text Encoding

Every tracked manifest is UTF-8 without BOM, uses LF only, and ends with one
LF. TSV fields are printable ASCII without tab, CR, or LF. Decimal integers
have no sign or leading zero except zero. Finite decimal values use Rust
shortest round-trip formatting. Boolean is `0` or `1`; absent is `-`; ordered
lists use comma without spaces. SHA-256 is lowercase hexadecimal over exact
file bytes.

The row-manifest header is exactly:

```text
schema	ordinal	row_id	stage	entrypoint	source_id	sample_rate	input_frames	channels	ratio_num	ratio_den	target_frames	cycle_us	render_count	assertion_ids	diagnostic_ids	comparator_row_id	artifact_policy
```

Every data row uses schema
`signal.event-ledger-audited-centered-cyclic.row-manifest.v2`. `ordinal` is global,
zero-based, and six digits. Derived integers use checked arithmetic. A
generator overflow is an authority failure, not a skipped row.

The fixed value orders are:

- ratios: `2`, `4`, `8`
- cycles: `5000`, `48000`, `90000`
- layouts: mono, stereo
- musical sources: percussion, bass, vocals, pads, full-mix
- mono synthetic sources: low-tone, high-tone, chord, harmonic-pad, impulse,
  impulse-train, silence-gap, uniform-noise, rademacher-noise, am-noise
- stereo fixtures: duplicate, anti-phase, common-negation, swap, right-gain,
  right-delay13

Cross products use the leftmost listed dimension as the outer loop. Row IDs
use lowercase ASCII slugs, decimal ratio, and six-digit cycle microseconds.
Gate-local indices start at `000`.

Global group order is `C00`, `S01..S09`, `Y01..Y06`, `E16`, `M01`, then
`M02`. The final manifest has exactly `588` data rows:
one construction, `339` structural, `183` synthetic, five exact-`16x`, and
`60` long-form render rows.

### Exact Row Expansion

`C00-000-construction` is the sole construction row. It uses the full test
name
`creative_event_ledger_audited_centered_compressed_anchor_cyclic::tests::event_ledger_audited_centered_cyclic_construction_row`.
Its assertion IDs are, in order:
`construction.file-inventory`, `construction.module-inventory`,
`construction.dependency-inventory`, `construction.renderer-spec`,
`construction.row-manifest`, `construction.comparator-manifest`,
`construction.listening-manifest`, `construction.source-generator`,
`construction.project-generator`, `construction.runner-profile`,
`construction.dispatch`, `construction.failure-receipt`,
`construction.panic-receipt`, `construction.kill-receipt`,
`construction.duplicate-block`, `construction.escape-vectors`,
`construction.fft-owner-known-answers`,
`construction.ledger-owner-known-answers`,
`construction.cadence-owner-known-answers`,
`construction.gap-owner-known-answers`,
`construction.stereo-owner-known-answers`,
`construction.level-owner-known-answers`,
`construction.comparator-owner-known-answers`,
`construction.worst-case-envelope`.
It has no diagnostics or renders.

Structural entrypoint is the same path ending
`event_ledger_audited_centered_cyclic_structural_row`. Structural rows are:

- `S01`: the following `22` cases in exact order:
  `empty-success`, `mono-identity`, `stereo-r2`, `mono-r8`, `channels-0`,
  `channels-3`, `partial-stereo`, `rate-7999`, `rate-192001`, `nan`,
  `positive-infinity`, `negative-infinity`, `cycle-4999`, `cycle-90001`,
  `empty-nonzero-target`, `nonempty-zero-target`, `compression`, `over-r8`,
  `exact-r16`, `direct-l-over-2p53`, `direct-t-over-2p53`,
  `direct-allocation-overflow`
- `S02`: cycle conversion for rates `8000,44100,48000,192000` × all cycles;
  map positions `first,pre-cycle,mid,last` for each ratio; anchors
  `zero,one,two,last,last-plus-one` for each ratio; complete monotonicity for
  each ratio
- `S03`: window values for `H=240,2304,4320` ×
  `zero,quarter,half,three-quarter,end`; complements for the same `H` ×
  `zero,one,quarter,half,pre-end`; interpolation cases
  `u-zero,u-quarter,u-half,u-three-quarter,u-next-down-one,integer-node,
  negative-exterior,right-exterior,coefficient-sum,peak-convexity`
- `S04`: ratios × cycles × positions `first,pre-cycle,cycle,post-cycle,last`;
  ratios × cycles spacing; ratios × events `0,48000,95999` at neutral cycle
- `S05`: ratios × cycles × lengths
  `one,h-minus-one,h,h-plus-one,two-h,two-h-plus-one`
- `S06`: layouts × identity lengths `0,1,4096`; layouts × ratios × cycles
  silence; ratios `2,8` × cycles `5000,90000` constant edge
- `S07`: fixtures × ratios × cycles
- `S08`: rates `8000,192000` × cycles `5000,90000` × layouts capacity;
  `instrumented-short-stereo`, `instrumented-long-stereo`; layouts × ratios
  `2,8` repeat; oracles `direct-size`, `duration-independence`,
  `processing-allocation`, `maximum-table`, `maximum-working-bytes`,
  `scalar-traversal`
- `S09`: `free-cursor`, `grain-list`, `repeat-counter`, `similarity-search`,
  `detector`, `fft`, `stochastic-state`, `feedback`, `post-gain`, `limiter`,
  `width-stage`, `public-module`, `public-api`, `feature-or-report`,
  `new-dependency`, `audio-thread-claim`

Structural row ID is
`<gate>-<local-index>-<case-slug>`. Cross-product slugs join dimension values
with `-`, for example `S05-000-r2-c005000-one`. The expansion above must yield
the frozen `339` rows and `168` renders.

Exact structural slug templates are:

| Family | Slug |
| --- | --- |
| S01 | the listed case verbatim |
| S02 cycle | `cycle-f<rate>-c<cycle6>` |
| S02 map | `map-r<ratio>-<position>` |
| S02 anchor | `anchor-r<ratio>-<anchor>` |
| S02 monotonic | `monotonic-r<ratio>` |
| S03 window | `window-h<h>-<point>` |
| S03 complement | `complement-h<h>-<point>` |
| S03 interpolation | `interpolation-<case>` |
| S04 schedule | `schedule-r<ratio>-c<cycle6>-<position>` |
| S04 spacing | `spacing-r<ratio>-c<cycle6>` |
| S04 ledger | `ledger-r<ratio>-e<event>` |
| S05 | `r<ratio>-c<cycle6>-<length>` |
| S06 identity | `identity-<layout>-l<length>` |
| S06 silence | `silence-<layout>-r<ratio>-c<cycle6>` |
| S06 edge | `edge-r<ratio>-c<cycle6>` |
| S07 | `<fixture>-r<ratio>-c<cycle6>` |
| S08 capacity | `capacity-f<rate>-c<cycle6>-<layout>` |
| S08 instrumented | the listed case verbatim |
| S08 repeat | `repeat-<layout>-r<ratio>` |
| S08 oracle | `oracle-<oracle>` |
| S09 | the listed case verbatim |

`cycle6` is six decimal digits, including leading zeros. Other numeric slug
fields have no leading zeros.

Structural defaults are `F=44100`, `L=4096`, mono, `T=8192`, and
`cycle_us=48000`. `S02` map/anchor rows use `F=48000`, `L=96000`, `H=2304`,
and `T=ratio*L`; `pre-cycle=H-1`, `mid=T/2`, and
`last-anchor=floor((T-1)/H)`. `S04` uses the same `F` and `L`. `S05` computes
each length from the row cycle's `H`, uses `F=48000`, and its source is `0.25`
plus `0.5` at `floor(L/2)`. `S07` uses `F=48000`, `L=8192`. `S08`
instrumented rows use `L=4096`: short is `F=8000`, `2x`, `5000 us`; long is
`F=192000`, `8x`, `90000 us`.

`S01` overrides are exact:

| Case | Input/request override |
| --- | --- |
| empty | zero samples, `L=0`, `T=0`, mono |
| identity | `L=T=4096`, mono |
| stereo `2x` | `8192` interleaved samples, `L=4096`, `T=8192` |
| mono `8x` | `L=4096`, `T=32768` |
| channels `0` | `4096` samples, channels `0`, `T=8192` |
| channels `3` | `4095` samples, channels `3`, `T=2730` |
| partial stereo | `8191` samples, channels `2`, `T=8192` |
| non-finite | replace mono sample `2048` with the named value |
| invalid empty/target | `(L,T)=(0,1)` or `(4096,0)` |
| compression | `L=4096`, `T=2048` |
| over `8x` | `L=4096`, `T=32769` |
| exact `16x` | `L=4096`, `T=65536` |
| direct `L` limit | internal dimension oracle `L=9007199254740992` |
| direct `T` limit | internal dimension oracle `T=9007199254740992` |
| allocation overflow | internal sample-count oracle, channels `2`, `T=9223372036854775808` |

All valid structural sources not otherwise specified are
`f32(0.25*sin(2*pi*440*n/F))`. `S03` interpolation uses source nodes
`[-0.75f32,0.5f32]`; the named `u` values apply between them. Negative
exterior reads at `p=-0.25`; right exterior reads at `p=1.25`. Coefficient
sum and convexity use the same nodes.

`S01` expected private errors are:

| Cases | Error |
| --- | --- |
| channels | `InvalidChannels` |
| partial stereo | `PartialFrame` |
| rates | `InvalidSampleRate` |
| three non-finite cases | `NonFiniteInput` |
| cycles | `InvalidCycle` |
| invalid empty/target pairs | `InvalidEmptyTarget` |
| compression | `UnsupportedCompression` |
| over-`8x`, exact `16x` | `UnsupportedRatio` |
| direct `L/T` limits | `ExactIntegerLimit` |
| direct allocation overflow | `AllocationOverflow` |

The synthetic entrypoint is the full construction path ending
`event_ledger_audited_centered_cyclic_synthetic_row`. Row IDs are
`<gate>-<local-index>-<source>-r<ratio>-c<cycle_us>`.

- `Y01`: mono source × ratio at `c048000`
- `Y02`: `low-tone,high-tone,chord,harmonic-pad` × ratio × cycle
- `Y03`: `impulse,impulse-train` × ratio × cycle
- `Y04`: `uniform-noise,rademacher-noise,am-noise` × ratio × cycle
- `Y05`: `silence-gap,harmonic-pad` × ratio × cycle
- `Y06`: stereo fixture × ratio × cycle

This yields exactly `183` rows and `201` renders. `common-negation` and `swap`
have two renders; every other row has one.

The exact-`16x` entrypoint ends `event_ledger_audited_centered_cyclic_exact16_row`. IDs are
`E16-<local-index>-<musical-source>-r16-c048000` in musical-source order.

The listening-render entrypoint ends `event_ledger_audited_centered_cyclic_listening_row`.
Candidate render rows are:

- `M01`: musical source × ratio × cycles `48000,5000,90000`, but ordered as
  all neutral `15`, then all short `15`, then all long `15`
- `M02`: musical source × ratio at neutral cycle, native stereo

`M01` IDs are
`M01-<local-index>-mono-<source>-r<ratio>-c<cycle6>`. `M02` IDs use the same
grammar with `stereo` and neutral cycle. This is exactly `60` candidate
renders. No row is generated at runtime from an untracked list.
`M01` input is `f32((f64(left)+f64(right))/2)` from the named retained source;
`M02` uses its native interleaved stereo samples.

Manifest fields are resolved as follows:

- stages are exactly `construction`, `conformance`, `synthetic`, `exact16`,
  `mono-render`, and `stereo-render`
- construction source and all numeric request fields are `-`
- structural `source_id` is `structural-<case-slug>`
- construction/structural comparator row is `-`, artifact policy `none`
- every synthetic row maps by source and ratio to `C-Y-*` or `C-ST-*`;
  artifact policy is `hash-only` except neutral `8x` `Y06`, which is
  `raw-wav`
- exact `16x` has no comparator and artifact policy `none`
- `M01/M02` map to `C-M-<source>-r<ratio>` and use `raw-wav`
- synthetic request fields are `F=44100`, `L=88200`, native channel count
- `E16/M01/M02` use `F=44100`, `L=220500`; E16 and M02 are stereo, M01 mono
- `target_frames=ratio*input_frames` for every ratio row
- `ratio_den=1`; non-ratio structural rows use `-`

### Source Completion

Structural stereo uses:

`b0[n]=f32(0.25*sin(2*pi*440*n/F)+0.10*cos(2*pi*173*n/F))`

`b1[n]=f32(0.20*cos(2*pi*311*n/F)-0.08*sin(2*pi*97*n/F))`.

Synthetic stereo uses the existing amplitude-modulated Rademacher source as
`b0`. `b1` uses the same envelope and formula but replaces `TEST` with
`TEST.rotate_left(17)` before `mix64`.

Fixtures are exactly:

- duplicate: `(b0,b0)`
- anti-phase: `(b0,-b0)`
- common-negation: `(b0,b1)` and `(-b0,-b1)`
- swap: `(b0,b1)` and `(b1,b0)`
- right-gain: `(b0,0x3f004dce*b0)`, multiplying in `f32`
- right-delay13: left `b0[n]`; right is exact zero for `n<13`, then `b0[n-13]`

Formula arithmetic not explicitly marked `f32` remains `f64` with one final
cast.

### Assertion And Diagnostic Binding

Manifest assertion lists contain the following exact IDs in listed order.
Construction expands the group; receipts contain IDs, never group names.

| Rows | Assertion IDs |
| --- | --- |
| S01 success | `request.accepted,output.frames,output.finite` |
| S01 identity | previous plus `output.identity-bytes` |
| S01 failure | `request.error-variant,allocation.output-zero` |
| S02 cycle | `cycle.half-up-frames` |
| S02 map | `map.numerator,map.denominator,map.sample-centre` |
| S02 anchor | `anchor.numerator,anchor.denominator,anchor.position` |
| S02 monotonic | `anchor.complete-monotonic` |
| S03 window | `window.value,window.endpoint` |
| S03 complement | `window.complement` |
| S03 interpolation | `interpolation.value,interpolation.exterior-zero,interpolation.convex` |
| S04 schedule | `schedule.two-anchors,schedule.anchor-ids` |
| S04 spacing | `schedule.spacing-law` |
| S04 ledger | `ledger.independent,ledger.event-present,ledger.order,ledger.centre` |
| S05 | `output.frames,output.finite,output.exact-crop` |
| S06 identity | `output.identity-bytes` |
| S06 silence | `output.exact-silence` |
| S06 edge | `output.frames,output.finite,output.direct-edge` |
| S07 | `output.frames,output.finite,linked.<fixture>` |
| S08 capacity | `memory.plan-bound` |
| S08 instrumented | `memory.plan-bound,memory.processing-allocation-zero,output.frames` |
| S08 repeat | `output.byte-repeat` |
| S08 oracle | `memory.<oracle>` except `schedule.scalar-traversal` |
| S09 | `absence.<case>` |
| Y01 continuous | `output.frames,output.finite,output.peak-bound,output.silence-where-unread,output.dropout,output.exterior-difference,diagnostics.complete` |
| Y01 sparse | `output.frames,output.finite,output.peak-bound,output.silence-where-unread,output.exterior-difference,ledger.outside-zero,ledger.event-present,ledger.order,ledger.centre,diagnostics.complete` |
| Y02 | `output.frames,output.finite,output.peak-bound,output.dropout,output.exterior-difference,pitch.finite,diagnostics.complete` |
| Y03 | `output.frames,output.finite,output.peak-bound,output.exterior-difference,ledger.outside-zero,ledger.event-present,ledger.order,ledger.centre,diagnostics.complete` |
| Y04 | `output.frames,output.finite,output.peak-bound,output.dropout,output.exterior-difference,cadence.finite,diagnostics.complete` |
| Y05 | `output.frames,output.finite,output.peak-bound,output.dropout,output.exterior-difference,gap.measured,pad.no-false-gap,diagnostics.complete` |
| Y06 | `output.frames,output.finite,output.peak-bound,output.exterior-difference,linked.<fixture>,linked.balance-bound,diagnostics.complete` |
| E16 | `request.error-variant,allocation.output-zero,render.count-zero` |
| M01/M02 | `output.frames,output.finite,artifact.raw-written,diagnostics.complete` |

`<fixture>`, `<oracle>`, and `<case>` are the exact row slugs. A row omits an
inapplicable assertion; no `not_applicable` status exists. Assertions after a
failure remain present as `not_run`.

For `Y05`, `silence-gap` includes `gap.measured` and omits
`pad.no-false-gap`; `harmonic-pad` does the reverse. For `S01`,
`output.identity-bytes` occurs only on `mono-identity`. For `S07/Y06`,
`linked.<fixture>` is the single exact fixture slug for that row.

Receipt `expected` strings use only:

- `true`, `false`, `finite`, `byte-exact`, or an exact error variant
- `exact:<decimal>` for equality after the independent oracle evaluates the
  frozen formula
- `max:<decimal>` for an inclusive upper bound
- `range:<decimal>..<decimal>` for an inclusive interval

`output.frames` is `exact:T`; peak is
`max:input_peak+0.000002`; ledger outside is `max:0.0000001`; samplewise linked
mechanics are `max:0.000001`; gain is `max:0.01` dB; delay is
`range:12..14`; correlation loss is `max:0.02`; long-form balance is
`max:comparator_error+0.50`; exterior difference is
`max:comparator_db+1.50`. Formula-owned structural values use `exact:` plus the
independent rational/window/interpolation result rendered with shortest
round-trip formatting. Construction rejects any other expected vocabulary.

The Y01 sparse rows are exactly local ordinals `012..017`: `impulse` then
`impulse-train`, each at ratios `2,4,8` and `c048000`. The other `24` Y01
rows are continuous. Row IDs, global ordinals, render counts, and the
`588`-row manifest total do not change.

Structural diagnostics are empty. In the lists below, `both exterior values`
expands exactly, in order, to
`exterior.candidate-db,exterior.comparator-db`. Synthetic diagnostic IDs are:

- `Y01` continuous: `output.raw-peak`, `dropout.count`, `dropout.minimum-active-rms-db`,
  `source.maximum-window-rms-db`, `exterior.candidate-db`,
  `exterior.comparator-db`
- `Y01` sparse: `output.raw-peak`, `exterior.candidate-db`,
  `exterior.comparator-db`, then the exact Y03 event expansion for the row's
  authored event count
- `Y02`: `output.raw-peak`, `dropout.count`,
  `dropout.minimum-active-rms-db`, `source.maximum-window-rms-db`,
  `exterior.candidate-db`, `exterior.comparator-db`, then for authored component
  index `cc` in ascending frequency order,
  `pitch.cCC.frequency-hz`, `pitch.cCC.input-cents`,
  `pitch.cCC.comparator-cents`, `pitch.cCC.magnitude`,
  `pitch.cCC.interpolated-bin`, `pitch.cCC.resolution-cents`
- `Y03`: `output.raw-peak`, both exterior values, then for event index `ee`,
  `event.eEE.appearance-count`, `event.eEE.spacing-min`,
  `event.eEE.spacing-median`, `event.eEE.spacing-max`,
  `event.eEE.weighted-centre`, `event.eEE.extent-start`,
  `event.eEE.extent-end`, `event.eEE.peak`, `event.eEE.energy`,
  `event.eEE.cropped-leading`, `event.eEE.cropped-trailing`,
  `event.eEE.comparator-centre-delta`
- `Y04` common: `output.raw-peak`, both exterior values, `cadence.planned-spacing`,
  `cadence.measured-spacing`, `modulation.strongest-frequency-hz`,
  `modulation.strongest-strength`, `autocorrelation.lag`,
  `autocorrelation.value`, `cadence.comparator-spacing`,
  `cadence.spacing-delta`, `modulation.comparator-frequency-hz`,
  `modulation.frequency-delta-hz`, `autocorrelation.comparator-value`,
  `autocorrelation.value-delta`
- `Y04` am-noise inserts, immediately after
  `modulation.strongest-strength`, `modulation.authored-frequency-hz` and
  `modulation.authored-strength`; uniform and Rademacher rows omit them
- `Y05` silence-gap: `output.raw-peak`, both exterior values,
  `gap.support-start`, `gap.support-end`, `gap.support-length`,
  `dropout.count`, `tail.last-active-frame`, `tail.inactive-frames`,
  `gap.comparator-support-length`, `gap.support-length-delta`,
  `tail.comparator-inactive-frames`, `tail.inactive-frames-delta`
- `Y05` harmonic-pad: `output.raw-peak`, both exterior values,
  `gap.false-run-count`, `gap.maximum-zero-run`, `dropout.count`,
  `tail.last-active-frame`, `tail.inactive-frames`,
  `gap.comparator-maximum-zero-run`, `gap.maximum-zero-run-delta`,
  `tail.comparator-inactive-frames`, `tail.inactive-frames-delta`
- `Y06`: `output.raw-peak`, both exterior values; source, candidate,
  comparator, candidate-source error, comparator-source error, and
  candidate-comparator delta for `balance.whole`, bands `low,mid,high`,
  `width.whole`, and the same bands; `correlation.lag`,
  `correlation.candidate-peak`, `correlation.comparator-peak`; and
  `window.wWW.<balance fields>` for every complete mapped window in ascending
  output order
- `M01/M02`: `source.rms`, `candidate.rms`, `comparator.rms`,
  `level.source-gain`, `level.candidate-gain`, `level.comparator-gain`,
  `level.safety-scale`, `output.raw-peak`

`CC`, `EE`, and `WW` are zero-based two-digit indices. `Y04`'s gate summary
owns `cadence.strict-cycle-order` for each source/ratio. Component counts are
tone `1`, chord `5`, pad `8`; event counts are impulse `1`, train `4`.

For each `Y06` balance or width prefix `p`, `<fields>` expands exactly to
`p.source-db`, `p.candidate-db`, `p.comparator-db`,
`p.candidate-source-error-db`, `p.comparator-source-error-db`,
`p.candidate-comparator-delta-db`. Window prefixes are
`window.wWW.balance`; width is not recomputed per window. Band prefixes are
`balance.band-low`, `balance.band-mid`, `balance.band-high`,
`width.band-low`, `width.band-mid`, and `width.band-high`.

Diagnostic units are exact:

- frequencies `Hz`; cents and resolution `cent`
- RMS and exterior values `dBFS`; balance, width, and their errors/deltas `dB`
- peaks, magnitudes, modulation strength, autocorrelation, and correlation
  peaks `linear`
- RMS, gain, and safety scale `linear`
- energy `sample2`; interpolated bin `bin`
- count fields `count`
- all positions, lags, extents, support, tail, and spacing values `frame`

Candidate/comparator frequency deltas use `Hz`; autocorrelation deltas use
`linear`. Construction derives the one-to-one unit list from these rules and
binds it beside the diagnostic-ID list.

### Executable Owner Known Answers

Construction calls the real evidence functions with the following vectors.
Every listed expected value is a hard construction assertion. Integer,
support, order, ID, and hash results are exact. Floating results use maximum
absolute error `1e-9` unless a tighter bound is named. No known-answer oracle
may call the function it checks.

#### FFT And Pitch Owner

The FFT backend first runs two independent `N=8` DFT known answers:

- `[1,0,0,0,0,0,0,0]` produces `(1,0)` in every bin
- `cos(2*pi*n/8)` produces `(4,0)` in bins `1` and `7` and zero elsewhere

Real and imaginary component error is at most `1e-12`. Expected values come
from the direct DFT definition
`X[k]=sum(x[n]*(cos(2*pi*k*n/N)-i*sin(2*pi*k*n/N)))`; the oracle is not an
FFT.

The complete pitch estimator then receives `F=262144`, `M=4096`, and
`x[n]=0.5*cos(2*pi*64*n/M)`, with the same vector used as source, candidate,
and comparator. It must choose bin `4096`, return interpolated bin `4096`,
frequency `4096 Hz`, both cents values `0`, and resolution
`1200*log2(4097/4096)`. Bin, frequency, and cents error is at most `1e-9`;
resolution error is at most `1e-12`. Construction also changes only the
candidate tone to `4097 Hz` and requires non-zero input and comparator cents.
This prevents authored-frequency or constant-zero substitution.

#### Event-Ledger Owner

The exact rejected-row geometry is a permanent known answer:

- `F=44100`, `L=88200`, `T=176400`, `cycle_us=48000`, `H=2117`
- one impulse at source event `44100`
- ideal mapped centre `88200.5`
- four positive half-open appearances:
  `[86440,86442)`, `[87498,87500)`, `[88557,88559)`,
  `[89615,89617)`
- absolute-weighted appearance centres `86440.24897416269`,
  `87498.74984046511`, `88557.25007537556`, and
  `89615.75048508307`
- spacing minimum `1058.500234910447`, median
  `1058.5004097075143`, and maximum `1058.50086630242`
- aggregate absolute-weighted centre `88217.5135681342`
- extent `[86440,89617)`, peak contribution `0.6985939636456214`,
  total contribution energy `0.9373787150738565`
- zero leading and trailing cropped appearances

Support and counts are exact; floating contribution values allow `1e-6` to
cover the supported platform's `cos` implementation. The oracle must also
prove `[88179,88400)` has no ledger support although its ideal-map source
range is `[44090,44199]`. This is the ownership distinction which creates the
fresh identity.

A second vector uses authored events `1` and `4`, `L=6`, `T=12`, `H=3`.
Construction derives both ledgers, requires non-empty ordered aggregate
centres, rejects one injected sample of value `2e-7` outside their union, and
accepts `1e-7`. Reversing the two measured centres must fail `ledger.order`.
Replacing either event's ledger samples with zero must fail
`ledger.event-present`. These mutations run through the real Y01/Y03
assertion dispatcher.

#### Cadence Owner

The RMS-envelope builder receives `3072` samples of value one. Its fixed
`2048/512` geometry must return exactly three values, all `1`.

The cadence FFT receives the `64`-value mean-zero envelope
`e[j]=cos(2*pi*j/8)` at envelope rate `44100/512 Hz`, with
`N=16384`. It must select bin `2048`, frequency
`10.7666015625 Hz`, and a strength equal within `1e-12` to an independent
direct-DFT computation over every searched bin. The unwindowed
autocorrelation search with planned lag `8` spans `6..=10`; it must return
lag `8` and value `0.875`. Frequency error is at most `1e-12`;
autocorrelation error is at most `1e-12`.

Construction repeats the analysis with periods `16`, `8`, and `4` envelope
frames and requires measured output spacing `8192`, `4096`, and `2048`
frames respectively. It then presents them to the cycle-order summary in
short, neutral, long order as `2048,4096,8192`; the summary must pass.
Swapping short and long must fail `cadence.strict-cycle-order`.

#### Gap, Dropout, Tail, And Exterior Owner

For hull `[3,14)`, the vector with non-zero samples only at indices
`3,4,9,13` must return maximal exact-zero support `[5,9)` of length `4`.
Adding a second length-four run later in the hull must retain the earlier run.
An all-non-zero hull returns `none`; it must fail `gap.measured`. The pad
control over the same hull must return false-run count `0` and maximum run
length `0`, satisfying `pad.no-false-gap`. Injecting one `221`-sample zero run
must return count `1`, maximum `221`, and fail that assertion.

For identity-map source and output length `442`, source value one everywhere,
and candidate exact zero only over `[110,331)`, the `221`-frame dropout scan
must report exactly one eligible dropout beginning at `110`. Changing one
sample in that run to `1e-3` must report zero. The scan must still find the
row when an unrelated partition marker is placed at frame `221`; block
boundaries do not reset it.

For `[0,0.25,0,0.5,0,0]`, tail analysis returns last active frame `3` and two
inactive terminal frames. Exterior first difference over `[0.25,-0.5]`
includes `0->0.25`, `0.25->-0.5`, and `-0.5->0`; its maximum is exactly
`0.75`.

#### Linked-Stereo Owner

Run linked mechanics on left `[0.25,-0.5,0.75,-1]` and right
`[-0.125,0.375,-0.625,0.875]`. Duplicate error, anti-phase error,
common-negation commutation error, and swap commutation error must each be
exact zero. Mutating one right sample by `2e-6` must fail the `1e-6` bound.

For right gain `0x3f004dce`, whole-render right/left ratio must be
`-6.0000004906757844 dB`. A separate `8192`-sample fixture at `8192 Hz`
contains bin-centred `128`, `1024`, and `2048 Hz` left tones and the same
right gain. Whole, low, mid, and high ratios must each match that value within
`1e-8 dB`.

For `n=0..255`, the deterministic non-periodic sequence is
`f32((((mix64(n xor 0x4c494e4b45444b41)>>11)/2^53)*2-1)*0.5)`.
Use it as left and a right copy delayed by two frames with exterior zero.
Normalized correlation over
`-4..=4` must return lag `+2`; positive lag means right lags left. Swapping
channels must return `-2`. Duplicate channels must return `0` and peak `1`
within `1e-12`. These calls use the real Y06 balance, band, and correlation
owners.

The `8192`-frame band fixture is exactly
`left[n]=f32(0.2*sin(2*pi*128*n/8192)+0.15*cos(2*pi*1024*n/8192)+0.1*sin(2*pi*2048*n/8192))`;
right is the exact `f32` gain multiplication already frozen above.

#### Level And Listening-Copy Owner

Source `[1,-1]`, candidate `[0.5,-0.5]`, and comparator `[2,-2]` have RMS
`1`, `0.5`, and `2`. The level owner must return gains `1`, `2`, and `0.5`,
common safety `0.95`, and final peak `0.95` for every stream. Any independent
per-stream peak normalization fails.

The terminal-fade helper receives eight ones and `N=4`. It must leave the
first four samples at one and produce tail gains `1`, `2/3`, `1/3`, `0`.
The raw input hash must remain unchanged. Reapplying the fade to the listening
copy must change its hash and fails the one-application owner.

#### Comparator Owner

The canonical mono IEEE-float WAV vector `[0,1,-1,0.5]` at `44100 Hz` has
PCM SHA-256
`bdc1999ac422c4823b4304d698512f70b2069ab5ea7976e4b0af9f75b4a9a3cd`
and container SHA-256
`4b75446080d2cbbc91dc71a44b999b274b66d92738ecd784c964b92b5f9c2478`.
Construction must generate those exact `60` bytes, decode them, and reproduce
both hashes.

Using that source hash, `L=4`, `T=8`, ratio `2`, source path
`/signal-known-answer/source.wav`, and render path
`/signal-known-answer/output.wav`, the exact normalized project projection
defined below hashes to
`2ad07d3e5a29d08b9cf9b96f333d7af874cd9337984014f29db0c9866b068fb4`.
The project generator must emit a project which the independent parser maps
to that hash and every expected field. Changing play rate, target frames,
stretch mode, fade, channel count, or either path must change the projection
and fail the field comparison.

Finally, the comparator manifest join receives rows `KA-C-000` and
`KA-C-001`. The first owns the canonical WAV above. The second owns the same
container with samples negated before encoding; its hashes are derived by the
independent known-answer builder. The two matching files must pass in row
order, then separately reject a missing row, duplicate `KA-C-000`, unknown
`KA-C-002`, container-hash mismatch, PCM-hash mismatch, wrong frame count,
wrong channel count, and a file whose first PCM word is quiet NaN. The real
pre-checkpoint comparator verifier owns every case.

### Summary Authority

The summary test full name ends `event_ledger_audited_centered_cyclic_summary`. Its compact
JSON keys are exactly:

`schema`, `identity`, `checkpoint`, `tree`, `phase`, `scope`,
`manifest_sha256`, `row_count`, `pass_count`, `fail_count`, `panic_count`,
`incomplete_count`, `planned_render_count`, `completed_render_count`,
`terminal_concat_sha256`, `receipt_set_sha256`, `assertions`, `error`.

Schema is `signal.event-ledger-audited-centered-cyclic.summary.v2`. `scope` is
`structural`, `Y01` through `Y06`, `exact16`, `mono-render`, or
`stereo-render`. `terminal_concat_sha256` hashes terminal JSON lines including
their LF in manifest order. `receipt_set_sha256` hashes, in the same order,
`row_id<TAB>SHA256(receipt-file)<LF>`.

Structural pass requires `339/339`, `168/168`, no fail/panic/incomplete, and
all receipt assertion maps exact. `Y04` summary adds the nine ordered
`cadence.strict-cycle-order` assertions. Other summaries add only count,
render, manifest, and receipt-completeness assertions. A summary uses
`create_new`, flush, and `sync_all`; an existing summary blocks rerun.

Every summary's ordered assertion IDs are exactly:

1. `summary.manifest-sha256`
2. `summary.row-count`
3. `summary.pass-count`
4. `summary.fail-count`
5. `summary.panic-count`
6. `summary.incomplete-count`
7. `summary.planned-render-count`
8. `summary.completed-render-count`
9. `summary.receipt-assertion-maps`
10. `summary.terminal-concat-sha256`
11. `summary.receipt-set-sha256`

Y04 appends
`cadence.strict-cycle-order.<source>.r<ratio>` in source order
`uniform-noise,rademacher-noise,am-noise`, then ratio order `2,4,8`.
Summary assertions use the row assertion object keys and status vocabulary.
`expected` and `actual` are exact decimal counts, lowercase SHA-256, or
`true`. A summary has no diagnostics.

### Runner And Sentinel Authority

The only runner environment names are:

- `SIGNAL_EVENT_LEDGER_AUDITED_CYCLIC_ROW_ID`
- `SIGNAL_EVENT_LEDGER_AUDITED_CYCLIC_RECEIPT_ROOT`
- `SIGNAL_EVENT_LEDGER_AUDITED_CYCLIC_PHASE`
- `SIGNAL_EVENT_LEDGER_AUDITED_CYCLIC_CHECKPOINT`
- `SIGNAL_EVENT_LEDGER_AUDITED_CYCLIC_TREE`
- `SIGNAL_EVENT_LEDGER_AUDITED_CYCLIC_MANIFEST`
- `SIGNAL_EVENT_LEDGER_AUDITED_CYCLIC_COMPARATOR_MANIFEST`
- `SIGNAL_EVENT_LEDGER_AUDITED_CYCLIC_ARTIFACT_ROOT`
- `SIGNAL_EVENT_LEDGER_AUDITED_CYCLIC_SUMMARY_SCOPE`
- `SIGNAL_EVENT_LEDGER_AUDITED_CYCLIC_SENTINEL`
- `SIGNAL_EVENT_LEDGER_AUDITED_CYCLIC_SENTINEL_ACK`

Receipt roots are:

`<ignored-root>/receipts/<checkpoint>/<execution-id>/<gate>`.

Conformance execution IDs are `conformance-round-1` and
`conformance-round-2`; receipt field `phase` is `conformance` in both.
Acoustic IDs are the exact gate names. The runner requires the execution root
to be absent, creates it once, and never removes it.

Compile runs through:

```text
effigy test compile -- --release --target-dir <ignored-root>/build
```

Every row runs through Effigy's configured nextest suite:

```text
effigy test nextest -- --release --target-dir <ignored-root>/build --profile event-ledger-audited-centered-cyclic -E test(=<full-entrypoint>)
```

The tracked shell runner supplies the row environment and invokes that argv
through `/usr/bin/python3` `subprocess.run(..., timeout=600, check=false)`.
Timeout, signal, non-zero exit, a result other than exactly one passed test, or
missing terminal receipt stops the runner. It never retries. The runner first
requires clean `git status --porcelain`, exact expected HEAD/tree, matching
manifest hashes, and an absent execution root.

Construction spawns the current libtest executable with the exact full
construction test name and sentinel values `fail`, `panic`, and `kill`.
Sentinel row IDs are `C00-sentinel-fail`, `C00-sentinel-panic`, and
`C00-sentinel-kill`. Each child enters the real wrapper. `fail` returns a
`Result` error; `panic` panics inside the body; `kill` writes and syncs
`started`, then creates and syncs the ACK file and blocks.

The parent polls ACK every `10 ms` for at most `5 s`, calls `kill`, waits, and
requires exactly one started line. It reruns each sentinel once against the
same path, requires `create_new` failure, and proves the receipt hash did not
change. Expected sentinel JSON is built by an independent test oracle that
does not call the writer or escaper.

### Comparator Manifest V2

The inaccessible checkpoint manifest hash
`eb5384681767dfd36e8daf81809a95d51a79f6cb178f0705fe4cffce9ecccacd`
is retired. It remains historical evidence only. The same applies to the old
mono ReaReaRea `4x/8x` hashes and the `48`-row group hash
`5bb7b55456065d8f3d69c7229abc117eacb9280cf298a779b634598a19663e11`.
They do not gate the fresh identity.

The new `63` rows are exactly:

1. five retained native-stereo musical sources × ratios = `15`
2. ten mono synthetic sources × ratios at the neutral cycle = `30`
3. six stereo fixtures × ratios at the neutral cycle = `18`

Order is group, source order, then ratio. IDs are
`C-M-<source>-r<ratio>`, `C-Y-<source>-r<ratio>`, and
`C-ST-<source>-r<ratio>`. The one native-stereo musical comparator output is
downmixed for mono review and reused natively for stereo review.

The comparator header is exactly:

```text
schema	ordinal	row_id	source_id	source_kind	source_container_sha256	source_pcm_sha256	source_frames	channels	sample_rate	ratio_num	ratio_den	target_frames	reaper_identity	project_semantics_sha256	project_container_sha256	output_container_sha256	output_pcm_sha256
```

Schema is `signal.event-ledger-audited-centered-cyclic.comparator.v2`. All hashes are
mandatory. Long-form source containers must match the five hashes already
frozen above. Synthetic source containers are canonical RIFF/WAVE: `RIFF`,
size `36+data_bytes`, `WAVE`, one `fmt ` chunk of size `16`, IEEE float format
`3`, native channel count, `44100 Hz`, byte rate
`44100*channels*4`, block align `channels*4`, `32` bits, then one `data` chunk
and little-endian interleaved `f32` samples. No metadata, padding, or extra
chunk exists.

REAPER identity is exactly `REAPER-7.69-macOS-arm64-ReaReaRea-983040`. The
project generator must emit a project which, when reparsed, produces this
normalized LF projection in exact key order:

```text
sample_rate=44100
source_path=<absolute-source-path>
source_container_sha256=<hash>
source_frames=<L>
source_channels=<C>
item_position_frames=0
item_length_frames=<T>
playrate_num=1
playrate_den=<ratio>
preserve_pitch=1
stretch_mode=983040
stretch_mode_field=0.0025
fade_in_frames=0
fade_out_frames=441
render_start_frames=0
render_end_frames=<T>
render_channels=2
render_format=wav-pcm24
render_path=<absolute-output-path>
```

`project_semantics_sha256` hashes that projection including final LF.
`project_container_sha256` records the full generated RPP as diagnostic
identity; no predeclared RPP hash exists because absolute paths are part of
the project. Construction reparses every RPP and compares every normalized
field before candidate source may exist.

Invoke REAPER only as:

```text
/Applications/REAPER.app/Contents/MacOS/REAPER -newinst -nosplash -renderproject <absolute-project-path>
```

Every output must be stereo `24-bit` PCM at `44100 Hz`, exactly `T` frames,
finite after `f32` decode, and have a non-empty container and PCM hash. A
fresh complete manifest hash is recorded in the candidate checkpoint; it is
derived from this frozen generator, not chosen as a target.

### Listening Files And Decisions

The listening-manifest TSV header is exactly:

```text
schema	ordinal	row_id	review	listener_class	source_id	ratio_num	cycle_set	artifact_a	artifact_b	artifact_c	decision_schema
```

Schema is `signal.event-ledger-audited-centered-cyclic.listening-manifest.v2`. Rows are:

- `L01`: neutral mono A/B, ratio order `8,4,2`, then musical source = `15`
- `L02`: short/neutral/long mono trio, musical source then ratio = `15`
- `L03`: neutral stereo A/B, ratio order `8,4,2`, then musical source = `15`
- `L04`: stereo fixture A/B at `8x`, neutral cycle, fixture order = `6`

IDs are `L01-<local-index>-neutral-mono-<source>-r<ratio>`,
`L02-<local-index>-direction-mono-<source>-r<ratio>`,
`L03-<local-index>-neutral-stereo-<source>-r<ratio>`, and
`L04-<local-index>-fixture-<fixture>-r8`.

Public pack manifest header is:

```text
schema	row_id	source_id	ratio_num	cycle_set	artifact_a	artifact_b	artifact_c
```

The private key header is:

```text
schema	row_id	a_role	b_role	concealment_sha256
```

Their schemas are respectively
`signal.event-ledger-audited-centered-cyclic.pack.v2` and
`signal.event-ledger-audited-centered-cyclic.key.v2`. File names are
`<row_id>-A.wav` and `<row_id>-B.wav`; `artifact_c` is `-`. The existing
concealment hash law owns A/B assignment. `L02` has no key row; its artifacts
are
`<row_id>-short.wav`, `<row_id>-neutral.wav`, and `<row_id>-long.wav` and have
no comparator identity.

Each append-only decision JSON line has exact key order:

`schema`, `checkpoint`, `listener_id`, `listener_class`, `row_id`, `sequence`,
`status`, `recognizable_cyclic`, `musically_useful`, `preference`, `usable`,
`metallic_direction`, `echo_direction`, `click`, `dropout`, `level_step`,
`buzz`, `lost_event`, `doubled_attack`, `centre_pull`, `one_sided_texture`,
`width_pumping`, `side_inversion`, `detached_echo`, `channel_local_cycle`,
`balance_movement`, `note`, `error`.

Schema is `signal.event-ledger-audited-centered-cyclic.decision.v2`. Started records use
sequence `0`, status `started`, and null decisions. Terminal records use
sequence `1`, status `recorded`, all applicable fields non-null, and null
error. `listener_class` is `operator` for `L01/L02` and
`eligible-independent` for `L03/L04`. Inapplicable fields are null. Preference
is `A`, `B`, or `tie`; other decisions are JSON booleans. Notes use the
canonical escaper.

Each row owns `<artifact-root>/decisions/<row_id>.jsonl`, created with
`create_new`; it contains exactly the started and recorded lines. `listener_id`
is lowercase SHA-256 of UTF-8
`listener_class + "|" + operator-supplied-local-label`; the label itself is
not persisted.

The speaker pre-screen is one file with row ID
`L05-000-speaker-prescreen`, listener class `operator`, and the same durable
two-line schema. It records only `usable` and the stereo artifact fields.

The reveal executor requires every decision terminal line, hashes the decision
files, and writes `revealed-key.tsv` once with `create_new` from the private
key bytes. A reveal receipt binds every pre-reveal decision-file hash and
rejects any later change. Listening summaries use the existing pass counts and
artifact vetoes without reinterpretation.

### Batch 32.10 Result

The exact fresh comparator set, manifest grammars, row expansion,
assertion/diagnostic identities, summary bytes, Effigy runner behavior,
sentinel handshake, and listening decision surface are now frozen without
recovering deleted state. Candidate implementation has no remaining manifest
choice.

### Batch 32.11 Result

The fresh evidence identity was generated before candidate source. Its frozen
manifest contains `588` rows. Fresh REAPER capture bound `63` comparator rows:
`15` native-stereo musical, `30` mono synthetic, and `18` stereo synthetic.

Private candidate checkpoint `74a6d6d9` implements the frozen renderer and
every compile-linked evidence owner. Release compile and construction `1/1`
passed. Both unchanged structural rounds passed all `339` rows and `168`
planned renders with byte-identical receipts and summaries. The structural
summary hash is
`75a2e5ed5c1406d9790a5ba904d7ce8d8e5c4dc459787b8e0dac6a1d761b43c2`.

The clean checkpoint is frozen at
`refs/signal-evidence/creative/audited-centered-compressed-anchor-cyclic/32-11-acoustic`.
No acoustic, exact-`16x`, long-form, or listening row has run. No candidate
source or evidence harness entered `main`.

### Batch 32.12 Result

The first `Y01` invocation stopped at
`Y01-000-low-tone-r2-c048000`. The test process wrote a passing two-line
receipt below a crate-relative duplicate evidence root. The shell runner
looked below the repository-relative root, found no receipt, and exited `66`.

The intended root contains only environment identity files. No `Y01` summary,
later synthetic row, exact-`16x` row, long-form render, or listening row
exists. The misplaced receipt hash is
`f9c12e26ca6d7e727749ae12e70e86262816715abad66850396ea6fdc4596d91`.

The out-of-root pass does not admit the row. No retry is allowed. This is an
evidence-path ownership failure, not an acoustic quality result.

### Batch 32.13 Result

Contract `085` Rule 11 classifies this as the second incomplete-evidence
checkpoint for centred compressed-anchor Cyclic. The identity is closed
without an acoustic pass or rejection. No runner repair, retry, third audited
identity, candidate promotion, or product route is allowed.

The isolated worktree, branch, build state, and generated evidence are gone.
The local acoustic ref is deleted after the closure commit. Candidate source
and evidence scaffolding never entered `main`.

### Batch 32.14 Operator Correction

The operator rejects renderer closure on a caller-supplied relative-root
mistake. The first row passed every assertion. No renderer, source, metric,
threshold, owner, manifest, comparator, or dependency byte may change.

Restore exact checkpoint `74a6d6d9`, tree `d519e2d8`, and invoke the unchanged
runner once with absolute ignored root
`/Users/tom/Dev/projects/signal-candidate-32-11/target/creative-stretch-audited-centered-compressed-anchor-cyclic-32-11`.
Run the complete `Y01` gate and stop before `Y02`.

### Batches 32.15 Through 32.17

Batch 32.15 stopped before DSP because cleanup had removed generated
comparator assets. Batch 32.16 restored and hash-verified the exact frozen
comparator environment, then the unchanged checkpoint passed `12` Y01 rows
and failed `Y01-012-impulse-r2-c048000` on one unexpected dropout.

Batch 32.17 independently reconstructed the failing row. The impulse remained
present in four commanded ledger groups; the zero window lay between them.
That batch selected sparse-event ledger ownership and proved the rejected Y02,
Y03, and Y04 helpers were placeholders. The checkpoint, harness, and evidence
implementation remain prohibited.

### Batch 32.18 Result

This brief is the sole current centred compressed-anchor Cyclic authority.
The renderer remains unchanged. Sparse Y01 event integrity now belongs to the
independent event ledger; continuous material retains the original dropout
gate. All `588` row IDs and counts remain fixed. Assertion ownership changes
only for Y01 rows `012..017`. Diagnostic vectors are now exact and omit
inapplicable authored-modulation or absent-gap fields.

Construction now has executable known answers for the real FFT and pitch
estimator, event ledger, cadence FFT and autocorrelation, gap/dropout/tail and
exterior analysis, linked stereo, listening level/fade policy, and comparator
generation and verification. None may be replaced by an authored value,
label, pointer-presence test, or self-comparison.

No candidate worktree, source, evidence harness, comparator render, or
acoustic receipt was created. Batch 32.19 may begin isolated implementation
and conformance from this exact docs closeout.

### Batch 32.19 Pre-Source Authority Audit

The first exact isolated worktree remained clean. Before evidence generation
or candidate source, audit found the sustained-source ramp endpoint law,
numeric evidence primitives, band aggregation, correlation support,
summary-assertion IDs, and three known-answer vectors were not executable
without new choices.

This correction freezes all of them together. No manifest, comparator project,
comparator render, candidate source, receipt, or DSP output existed. Delete
the empty isolation and recreate the same name from this correction commit.
This is an authority completion, not a candidate repair or rerun.

### Batch 32.19 Result

Fresh evidence commit `6a909c74` bound the complete manifests and regenerated
comparator environment before candidate source. Private implementation commit
`08e5c57c` added the renderer and evidence owners.

The first checkpoint stopped before construction, row dispatch, or DSP because
the mandated Python 3.10 runner rejected `zip(strict=True)`. Its execution root
contains zero receipts. Evidence-only commit `995ea516` replaces that
Python 3.11-only call with an explicit field-count check; renderer and test
logic are unchanged.

Checkpoint `995ea516`, tree `fd42543b`, passes release compile, construction
`1/1`, and all `339` structural rows twice with `168/168` renders. Corresponding
receipt files and summaries are byte-identical. Structural summary SHA-256 is
`f1e90cd36557d1c1b6ef3be70175b7f025cdd00f1d7405a7a7958cf8a91cb08b`.
The local `32-19-acoustic` ref points to that checkpoint. No Y01 or later row
ran.

### Batch 32.20 Result

Preflight resolved the exact ref, checkpoint `995ea516`, tree `fd42543b`, and
clean candidate worktree. The checkpoint cannot execute the frozen Y01 lane:

- tracked runner SHA-256
  `ce4ca173dd3892a85ffa4eb8e5369263d3a96a62204e2f2ce41804f076596fe9`
  accepts only the two conformance-round IDs
- the summary owner selects only conformance rows, hardcodes scope
  `structural`, and writes only `summary/structural.json`
- no separately frozen acoustic runner exists

No runner invocation, Y01 receipt, summary, or acoustic render occurred. This
is incomplete executable evidence under Contract `085` Rule 11, not an
acoustic result. The checkpoint and local ref remain immutable for the
complete evidence-integrity reassessment.

### Batch 32.21 Result

The complete checkpoint audit found no bounded runner-only correction:

- construction did not execute the frozen post-checkpoint owners it claimed
- receipt assertions were synthesized as pass from row success rather than
  evaluated by assertion owner
- no acoustic gate runner or non-structural summary path exists
- Y01/Y03 use an approximate event support, not the frozen independent ledger
- Y02 pitch, Y04 cadence, Y05 gap, and most Y06 linked measurements are absent,
  generic zero, or semantically wrong
- exact-`16x` lacks the frozen allocation proof
- comparator project identity and normalized project semantics are not checked
- long-form level/fade copies, concealment, decisions, speaker pre-screen,
  reveal, and listening summaries do not exist

Evidence implementation SHA-256:

`a1a7ae3d96c303652ce2f0e19f36b73c5ff7bde7b603bbfa796a25919612b1ae`

This repeats the incomplete executable-owner class found in the previous
audited centred compressed-anchor checkpoint. The fresh event-ledger authority
was the Rule 11 correction for that exact class. A further evidence-only
identity is therefore prohibited protocol churn.

Batch 32.21 closed the family under the then-current Rule 11. Checkpoint
`995ea516` had no valid acoustic pass or rejection, and its renderer had no
listening judgment. Batch 32.23 supersedes that closure below.

### Batch 32.22 Result

The exact local acoustic ref, candidate branch, worktree, ignored build state,
comparator assets, receipts, and artifacts are deleted. `main` contains no
candidate DSP or evidence implementation.

### Batch 32.23 Operator Completion Correction

The renderer remains acoustically unjudged. The operator explicitly rejects
evidence-protocol exhaustion as product completion and authorizes exact
recovery of checkpoint `995ea516`, tree `fd42543b`.

Recover only the checkpoint's candidate module, tests, manifests, generators,
runner, nextest configuration, and required `lib.rs` module binding onto a
fresh branch from current `main`. Do not restore checkpoint-era canonical
docs. The recovered renderer bytes are the initial acoustic subject.

Evidence implementation is now repairable infrastructure. It must:

- execute every frozen assertion and diagnostic rather than infer pass from
  row success
- expose structural, synthetic, exact-`16x`, long-form, mono, stereo,
  decision, and reveal stages
- write fail-durable row receipts and correctly scoped summaries
- implement the independent event ledger, measured pitch, cadence, gap,
  stereo, level, fade, and comparator owners frozen in this brief
- create level-matched faded concealed packs and immutable decisions
- keep renderer formulas, sources, comparator semantics, metrics, thresholds,
  and assertion meaning unchanged while infrastructure is repaired

Each evidence correction is committed and rerun from a clean evidence root.
No evidence failure closes the renderer. A valid acoustic receipt may reject a
renderer version; listening decides promotion.

## Admission Result

Batch 32.24 recovered the renderer exactly and repaired evidence
infrastructure without changing the acoustic implementation. Two complete
structural rounds passed `340/340`. Checkpoint
`bab6ce96b0476e025dce5c957d91eab27e375fd6` then passed:

- synthetic admission: `183/183`, `201` planned renders
- exact `16x` rejection: `5/5`, before output allocation
- long-form mono: `45/45`
- long-form linked stereo: `15/15`

The operator judged the concealed mono and speaker-stereo comparisons hard to
distinguish, similar, and solid. After every hard stereo gate passed, the
operator explicitly waived independent review for this exact fixed `2x`,
`4x`, and `8x` creative renderer. The one-ear hearing limitation remains
recorded. The waiver does not cover another implementation, ratio, character,
automatic route, dynamic path, or transparent stretch.

Batch 32.26 admits the unchanged renderer privately as `creative_cyclic` in
commit `81edaada`. The plan, schedule, interpolation, and synthesis files are
byte-identical to the accepted checkpoint. The candidate evidence system and
listening assets remain isolated.

## Residual Renderer Risks

- direct two-read crossfade may create audible gaps on non-sparse material
- compressed-anchor replicas may sound too pitch-shifted, metallic, or regular
- long cycles may become detached echo; short cycles may become buzz
- zero exterior reads may create material-dependent onset or tail energy
- one fixed linked schedule may preserve algebra while sounding narrow or
  spatially detached
- fresh native-stereo ReaReaRea captures may differ from the historical mono
  comparator behavior
- the expanded one-row protocol has high process overhead and must still fit
  its frozen envelope
- the scoped operator waiver does not establish an eligible independent
  stereo judgment

These are admission risks, not open design choices.

## Next Task

Keep this admitted renderer and public ratios frozen. Batch 34.1 selected
`ContinuousEventLedgerCyclic` over `2N..=8N` as a static-compatible evidence
candidate, not acoustic admission. Execute Batch 34.2 only as documentation.
