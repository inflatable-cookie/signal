# Offline Creative ContinuousDirectRenewalDream Brief

Status: admitted privately
Owner: dsp
Updated: 2026-07-24
Contract: `085`
Roadmap: `g10.033`, Batch 33.3

## Decision

Generalize the admitted private `DirectRenewalDream` renderer to every exact
integer target satisfying:

`4L <= T <= 16L`

where `L` is source frames and `T` is target frames.

The candidate name is `ContinuousDirectRenewalDream`. It is the same acoustic
renderer with a wider private target gate. It is not a blend, router, new
transform, or replacement character.

The complete candidate keeps:

- one long-window native-channel analysis
- one exact sample-centred monotonic source map
- one target-driven frame schedule
- independent deterministic renewal by output frame and positive bin
- one shared linked-stereo rotation plus symmetric `space` decorrelation
- one compensated adjacent-frame blend
- the admitted short entry guard and longer terminal release
- exact target crop and bounded deterministic offline state

There is no phase propagation, peak tracking, transient detector, material
separator, source-alignment search, recurrence, limiter, post-render gain,
ratio crossfade, or second renderer.

## Immutable Parent

The acoustic parent is the `main` tree at Batch 33.1 commit
`cfe1ab92dba08af93215ca47caca0946ed502e67`, tree
`5f776745c9181d95152c6889c67c66757a51465d`.

Frozen source hashes are:

| File | SHA-256 |
| --- | --- |
| `plan.rs` | `6fbfa85252208c806bb0e28ccf502f100e3a827f8bc718dda77c5f4dfb12ce6a` |
| `analysis.rs` | `71a1ae1815078fd5916ac28547a961ce4ed6fadbd978435de9f18033945713ac` |
| `stereo.rs` | `e159cdab1b674dd97e00658d62902719b4100e26744db57da914e2eb20c94ee3` |
| `synthesis.rs` | `ca0e267617cfb49634f79d630bbded7c5570e5db2e7a13d3014c346afefdfe54` |
| `mod.rs` | `36316c8dace47c4963d7d3df7874b3e6e2353d6e39a7c730aca80b92f30fadbb` |
| `tests.rs` | `4618bd2ff585ece3c80cb1f93000338326eec3f98aaef4a2b978bd082f2868e8` |
| `regression_manifest.tsv` | `406796fca9b6a44bcbc55ebdffd5bcee1061cf2f389bd2975f7fbf3a4b893705` |

The only acoustic production edit permitted before checkpoint is the
`validate_dimensions` supported-target predicate in `plan.rs`.
`analysis.rs`, `stereo.rs`, and `synthesis.rs` must remain byte-identical.
Every other `plan.rs` line must remain semantically and textually identical.

Candidate-only evidence may:

- add a `#[cfg(test)]` continuous test module declaration
- add that test module and a tracked evidence ledger/profile
- replace the existing structural `unsupported-5x` row with
  `unsupported-3x`, preserving its row and render count
- update the matching regression-manifest row ID

No public request, dispatch, engine version, cache identity, report mode,
binary, feature, dependency, artifact, runtime, Loophole, or Chorus surface
may change in the candidate.

## Exact Request Domain

The private request remains:

- finite mono or interleaved stereo `input: &[f32]`
- `channels` equal to `1` or `2`
- sample rate `8000..=192000`
- exact integer `target_frames`
- explicit `seed: u64`
- finite `space` in `[0,1]`

Validate with checked integer arithmetic only:

1. Reject `L` or `T` above `2^53-1`.
2. Accept only `L=0,T=0` as empty success.
3. Reject non-empty `L` with `T=0`.
4. Compute `minimum=checked_mul(L,4)` and
   `maximum=checked_mul(L,16)`.
5. Accept exactly when `minimum <= T <= maximum`.

Every integer target in that closed interval is supported. The ratio need not
be integral, rationally named, a power of two, hop-divisible, or
frame-adjacent to an admitted anchor. Rejection happens before output
allocation. Values are rejected, never clamped.

The four interior acoustic probes are exact rationals:

| Name | Target |
| --- | --- |
| `9/2` | `T=9L/2` |
| `6` | `T=6L` |
| `10` | `T=10L` |
| `31/2` | `T=31L/2` |

All frozen acoustic source lengths are even, so these targets are exact
integers. The endpoints and near-endpoint structural probes are `4L`,
`4L+1`, `16L-1`, and `16L`.

## Frozen Renderer

The admitted
[DirectRenewalDream brief](./offline-creative-direct-renewal-dream-brief.md)
remains normative for every acoustic equation. The following compact
restatement is the complete candidate.

For sample rate `F`:

- `N=clamp(nearest_power_of_two(round_half_up(2F/3)),8192,131072)`
- power-of-two distance ties select the larger value
- `H=N/2`
- `w[n]=0.5-0.5*cos(2*pi*n/N)`
- `G=sqrt(N/sum(w[n]^2))`

Output block `j` starts at `y_j=jH`; `B=ceil(T/H)`. Analysis/inverse frames
are `j=0..B`; output blocks are `j=0..B-1`.

The only source centre is:

`x_j=((y_j+0.5)L/T)-0.5`

Evaluate `(2y_j+1)L/(2T)` with checked `u128`, convert once to `f64`, then
subtract `0.5`. One `FrameSchedule` computes the strictly increasing sequence
once for all linked channels.

For `p=x_j+n-(N-1)/2`, use the admitted four-point cubic Lagrange read over
`floor(p)-1..floor(p)+2`; source exterior is exact positive zero. Multiply by
the periodic Hann, cast once to `f32`, and transform as `Complex32`.

Renewal keeps the admitted counter-addressed `mix64` law, stream tags,
high-53 phase conversion, and
`ADMISSION_SEED=0x0123456789abcdef`. There is no mutable RNG. For mono,
positive non-Nyquist bins use one independent `BASE` rotation. For stereo,
both native coefficients use the same `BASE` rotation plus symmetric `SPACE`
offset:

- frequency weight `h=0` through `250 Hz`
- smoothstep transition from `250` to `1500 Hz`
- `h=1` from `1500 Hz`
- `d=0.5*space*h*zeta`
- left rotation `theta-d`
- right rotation `theta+d`

DC and Nyquist retain their real native coefficients. Negative bins are
conjugate mirrors. Linked channels share map, frame identity, and random
address field.

Synthesis keeps two adjacent inverse frames per channel. For `0<=n<H`:

- `u=(n+0.5)/H`
- `a=0.5+0.5*cos(pi*u)`
- `b=1-a`
- `c=1/sqrt(a^2+b^2)`
- `q[j,n]=G*c*(a*z[j,H+n]+b*z[j+1,n])`

There is no synthesis window, OLA denominator, adaptive gain, limiter,
compressor, or normalization pass.

Boundary ownership remains:

- `E_head=min(ceil(F/200),floor(T/4))`
- `E_tail=min(2H,floor(T/4))`
- sine head and tail factors from the admitted brief
- exact-zero first and last sample
- exact emission of `[0,T)`

No transient state exists. The long magnitude view deliberately smears
attacks. Audible replicas, micro-echo, stutter, static freeze, clicks, arbitrary
loudness, or a new entry/tail character reject the candidate.

## Material, Event, And Tonal Ownership

One long-window spectrum is the unified material representation. No
material-dependent resolution exists or is added. Every source event enters
only through the monotonic `x_j` schedule and cubic source read; there is no
second event clock, transient copy, reset, reassignment, search, loop, or
grain trigger.

There is no tonal peak, propagated phase, dormant-bin, or reactivation state.
Every active frame/bin is renewed from its current native coefficient and
counter address. Replica prevention therefore comes from one map, one visit
per output block, no source replay mechanism, and the hard impulse/periodicity
plus listening gates. An event-placement or replica miss rejects the complete
candidate; it does not authorize a local transient mechanism.

## State, Determinism, And Cost

Allocation and traversal remain unchanged:

- output and all work buffers allocate before frame zero
- exactly two adjacent inverse frames are retained per channel
- actual peak working state is at most `32 MiB` for stereo at
  `N<=131072`, excluding borrowed input and returned output capacity
- working memory is independent of duration
- no allocation or reallocation occurs during processing
- mono performs one forward and inverse FFT per new frame
- stereo performs two of each
- cost is `O((T/H)N log N)`
- traversal is single-threaded with fixed reduction order
- identical requests remain byte-identical on the frozen platform/toolchain

The renderer stays offline-only. No audio-thread execution, synchronization,
I/O, source fill, or allocation is authorized.

## Candidate Isolation

Batch 33.3 starts only after the Batch 33.2 closeout commit. It creates:

- worktree: `/Users/tom/Dev/projects/signal-candidate-33-3`
- branch: `candidate/g10-033-continuous-direct-renewal-dream`
- tracked ledger:
  `candidate-evidence/g10-033/33-3/conformance.tsv`
- tracked parent manifest:
  `candidate-evidence/g10-033/33-3/anchor-parity.tsv`
- candidate-only nextest profile: `continuous-dream`
- ignored evidence root:
  `target/creative-stretch-continuous-dream-33-3/`
- acoustic ref:
  `refs/signal-evidence/creative/continuous-direct-renewal-dream/33-3-acoustic`

The closeout commit, its tree, and all seven parent hashes above are recorded
before the worktree is created. If the expected worktree or branch already
exists, stop and reconcile it; never overwrite or reuse unknown state.

No historical rejected candidate, helper, output, threshold, or checkpoint may
be recovered. The admitted parent source and retained comparator assets are
the only reusable implementation/evidence inputs.

## Frozen Comparator Preparation

Comparator preparation happens before candidate acoustic checkpoint and never
executes candidate DSP.

Use the retained PaulXStretch `1.6.0` default, FFT `16384`, derived from
upstream commit `8ec191fdd7203354c79391cbc04c9fd83fa30ea0`.
The local reference identities are:

| Item | SHA-256 |
| --- | --- |
| mono executable | `88325cad1fbb6d3bbcdfede1f05d10f8c75c1340d1780a9300cd34551672cb8e` |
| stereo executable | `5bd69b41c15afba298d4b9113355def44613b7fc00326c3da18672491f567d7e` |
| mono CLI source | `3db79dcdeb129faa4e1561bc17c5f553c2d8b0907591ae964b078ef1d3605b4b` |
| stereo CLI source | `04b0865ddebf98f4a0d470de98972e2883bedbbf98867457764165b83e83ae0f` |
| pinned `Stretch.cpp` | `b0f994539d2b59f5f66f7486958792c18e6e83a282a18a702fbbb87b47ef89c5` |

The executable contract is little-endian interleaved `f32`:

`paulx-reference INPUT.raw OUTPUT.raw RATIO`

and:

`paulx-reference-stereo INPUT.raw OUTPUT.raw RATIO`

Use decimal ratios `4.5`, `6`, `10`, and `15.5`. Crop only trailing excess to
exact `T`; short output rejects comparator preparation. Record source,
executable, full raw output, and cropped output hashes in an immutable
comparator manifest under the ignored evidence root. Existing `4x`, `8x`,
and `16x` PaulX files remain the anchor references; they are not regenerated.

Before editing `plan.rs`, render the admitted Signal parent over the frozen
anchor-parity matrix below and record little-endian output SHA-256 values in a
tracked `anchor-parity.tsv`. The generator records the parent commit, tree,
seven source hashes, request fields, input hashes, and output hashes. It
records hashes only; no baseline audio enters git.

Anchor parity uses the admitted `F=48000`, `L=96000` source generator,
`ADMISSION_SEED`, and `space=0.5`:

- mono low tone, chord, impulse, and uniform noise at `4x`, `8x`, `16x`
- stereo duplicate, delayed pad, and mixed fixtures at each anchor and
  `space=0`, `0.5`, `1`

That is exactly `39` rows. Combined with the gate-only source diff, this proves
unchanged anchor behavior. Any baseline-generation mismatch against the
admitted parent stops the candidate before implementation.

## Evidence Authority

Candidate-only tests use exact prefixes:

- `continuous_direct_renewal_dream_construction_`
- `continuous_direct_renewal_dream_structural_`
- `continuous_direct_renewal_dream_acoustic_`

One compile-linked owner table contains exactly these owners:

| ID | Owner | Rows | Candidate renders | Boundary |
| --- | --- | ---: | ---: | --- |
| C01 | `authority_manifest` | 1 | 0 | hashes, counts, commands, masks, paths |
| S01 | `request_domain` | 16 | 0 | closed target interval and preallocation rejection |
| S02 | `small_domain_oracle` | 64 | 0 | every target around `4L..16L` for `L=1..64` |
| S03 | `map_schedule` | 9 | 0 | endpoints, interiors, near-endpoints, monotonicity |
| S04 | `immutable_surface` | 20 | 0 | source hashes, allowed diff, absent public/forbidden surfaces |
| S05 | `boundary_crop_matrix` | 36 | 36 | frame-adjacent lengths and non-hop targets |
| S06 | `determinism_memory_stereo` | 15 | 20 | repeat, seed, relation, allocation, bounded state |
| A00 | `anchor_byte_parity` | 39 | 39 | exact parent equality at `4x`, `8x`, `16x` |
| A01 | `integrity_crest_discontinuity` | 40 | 40 | ten mono sources at four interiors |
| A02 | `pitch_diagnostic` | 28 | 12 | two tones and five chord partials |
| A03 | `impulse_diagnostic` | 8 | 8 | impulse and train |
| A04 | `periodicity_modulation_gap` | 12 | 12 | noise, tone, silence gap |
| A05 | `linked_stereo_inventory` | 27 | 27 | exact interior relation/space matrix |

Continuous structural totals are `160` rows and `56` candidate renders.
Continuous acoustic totals are `154` rows and `138` candidate renders. The
existing admitted `1` construction, `10` structural, and `5` synthetic owners
remain compiled. Before checkpoint, execute only their construction and
structural prefixes; their acoustic owners must not run.

`S01` rows, in order, are empty success; empty/non-zero; non-empty/zero;
`4L-1`; `4L`; `4L+1`; `9L/2`; `6L`; `8L`; `10L`; `31L/2`; `16L-1`; `16L`;
`16L+1`; source above the exact-integer limit; target above the exact-integer
limit. Non-empty domain rows use `L=96000`; limit rows call the same direct
dimension oracle without materializing impossible slices. `S02` uses one
receipt row per `L=1..64` and internally checks every target from `4L-1`
through `16L+1` against the integer oracle.

`S03` uses `F=48000`, `L=96000`, and the nine accepted targets named in `S01`.
Each row verifies `B`, `x_0`, `x_1`, `x_floor(B/2)`, `x_(B-1)`, `x_B`, exact
`u128` arithmetic, and strict monotonicity.

`S05` crosses source lengths `1,H-1,H,H+1,2H,2H+1` with targets `4L+1`,
`ceil(9L/2)`, `6L`, `10L`, `floor(31L/2)`, and `16L-1`. Ceiling and floor use
checked integer division and equal the exact named ratios for even `L`. Every
render checks frame count, blend, envelopes, exact-zero endpoints, exact
length, finiteness, crop, and absence of unowned fill.

`S04` proves the four immutable acoustic files byte-identical, the parent
`plan.rs` diff restricted to the supported predicate, public `creative.rs`,
`lib.rs`, `Cargo.lock`, and dependencies unchanged, and the absence of phase
recurrence, peak/transient/material state, search, limiter, post-gain, router,
cache, artifact, runtime, Loophole, and Chorus edits.

Its exact `20` rows are the hashes of `analysis.rs`, `stereo.rs`,
`synthesis.rs`, and unchanged `plan.rs` regions; one allowed-diff row; hashes
for `creative.rs`, `lib.rs`, and `Cargo.lock`; one dependency-set row; and
absence rows for phase recurrence, peak/transient/material state, alignment
search, limiter/post-gain, public/router change, cache/artifact change,
runtime/audio-thread change, Loophole change, Chorus change, feature/report
change, and external production dependency.

`S06` has exact rows:

- byte-identical repeat at each interior ratio: `4` rows, `8` renders
- changed-seed inequality at `6x` and `10x`: `2` rows, `4` renders
- duplicate at `4.5x/space=0`: `1` row/render
- anti-phase at `10x/space=0`: `1` row/render
- common polarity and its negation at `15.5x/space=0.5`: `1` row,
  `2` renders
- mixed stereo and channel swap at `6x/space=0.5`: `1` row, `2` renders
- planned mono/stereo state at `8000` and `192000 Hz`: `4` rows, no render
- instrumented duration-independent allocation at `4.5x` and `15.5x`:
  `1` row, `2` renders

Total: `15` rows and `20` renders.

The tracked nextest profile is:

```toml
[profile.continuous-dream]
retries = 0
fail-fast = false
test-threads = 1
slow-timeout = { period = "300s", terminate-after = 2 }
failure-output = "immediate-final"
success-output = "never"
status-level = "all"
final-status-level = "all"
```

Receipts use the admitted DirectRenewalDream canonical JSON-line schema and
write each row with `flush` plus `sync_all`. Tests never rewrite the tracked
ledger. Every invocation gets a fresh ignored directory named by checkpoint,
stage, round, and owner. Audio stays ignored; receipts contain hashes.

## Conformance And Checkpoint

From one clean candidate commit, run:

1. `effigy test cargo-nextest -- -P continuous-dream -r --no-run -p signal-dsp-stretch`
2. `effigy test cargo-nextest -- -P continuous-dream -r -j 1 --no-fail-fast --failure-output immediate-final -E 'test(/^direct_renewal_dream_construction_/)'`; require `1/1`
3. `effigy test cargo-nextest -- -P continuous-dream -r -j 1 --no-fail-fast --failure-output immediate-final -E 'test(/^direct_renewal_dream_structural_/)'`; require `10/10`
4. `effigy test cargo-nextest -- -P continuous-dream -r -j 1 --no-fail-fast --failure-output immediate-final -E 'test(/^continuous_direct_renewal_dream_construction_/)'`; require `1/1`
5. `effigy test cargo-nextest -- -P continuous-dream -r -j 1 --no-fail-fast --failure-output immediate-final -E 'test(/^continuous_direct_renewal_dream_structural_/)'`; require `6/6`

Repeat all five unchanged from the same clean tree and require identical counts
and receipts.

Record candidate commit/tree, every candidate/evidence SHA-256, parent
identity, allowed diff, `Cargo.lock`, compiler, Effigy, nextest, OS,
architecture, owner tables, comparator manifest, anchor manifest, and ledger
hash. Create the acoustic ref directly at that commit.

Compiler, type, visibility, allocation, and exact-conformance repairs may
iterate before checkpoint only when this brief already fixes the answer. Any
new acoustic formula, scalar, seed, source, helper algorithm, metric,
threshold, assertion, comparator, or listening choice stops for docs-level
reassessment. No candidate acoustic owner or inspectable candidate audio runs
before the ref.

After checkpoint, run `A00..A05` individually in numeric order from the
immutable ref. Stop on the first failure. Evidence-plumbing defects follow
Contract `085` Rule 11; they never authorize an acoustic change.

## Synthetic Sources And Gates

Reuse the admitted brief's exact ten mono sources, five stereo fixtures,
`F=48000`, `L=96000`, support, ramps, formulas, and input hashes. Change only
the target list to `9L/2`, `6L`, `10L`, and `31L/2`.

`A05` renders:

- duplicate at all four ratios and `space=0,0.5,1`: `12`
- base and common-negated at `10x/0.5`: `2`
- mixed and channel-swapped mixed at `10x/0.5`: `2`
- anti-phase at `10x` and all spaces: `3`
- delayed pad at all four ratios and `space=0,1`: `8`

Total: `27` rows and renders.

Hard for every acoustic output:

- exact target length
- finite samples
- exact-zero first and last sample
- absolute sample no greater than `8`
- complete deterministic receipt and hashes
- no exact-zero run of `H` complete frames wholly inside mapped authored
  non-zero support

`A00` must match every frozen parent hash exactly.

Reuse the admitted definitions for first-difference crest, pitch, impulse
width/centroid/regions, periodicity, block-RMS variation, silence-gap RMS,
low-frequency energy, stereo residuals, and three-band balance. Interior
PaulX values are diagnostics except:

- uniform-noise autocorrelation and uniform/mid block-RMS CV must not exceed
  the matching PaulX row by more than `0.05`
- candidate-source whole and three-band stereo balance error must not exceed
  `0.75 dB`
- balance spread across complete `space` trios must not exceed `0.50 dB`
- dominance must not reverse where source balance magnitude is at least
  `0.50 dB`

Pitch error, crest delta, entry/tail energy, impulse regions, event placement,
low-frequency noise, and image metrics stay finite mandatory diagnostics.
Metrics diagnose or reject integrity; they cannot promote sound.

## Long-Form Mono Gate

Only after `A00..A05` pass, use the admitted five `44.1 kHz`,
`220500`-frame source files and hashes:

| Family/file | Original-file SHA-256 | Decoded mono SHA-256 |
| --- | --- | --- |
| percussion `0000-drums_percussion-000002.wav` | `89e55b28c6ed36e26bf73f2024d301aaeedd07cca30b315f5530051c28f4e1e7` | `da20fe2d616be4f2e589b46390cd85ecff3c323a870bd35d36d3432fbc0ad68f` |
| bass `0004-bass-000236.wav` | `3d587007a5d9a683e82e14a184530a9a0f953e58fbc2fe3712a42aa86ecf9ad8` | `7bf03e3ba0c3ab0c197c881a106767d46f26a1d6a11bf84449b2f500aa2f0687` |
| vocals `0008-vocals-000010.wav` | `3d74a686dccf6dcdfedc57e0fc2b76a0d29374ba28afa1bb497172cc441f7ee9` | `64c96ca0ba72778def43940d963e5467e0c62f48e429cead53d7f08841bfc417` |
| pads `0012-pads_sustains-000423.wav` | `a736a0e04ade9e879db954069c8c0b68842bd4d364eec69e62dfef1447763131` | `5ac7f7b3650b02f2ed160d682c56b0359d8972c8b8f77d671552b57e56da8a2e` |
| full mix `0016-full_mix-000144.wav` | `caa5d0d7c51bc7e2d537d3d13dbe32055f0c2032c69bd8e3f28a38df96fafbf1` | `637b60f3b0e0230f871d48b804507a1388ece6570abcfa58d4a74610ea84c96b` |

Render exact `4.5x`, `6x`, `10x`, and `15.5x` mono with
`ADMISSION_SEED`, `space=0.5`, and exact crop. Render matching PaulX
comparators with the frozen mono executable. This is `20` candidate and `20`
comparator rows.

Mono decoding is samplewise
`f32((f64(left)+f64(right))/2)`. Stereo review uses the original decoded
channels. Any source-file, decoded-mono, channel-count, sample-rate, or frame-
count mismatch rejects preparation.

Within each row, source, candidate, and comparator share one RMS target reduced
only to keep peak at or below `0.95`. Conceal identity. Review all five `6x`
rows, then all five `10x`, then `4.5x` and `15.5x`.

Pass requires:

- no unusable candidate row
- candidate preferred or tied on at least `16/20`
- no source family losing all four ratios

Reject exposed vocoder colour, rough periodicity, cyclic repetition, audible
replica, doubled attack, micro-echo, stutter, static freeze, arbitrary
loudness, click, or a material regression in the accepted smooth Dream
character.

The scorecard records smoothness, musical usefulness, tonal focus/ringing,
transient softness/spike, low-frequency noise, event placement, width, entry
energy over the first `5%`, and tail energy over the last `5%`. The operator is
mono promotion authority.

## Linked-Stereo Gate

Only after mono passes, render all five original stereo sources at all four
ratios and `space=0,0.5,1`: `60` candidate renders. Use `20` neutral PaulX
stereo comparators.

Hard controls reuse the admitted exact length, finiteness, deterministic,
whole/three-band balance, trio spread, dominance, correlation, width,
low-frequency, entry, and tail diagnostics.

The operator may reject during speaker pre-screen. Default promotion requires
an eligible independent listener. Concealed `space=0.5` review passes only
with:

- no unusable candidate row
- preferred or tied on at least `16/20`
- no source family losing all four ratios
- every `space` trio moving preserve-to-widen without image jump, unrelated
  channel motion, low-frequency pull, balance shift, or unusable setting

The Batch 31.66 waiver applied only to checkpoint `760da32d`; it does not
silently generalize. After all hard controls pass, the operator may make one
new explicit checkpoint-scoped Rule 5 decision. Without independent review or
that explicit decision, promotion remains blocked.

## Rejection, Cleanup, And Minimal Admission

Any hard objective, mono-listening, speaker, or required stereo-review miss
rejects the checkpoint. A finite diagnostic alone does not. Record the
dominant cause, stopped gate, completed rows, comparator identity, and exact
checkpoint.

Do not tune, repair, or rerun after acoustic identity. Delete the candidate
worktree, branch, build state, test/config surface, anchor manifest, comparator
copies, receipts, and rendered audio. Retain the local evidence ref through
one docs-only reassessment, then delete it when the evidence question closes.
Nothing from a rejected candidate enters `main`.

Only a complete pass may open a separate minimal-admission batch for:

- the private validation predicate accepting every `4L<=T<=16L`
- the admitted acoustic files unchanged
- focused continuous structural and synthetic regression owners
- internal renderer identity `signal-creative-direct-renewal-dream-v2`

Candidate evidence runners, nextest profile, listening assets, comparator
tools, receipts, and checkpoint ref stay out of `main`. Public ratio widening,
public engine identity, hidden same-character routing, cache, artifacts,
dynamic ratio, UI, runtime, Loophole, and Chorus remain separate decisions.

Two complete continuous candidates failing for the same dominant acoustic
cause close this direction for architectural reassessment. No ratio, seed,
window, phase, envelope, threshold, or assertion sweep follows failure.

## Admission

Checkpoint `0e9969ab68067102b46c18205ef064da4fdb71c9`, tree
`e5184e08d5bf792db42433e5fcd1bf0b11b17b68`, passed both complete conformance
rounds with identical normalized receipts. Acoustic admission passed
`154/154` rows and `138/138` candidate renders. Exact `4x`, `8x`, and `16x`
anchors remained byte-identical.

Concealed mono passed as `20/20` usable ties against PaulXStretch. All `60`
long-form stereo hard-control renders passed. The operator then accepted all
`20` neutral stereo comparisons and preserve-to-widen trios and explicitly
waived independent review for checkpoint `0e9969ab`.

Commit `73910aad` admits only the private target predicate, internal
`signal-creative-direct-renewal-dream-v2` identity, and focused continuous
regression owners. The admitted acoustic files remain unchanged. Candidate
evidence, comparators, receipts, audio, and the nextest profile did not enter
`main`.

## Next Task

Execute `g10.033` Batch 33.5 from the frozen
[public surface](./offline-creative-fixed-ratio-public-surface.md). Widen only
the public Dream target domain and discovery; do not change this private
renderer or add routing.
