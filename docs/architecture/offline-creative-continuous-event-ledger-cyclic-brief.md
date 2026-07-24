# Offline Creative ContinuousEventLedgerCyclic Brief

Status: frozen; isolated execution ready
Owner: dsp
Updated: 2026-07-24
Contract: `085`
Roadmap: `g10.034`, Batch 34.2
Parent:
[EventLedgerAuditedCenteredCompressedAnchorCyclic](./offline-creative-event-ledger-audited-centered-compressed-anchor-cyclic-brief.md)

## Decision

Generalize the admitted private event-ledger Cyclic renderer to every exact
integer target satisfying:

```text
2L <= T <= 8L
```

`L` is source frames and `T` is target frames. The candidate identity is:

```text
signal-creative-continuous-event-ledger-cyclic-v1
```

The candidate keeps one complete renderer:

- one exact rational, sample-centred source/output map
- one fixed render-wide cycle clock
- two neighbouring forward native-rate reads
- one complementary half-cosine crossfade
- one independently computed commanded-event ledger
- one schedule shared by linked channels
- exterior zero and direct exact-length evaluation
- duration-independent deterministic offline state

It is not a router, blend, new grain engine, adaptive cycle, similarity
search, post-process, or second timeline. Interior targets change only the
exact target in the admitted equations.

## Immutable Parent And Allowed Change

The acoustic parent is Batch 34.1 commit
`4227390e48886075bd6d95103f6995508de2bb32`, tree
`c84dda048ea0e65b74882acdb14f3fee5126eebf`.

Frozen hashes are:

| File | SHA-256 |
| --- | --- |
| `creative_cyclic/mod.rs` | `620e0c35e5ab60125dc1e581e6db8c9d1f4e654f1e29fb84a21d5d30e62bb7a2` |
| `creative_cyclic/plan.rs` | `24d48c05c7c572a49faa58a83a180e5d9c84d0cd511b14ba674bfce85a004abd` |
| `creative_cyclic/schedule.rs` | `f9f9f7bd4c56a0c6827abd944365c174c14399dbea1a6816886078c41db6b865` |
| `creative_cyclic/interpolate.rs` | `129ee37ae5aacd7ba0ce1665e807b717854b9dfadd73124185f50b6cd8961d87` |
| `creative_cyclic/synthesis.rs` | `3d059381d87e7e9945fa25e32c4e4fa2d1e94872e887394e746747e5a38caddd` |
| `creative_cyclic/tests.rs` | `0cd5365070072fb0e96d52fcbef931d0b61f8bcaa087e3e0662e8773b76a307b` |
| `creative.rs` | `59b337ab547b66dafeec556e6b64c516a17cbeaf9bdbd9a430a8fb836fd8aaa5` |
| `lib.rs` | `5675f834150dd7918f8e73b48fbf888d7fe6d4143b3d29ae3531c008a95913ed` |
| crate `Cargo.toml` | `49b5e86d0a79530d0a6b1f2bb7cb17b8840cfb2cf0eb371ba7b686640807e3cd` |
| workspace `Cargo.lock` | `e3848a40d2ea1ff88a0e036df40d1fefa56c7aca950a95262c1d8c5668fd394d` |

Before acoustic checkpoint, production edits are limited to:

1. `mod.rs`: add the private behavior identity and one private
   `render_continuous` entry.
2. `tests.rs`: add focused domain, anchor-parity, and interior regression
   ownership.

`plan.rs`, `schedule.rs`, `interpolate.rs`, and `synthesis.rs` remain
byte-identical. Public `creative.rs`, `lib.rs`, the crate manifest,
dependencies, `Cargo.lock`, existing binaries, and existing tests remain
byte-identical.

The private entry performs exactly:

1. call the existing `Plan::new`
2. return empty success when `input_frames=target_frames=0`
3. compute `minimum=input_frames.checked_mul(2)`
4. return `CandidateError::UnsupportedRatio` when
   `target_frames < minimum`
5. call existing `synthesis::render` with that already validated plan

No second plan is built. Existing validation keeps compression, target above
`8L`, malformed input, non-finite input, invalid cycle, overflow, and
allocation errors unchanged. The candidate entry allocates no output before
the complete domain check.

The candidate-only evidence binary may import the private module with:

```rust
#[path = "../creative_cyclic/mod.rs"]
mod creative_cyclic;
```

It may not copy or fork renderer source.

## Exact Request Domain

The request remains finite mono or interleaved linked stereo, channels `1` or
`2`, sample rate `8000..=192000`, exact `T`, and integer
`cycle_us=5000..=90000`.

Rules:

- `L=0,T=0` is empty success
- non-empty identity and every `L<T<2L` request are outside this behavior
- every integer `2L<=T<=8L` is accepted
- `T>8L`, compression, and malformed requests retain their existing typed
  errors
- values reject; nothing clamps or rounds
- target frames remain the sole duration owner

The complete target vocabulary is:

| Class | Targets |
| --- | --- |
| admitted anchors | `2L`, `4L`, `8L` |
| one-frame adjacency | `2L+1`, `4L-1`, `4L+1`, `8L-1` |
| fractional interiors | `5L/2`, `15L/2` |
| integer interiors | `3L`, `5L`, `6L` |
| invalid boundaries | `2L-1`, `8L+1`, `16L` |

Structural sources use even lengths wherever a fractional target is named.
Synthetic sources use `L=88200`; musical sources use `L=220500`.

Acoustic interior probes are exactly:

| Tag | Ratio | Synthetic `T` | Musical `T` |
| --- | ---: | ---: | ---: |
| `r5d2` | `5/2` | `220500` | `551250` |
| `r5d1` | `5/1` | `441000` | `1102500` |
| `r15d2` | `15/2` | `661500` | `1653750` |

## Complete Renderer

The admitted parent brief remains normative for every acoustic equation. This
section restates the complete target-dependent path.

Cycle frames use checked positive half-up conversion:

```text
H=floor((sample_rate*cycle_us+500000)/1000000)
```

At `44.1 kHz`, `5`, `48`, and `90 ms` give `H=221`, `2117`, and `3969`.
The same three values are minimum, neutral, and maximum evidence anchors.

The only ideal source map at output frame `y` is:

```text
ideal(y)=((2y+1)L-T)/(2T)
```

For launch `k`:

```text
anchor(k)=((2kH+1)L-T)/(2T)
anchor_gap=HL/T
planned_replica_spacing=H(T-L)/T
```

All numerators use checked signed `i128`; the positive denominator is `2T`.
For output `y`, only launches `floor(y/H)` and `floor(y/H)+1` contribute.
Each launch reads forward at native rate. Linear interpolation uses Euclidean
division; source indices outside `[0,L)` are exact positive zero.

For remainder `r=y mod H`:

```text
c[r]=0.5-0.5*cos(pi*r/H)
w_right=c[r]
w_left=1-c[r]
```

The table has `H+1` `f64` entries with exact endpoints `0` and `1`.
Evaluation order stays output-major, channel-minor. Input converts once from
`f32` to `f64`; interpolation and crossfade use `f64`; output casts once to
`f32`.

There is no event detector, material classifier, tonal peak, propagated
phase, dormant state, reactivation state, random state, adaptive anchor,
feedback, normalization, limiter, implicit fade, tail repair, wrap, reflect,
or edge clamp. Transient placement and replica prevention are owned by the
single map, the two-launch schedule, and the commanded-event ledger.

Both linked channels share validation, target, cycle, anchors, rational
positions, fractions, weights, ledger, traversal, and boundary crop. Only
native channel samples differ.

Evaluate exactly `[0,T)`. Working state is the `H+1` table, plan, rational
positions, and scalar accumulators. It remains at most `256 KiB`, excluding
borrowed input and returned output, and independent of `L` and `T`.
Deterministic cost is `O(H+C*T)`. Execution is single-threaded and
offline-only.

## Candidate Isolation

Batch 34.3 starts from the Batch 34.2 closeout commit and creates:

- worktree: `/Users/tom/Dev/projects/signal-candidate-34-3`
- branch: `candidate/g10-034-continuous-event-ledger-cyclic`
- candidate evidence binary:
  `crates/signal-dsp-stretch/src/bin/continuous-event-ledger-cyclic-evidence.rs`
- candidate evidence modules:
  `crates/signal-dsp-stretch/src/bin/continuous-event-ledger-cyclic-evidence/`
- tracked authority:
  `candidate-evidence/g10-034/34-3/`
- ignored root:
  `target/creative-stretch-continuous-event-ledger-cyclic-34-3/`
- acoustic ref:
  `refs/signal-evidence/creative/continuous-event-ledger-cyclic/34-3-acoustic`

If the path, branch, or ref already exists, stop and reconcile it. Do not
overwrite or reuse unknown state.

Tracked authority contains:

- `row-manifest.tsv`
- `comparator.tsv`
- `listening-manifest.tsv`
- `run-continuous-event-ledger-cyclic.py`
- `generate-continuous-event-ledger-cyclic-sources.py`
- `generate-continuous-event-ledger-cyclic-reaper.py`

The evidence module directory contains exactly `manifest.rs`, `receipt.rs`,
`sources.rs`, `oracle.rs`, `metrics.rs`, `audio.rs`, `rows.rs`, and
`summary.rs`. The row-manifest header is:

```text
schema	ordinal	row_id	phase	owner	source_id	ratio_num	ratio_den	target_frames	cycle_us	channels	planned_render_count	comparator_row	assertion_ids	diagnostic_ids	artifact_policy
```

Its schema is `signal.continuous-event-ledger-cyclic.manifest.v1`.

Existing dependencies only. Candidate audio, comparator audio, receipts,
decisions, summaries, and build output stay ignored.

The evidence binary has only `construction`, `row`, and `summary` commands.
Each manifest row runs in its own OS process. The runner uses manifest order,
one thread, no retry, immediate failure output, and a `600 s` per-process
timeout. It stops after the first terminal failure or incomplete receipt.

Compile through:

```text
SIGNAL_CONTINUOUS_CYCLIC_BUILD_ROOT=<absolute-build-root> \
effigy build --release --bin continuous-event-ledger-cyclic-evidence
```

The runner invokes only the resulting absolute executable path. It requires a
clean worktree, exact commit/tree, exact manifest and source hashes, and an
absent execution root before each phase.

## Receipt And Construction Authority

Use UTF-8 JSONL, one trailing LF, no insignificant whitespace, ordered keys,
lowercase SHA-256, locale-independent decimal strings, and `create_new`.
Every row persists and `sync_all`s `started` before source load or rendering,
then persists and `sync_all`s exactly one `pass`, `fail`, or `panic`.

Schema is:

```text
signal.continuous-event-ledger-cyclic.row.v1
```

Identity is `signal-creative-continuous-event-ledger-cyclic-v1`. Ordered
fields are:

```text
schema,identity,checkpoint,tree,phase,row_id,sequence,status,
source_id,ratio_num,ratio_den,target_frames,cycle_us,channels,
planned_render_count,completed_render_count,input_container_sha256,
input_pcm_sha256,comparator_container_sha256,comparator_pcm_sha256,
output_pcm_sha256,artifact_sha256,assertions,diagnostics,error
```

Assertions retain ordered `id,status,expected,actual`; diagnostics retain
ordered `id,value,unit`. A pass requires every hard assertion to pass and
every mandatory diagnostic to exist and be finite. A crash, kill, timeout,
missing terminal line, duplicate row file, or summary mismatch is evidence
failure, not acoustic evidence.

Construction is one non-DSP row. It proves:

- exact file, module, dependency, production-diff, manifest, and command
  inventories
- all `577` row IDs, row counts, render counts, assertion IDs, diagnostic IDs,
  sources, comparators, listener rows, and gate order
- every owner is compile-linked to its real dispatcher
- exact formulas, rational known answers, error variants, units, thresholds,
  and aggregation laws
- source/WAV and REAPER project generators
- receipt failure, panic, kill, duplicate-open, escaping, and `sync_all`
  behavior
- independent map, ledger, FFT, cadence, gap, stereo, level, fade,
  concealment, decision, and summary known answers
- worst-case row completion and receipt persistence inside `600 s`

Sentinel construction uses the parent brief's exact fail, panic, and kill
protocol. It never invokes candidate DSP.

## Conformance Matrix

One manifest freezes:

| ID | Owner | Rows | Candidate renders | Other renders |
| --- | --- | ---: | ---: | ---: |
| C00 | construction | 1 | 0 | 0 |
| S01 | request domain | 32 | 0 | 0 |
| S02 | exhaustive small domain | 64 | 0 | 0 |
| S03 | rational map and schedule | 12 | 0 | 0 |
| S04 | immutable surface | 17 | 0 | 0 |
| S05 | boundary and exact crop | 36 | 36 | 0 |
| S06 | cycle, window, interpolation | 52 | 0 | 0 |
| S07 | determinism, memory, silence, stereo | 102 | 129 | 0 |
| P00 | admitted anchor byte parity | 18 | 18 | 18 public references |
| Y01..Y06 | interior synthetic | 183 | 201 | comparator reuse |
| M01..M02 | interior long-form | 60 | 60 | comparator reuse |

The manifest contains exactly `577` rows. Conformance is C00, S01..S07, and
P00: `334` rows, `183` candidate renders, and `18` public reference renders.
Acoustic evidence is `243` rows and `261` candidate renders.

S01 uses these exact cases in order:

```text
empty-success,empty-nonzero,nonempty-zero,identity,below-r2,r2,
r2-plus-one,r5d2,r3,r4-minus-one,r4,r4-plus-one,r5,r6,r15d2,
r8-minus-one,r8,over-r8,r16,channels-0,channels-3,partial-stereo,
rate-7999,rate-192001,cycle-4999,cycle-90001,nan,
positive-infinity,negative-infinity,direct-l-over-2p53,
direct-t-over-2p53,direct-allocation-overflow
```

Non-limit cases use `L=96000`. The target names map exactly to the target
vocabulary above. It requires the existing validation order and zero candidate
output allocation on every rejection.

S02 has one row per `L=1..64`. Each row checks every integer target
`0..=8L+1` against the closed-domain oracle. It also checks `16L`.

S03 uses `F=48000`, `L=96000`, `H=2304`, and these twelve accepted targets in
order:

```text
2L,2L+1,5L/2,3L,4L-1,4L,4L+1,5L,6L,15L/2,8L-1,8L
```

Every row verifies the ideal map at first, pre-cycle, cycle, midpoint, and
last output; anchors from zero through the final contributing successor;
strict complete monotonicity; exact rational positions; two contributing
launches; and positive, strictly ordered planned spacing.

S04 has exact rows `hash-mod`, `hash-plan`, `hash-schedule`,
`hash-interpolate`, `hash-synthesis`, `hash-tests`, `hash-creative`,
`hash-lib`, `hash-cargo-toml`, `hash-cargo-lock`, `allowed-mod-diff`,
`allowed-tests-diff`, `candidate-file-inventory`, `dependency-inventory`,
`absence-public-feature-report`, `absence-runtime-consumers`, and
`absence-forbidden-dsp`. Together they restrict the production diff to the
private entry, identity, and focused tests and prove no public, dependency,
feature, report, cache, artifact, runtime, audio-thread, Loophole, Chorus,
router, FFT, detector, adaptive schedule, random state, limiter, post-gain, or
external production dependency entered the tree.

S05 crosses source lengths `1,H-1,H,H+1,2H,2H+1` with target policies
`2L`, `2L+1`, `ceil(5L/2)`, `5L`, `floor(15L/2)`, and `8L-1`.
Every render checks exact target, finite output, direct crop, exterior zero,
and absence of unowned resize or fill.

S06 contains:

- rates `8000,44100,48000,192000` × cycles `5000,48000,90000`: `12`
- window points `0,H/4,H/2,3H/4,H` at the three `48 kHz` cycle anchors: `15`
- complement points `0,1,H/4,H/2,H-1` at the same anchors: `15`
- the parent brief's ten interpolation/exterior known answers: `10`

S07 contains:

- byte repeat, mono/stereo × three interiors × three cycles: `18` rows,
  `36` renders
- exact silence, mono/stereo × three interiors × three cycles: `18` rows and
  renders
- duplicate, anti-phase, common-negation, swap, right-gain, and
  right-delay-13 × three interiors × three cycles: `54` rows, `72` renders
- capacity at minimum/maximum rate and cycle in mono/stereo: `8` rows
- working-byte formula, maximum bound, duration independence, and exact output
  capacity: `4` rows, `3` renders

Samplewise duplicate, anti-phase, common-negation, and swap tolerance is
`1e-6`. Right gain is exact `0x3f004dce` and may move whole or three-band
ratio by at most `0.01 dB`. Delay retains strongest interior
cross-correlation lag in `12..=14`; correlation may not fall more than `0.02`
below ReaReaRea.

P00 uses `F=48000`, `L=96000`, one admitted mono fixture and one admitted
mixed-stereo fixture, exact ratios `2,4,8`, and cycles `5000,48000,90000`.
Each private continuous output must be byte-identical to the current public
Cyclic output. The row records their one common PCM hash. Any anchor miss
rejects before interior acoustic evidence.

## Conformance And Acoustic Checkpoint

From one clean candidate commit:

1. compile the complete evidence binary
2. run C00, S01..S07, and P00 in manifest order
3. write and verify summaries for every owner
4. repeat steps 1-3 from fresh receipt roots without changing a byte
5. require identical normalized receipts and output hashes
6. record commit, tree, all source/evidence/manifest hashes, toolchain,
   platform, REAPER identity, and comparator manifest
7. create the acoustic ref directly at that clean commit

No Y01..Y06, M01/M02, or inspectable listening render runs before the ref.
Conformance renders persist only hashes and receipts. Corrections before
checkpoint may only make code conform to an answer already fixed here. A
missing formula, source, metric, threshold, comparator, assertion, or listener
policy stops for docs-level correction.

After checkpoint, renderer bytes are immutable. Evidence plumbing follows
Contract `085` Rule 11: repairable evidence defects do not become renderer
failures or excuses to abandon the product target.

## Interior Synthetic Authority

Reuse the admitted parent brief's exact `44.1 kHz`, `88200`-frame source
formulas, support, ramps, sample hashes, numeric primitives, event ledger,
dropout map, FFT pitch estimator, cadence/autocorrelation estimator, gap/tail
owner, linked-energy owner, assertion IDs, units, and hard bounds.

The ten mono sources remain:

```text
low-tone,high-tone,chord,harmonic-pad,impulse,impulse-train,
uniform-noise,rademacher-noise,am-noise,silence-gap
```

The six stereo fixtures remain:

```text
duplicate,anti-phase,common-negation,swap,right-gain,right-delay13
```

Only the ratio set changes from `2,4,8` to `5/2,5/1,15/2`. Row IDs use
`r5d2`, `r5d1`, and `r15d2`; receipt fields use the exact numerator and
denominator.

Exact expansion is:

- Y01: ten mono sources × three ratios at `48000 us` = `30`
- Y02: four tonal sources × three ratios × three cycles = `36`
- Y03: two event sources × three ratios × three cycles = `18`
- Y04: three noise sources × three ratios × three cycles = `27`
- Y05: silence-gap and harmonic-pad × three ratios × three cycles = `18`
- Y06: six stereo fixtures × three ratios × three cycles = `54`

Totals are `183` rows and `201` candidate renders. Common-negation and swap
render both transformations; other rows render once.

Hard assertions retain:

- exact `T`, finite output, deterministic receipt, and raw peak no greater
  than input peak plus `2e-6`
- exact silence where the schedule reads only exterior zero
- comparator-relative exterior first difference within `+1.50 dB`
- continuous-source dropout ownership
- sparse-event ledger outside-zero, presence, order, and centre ownership
- silence-gap measurement and harmonic-pad no-false-gap
- linked samplewise, gain, delay, whole-balance, and three-band controls
- complete finite diagnostics

Pitch, cadence, replica distribution, event extent/energy, tail, local image,
and candidate-minus-comparator values remain diagnostic unless a hard
integrity rule above owns them. No diagnostic promotes output. Missing or
non-finite mandatory evidence rejects the receipt.

Y04 summary requires planned and measured cycle direction to be ordered short,
neutral, long for every source and interior ratio. The short cycle may exceed
the frozen cadence FFT search range; that frequency remains diagnostic, while
the exact planned-spacing order is hard.

## Comparator Preparation

Use REAPER `7.69/macOS-arm64`, ReaReaRea mode `983040`, field `0.0025`,
`44.1 kHz`, preserve pitch, zero fade-in, `10 ms` fade-out, exact item and
render length `T`, and stereo `24-bit` WAV.

The `63` comparator rows are:

- five retained native-stereo musical sources × three interiors: `15`
- ten mono synthetic sources × three interiors: `30`
- six stereo fixtures × three interiors: `18`

The musical source containers remain:

| Family/file | SHA-256 |
| --- | --- |
| percussion `0000-drums_percussion-000002.wav` | `89e55b28c6ed36e26bf73f2024d301aaeedd07cca30b315f5530051c28f4e1e7` |
| bass `0004-bass-000236.wav` | `3d587007a5d9a683e82e14a184530a9a0f953e58fbc2fe3712a42aa86ecf9ad8` |
| vocals `0008-vocals-000010.wav` | `3d74a686dccf6dcdfedc57e0fc2b76a0d29374ba28afa1bb497172cc441f7ee9` |
| pads `0012-pads_sustains-000423.wav` | `a736a0e04ade9e879db954069c8c0b68842bd4d364eec69e62dfef1447763131` |
| full mix `0016-full_mix-000144.wav` | `caa5d0d7c51bc7e2d537d3d13dbe32055f0c2032c69bd8e3f28a38df96fafbf1` |

For ratio `p/q`, item play rate is the exact rational `q/p`. The normalized
project projection records `ratio_num`, `ratio_den`, `T`, source identity,
play-rate numerator/denominator, mode, fades, render bounds, channels, and
absolute paths. The generator reparses every project before REAPER runs.

Invoke only:

```text
/Applications/REAPER.app/Contents/MacOS/REAPER \
  -newinst -nosplash -renderproject <absolute-project-path>
```

Every source, project semantics, project container, output container, decoded
PCM, frame count, channel count, sample rate, and REAPER identity is hashed.
Missing REAPER or a preparation mismatch blocks checkpoint creation; it does
not reject the renderer.

## Long-Form And Listening Authority

Use the same five `220500`-frame native-stereo musical sources.

Candidate renders are:

- M01 mono: five sources × three interiors × cycles
  `48000,5000,90000` = `45`
- M02 native stereo: five sources × three interiors at `48000 us` = `15`

Objective WAVs are deterministic IEEE `f32`, native channel count, exact
`T`, and never normalized, faded, or overwritten.

Listening copies use the admitted policy:

1. decode source, candidate, and comparator to `f32`
2. mono uses `f32((f64(left)+f64(right))/2)`
3. compute whole-file `f64` RMS
4. retain source gain `1`; RMS-match candidate and comparator to source
5. apply one common safety factor to keep the largest peak at or below `0.95`
6. apply the comparator's `441`-sample terminal linear fade after matching

No entry fade exists. Receipts bind raw and matched PCM/WAV hashes, gains,
RMS, peaks, safety factor, fade, and final artifacts. This avoids mistaking
file-generation fades for renderer energy distribution.

Conceal A/B identity with:

```text
SHA256(checkpoint || "|" || row_id ||
       "|continuous-event-ledger-cyclic-conceal-v1")
```

Listening rows are:

- L01: neutral mono A/B, high/mid/low ratio order, then source = `15`
- L02: short/neutral/long mono trios, source then ratio = `15`
- L03: neutral stereo A/B, high/mid/low ratio order, then source = `15`
- L04: six high-interior neutral stereo fixture A/B rows = `6`
- L05: one operator speaker pre-screen decision

Decision records are append-only and locked before reveal. They contain
recognizable Cyclic, musical usefulness, preference, usability, metallic
direction, echo direction, and the admitted mono/stereo artifact flags.

Neutral mono passes only when:

- recognizable Cyclic on at least `12/15`
- musically useful on at least `10/15`
- candidate preferred or tied on at least `10/15`
- candidate preferred or tied on at least `7/10` middle/high rows
- no unusable row and no family loses all three ratios
- no uncontrolled click, dropout, level step, buzz, lost event, doubled
  attack, or attack outside the commanded grammar

Direction passes only when short moves toward metallic/ring-like motion on at
least `12/15`, long moves toward tremolo/echo-like motion on at least `12/15`,
and each endpoint is useful on at least `10/15`. Inversion or an unusable
endpoint rejects.

## Linked-Stereo Admission

All Y06 and M02 hard controls complete before stereo listening. Long-form
whole and three-band candidate-source balance error may not exceed matching
ReaReaRea-source error by more than `0.50 dB`. Four-second mapped windows with
two-second hops record complete source, candidate, comparator, and delta
values. Missing or non-finite evidence rejects.

The operator performs a speaker pre-screen and may reject. Default promotion
then requires one eligible independent listener over L03 and L04.

Stereo passes only when:

- every samplewise, delay, gain, whole-balance, and band-balance hard control
  passes
- no unusable row
- candidate is preferred or tied on at least `10/15` musical rows
- no family loses every interior ratio
- no centre pull, one-sided texture, width pumping, side inversion, detached
  echo, channel-local cycle, or arbitrary balance movement

The Batch 32.25 waiver does not transfer. After every hard control passes, the
operator may make a new Contract `085` Rule 5 decision naming this exact
checkpoint. Without eligible review or that explicit checkpoint-scoped
decision, promotion remains blocked.

## Gate Order

Every stage uses the immutable acoustic ref:

1. Y01 through Y06 in manifest order
2. concealed neutral mono L01
3. short/neutral/long direction L02
4. M02 hard linked-stereo objectives
5. operator speaker pre-screen L05
6. eligible independent L03/L04 or explicit checkpoint-scoped Rule 5 decision
7. private promotion decision

Stop on the first valid terminal hard or listening failure. Finite diagnostics
alone do not reject. Listening remains creative promotion authority.

## Failure, Completion, And Cleanup

Evidence plumbing is not renderer quality. Missing assets, false assertions,
wrong paths, incomplete summaries, timeout plumbing, concealment defects, and
receipt defects are repaired under Contract `085` Rule 11, audited, and rerun
from a fresh root without changing acoustic bytes or policy. They do not close
the candidate or product goal.

A valid hard or listening failure rejects this acoustic checkpoint. Record the
row, dominant cause, last complete gate, checkpoint commit/tree/ref, receipt,
source, comparator, output, and artifact hashes. Do not scalar-sweep or tune
from the failed output.

After docs-level attribution, delete:

- candidate worktree and branch
- evidence binary and modules
- tracked candidate authority
- build state, receipts, comparator copies, renders, listening pack, and
  decisions
- acoustic ref when attribution no longer needs it

Nothing from a rejected checkpoint enters `main`. A rejection does not by
itself close continuous Cyclic. Continue with complete-system attribution.
Two complete candidates failing for the same dominant acoustic cause require
architectural reassessment, not a parameter sweep.

Only a complete pass may admit:

- the private `render_continuous` entry and behavior identity
- the `mod.rs` change frozen above
- focused continuous domain, anchor, deterministic, mono, and linked-stereo
  regression tests in `tests.rs`

`plan.rs`, `schedule.rs`, `interpolate.rs`, and `synthesis.rs` remain
byte-identical. Candidate runners, manifests, profiles, fixtures, comparator
tools, receipts, audio, decisions, and refs stay out of `main`.

Public Cyclic widening, discovery, public engine identity, cache, artifacts,
router, dynamic ratio, runtime, UI, Loophole, and Chorus remain separate
batches.

## Readiness

The candidate has one transform, one domain, one map, one schedule, one
transient/event owner, one tonal policy, one stereo owner, one boundary law,
one memory/cost bound, exact source and comparator families, fixed metrics and
thresholds, one listening policy, explicit evidence repair, rejection,
cleanup, and minimal admission.

Batch 34.3 is ready. It may implement and execute only this isolated candidate.

## Next Task

Execute `g10.034` Batch 34.3 only. Create the isolated worktree, implement the
frozen private entry and complete evidence system, pass two conformance rounds,
freeze one acoustic checkpoint, then run the gates in order. Do not widen the
public API or start routing, cache, artifact, runtime, UI, Loophole, or Chorus
work.
