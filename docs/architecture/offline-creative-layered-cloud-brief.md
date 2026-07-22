# Offline Creative LayeredCloud Renderer Brief

Status: closed for Batch 31.70; synthetic receipt invalid
Owner: dsp
Updated: 2026-07-22
Contract: `085`
Roadmap: `g10.031`, Batches 31.69-31.71

## Decision

Freeze one clean-room `LayeredCloud` renderer for fixed creative expansion from
`16x` through `100x`. It is a pointer-led granular synthesizer with one exact
map, one deterministic launch lattice, bounded variable-length unit-rate
grains, one channel-shared normalization law, and the admitted Dream exterior
envelope.

The architecture follows the complete family demonstrated by Csound
`sndwarpst`: bounded overlapping windows re-anchor to one time pointer, read
the source at unit rate, and share their scheduler across both source channels.
SuperCollider `Warp1` independently confirms the pointer-led granular family,
but its implementation keeps active grains and random launches per output
channel. It is not Signal's stereo authority.

This is not a stack of wet effects, a spectral layer over Dream, a recovered
cyclic candidate, or an external dependency. Csound and SuperCollider source
are clean-room architecture evidence only. Their code, constants, random
generators, tables, and control flow do not transfer.

## Supported Request

The private request contains exactly:

- finite mono or interleaved stereo `input: &[f32]`
- `channels` equal to `1` or `2`
- `sample_rate` from `8000` through `192000`
- exact `target_frames`
- explicit `seed: u64`

Let source frames be `L`, target frames be `T`, and sample rate be `F`. Empty
input with `T=0` returns empty. For non-empty input, require checked
`H=max(1,round_half_up(F/64))`, `L>=H`, `16L <= T <= 100L`, `L<=2^53-1`,
and `T<=2^53-1`. Partial frames, non-finite input, a non-empty source shorter
than `H`, zero target, range misses, size overflow, and allocation overflow
fail before output allocation. Values are rejected, never clamped.

Only fixed-ratio canonical `Cloud` exists. `motion`, `detail`, `space`, pitch,
reverse, dynamic ratio, other characters, routing, cache, artifacts, realtime
use, and public exposure are absent.

## Source Map And Launch Lattice

Use the same sample-centred map as admitted `DirectRenewalDream`:

`x(y)=((y+0.5)L/T)-0.5`.

Evaluate `(2y+1)L/(2T)` with checked `u128`, convert once to `f64`, then
subtract `0.5`.

Freeze the launch hop:

`H=max(1,round_half_up(F/64))`.

Launch centres are `0,H,2H,...` below `T`, plus `T-1`. Sort and deduplicate.
For launch `j` at output centre `y_j`, source centre is exactly `x_j=x(y_j)`.
The sequence is strictly increasing. There is no source-position jitter,
alignment search, accumulated correction, free-running cursor, or second
event timeline.

Every linked channel consumes the same ordered launch records. Grain length
changes texture; it never changes the source centre.

## Grain Geometry

Use wrapping `u64` `mix64`:

1. `z=(z xor (z>>30))*0xBF58476D1CE4E5B9`
2. `z=(z xor (z>>27))*0x94D049BB133111EB`
3. return `z xor (z>>31)`

Use the distinct tag `0x434c4f55445f4455`. For launch ordinal `j`:

`r_j=mix64(seed xor tag xor j)`.

Select the integer duration:

`D_j=12H+(r_j mod (8H+1))`.

Every integer duration from `12H` through `20H` is therefore addressable.

Freeze `ADMISSION_SEED=0x4c41594552434c44`.

Construction owns these exact vectors:

| Input or launch | `mix64` result | `D_j` at `H=750` |
| --- | --- | ---: |
| `0` | `0x0000000000000000` | — |
| `1` | `0x5692161d100b05e5` | — |
| `u64::MAX` | `0xb4d055fcf2cbbd7b` | — |
| `seed xor tag xor 0` | `0x21ff9c0ce71cf025` | `14565` |
| `seed xor tag xor 1` | `0x07696f531ec8a5a5` | `14048` |
| `seed xor tag xor 2` | `0xdd9ceeba647a4de2` | `9952` |
| `seed xor tag xor 7` | `0xa9ddfca3dcb6d346` | `14687` |
| `seed xor tag xor 11` | `0xbc8b4ab4f7f4abd6` | `12242` |

For output frame `y`, let signed offset `q=y-y_j`. A grain is active only when
`2*abs(q)<D_j`. Its continuous window is:

`w_j(y)=0.5+0.5*cos(2*pi*q/D_j)`.

Its source coordinate is:

`p_j(y)=x_j+q`.

Each grain therefore reads forward at unit rate. Adjacent grain centres move
slowly on the common map while their native-rate interiors overlap. This
creates the cloud character without pitch-shifting a grain, randomizing a
channel independently, or introducing a second renderer.

At most `22` grains may contribute to one output frame, including the explicit
terminal launch. Candidate code uses a fixed-capacity launch window. The proof
combines the strict support `D<=20H`, the `H`-spaced regular lattice, and at
most one non-lattice terminal launch; construction exhausts every supported
integer sample rate and all terminal-launch residues. `L` and ratio cannot
increase the output-lattice occupancy.

## Sampling And Normalization

For source coordinate `p`, let `i=floor(p)` and `u=p-i`. Define source samples
outside `[0,L)` as positive zero. Use linear interpolation:

`s_c(p)=(1-u)a_c[i]+u*a_c[i+1]`.

Interpolate the source-validity mask by the same law:

`m(p)=(1-u)I(0<=i<L)+uI(0<=i+1<L)`.

For each output frame and channel, accumulate in fixed launch order:

- `A_c(y)=sum_j w_j(y)*s_c(p_j(y))`
- `W(y)=sum_j w_j(y)*m(p_j(y))`

Require `W(y)>=2^-20` for every non-empty output frame. A lower value is a
processing error; it is never repaired with held audio or a fill pass. Before
the exterior envelope:

`z_c(y)=A_c(y)/W(y)`.

The denominator is channel shared and source-content independent. The
non-negative interpolation and window coefficients make `z` a convex
combination of valid source samples. Before rounding, its channel peak cannot
exceed the corresponding source peak. There is no limiter, compressor, RMS
balancer, adaptive gain, feedback, spectral processor, or post-render wet mix.

## Linked Stereo

Mono uses the same renderer with one channel. Stereo reads native left and
right samples with the identical launch set, duration choice, source
coordinate, interpolation weights, window, denominator, and exterior factor.

At this canonical setting, duplicate, channel swap, common polarity, and
anti-phase transforms commute samplewise apart from signed-zero
canonicalization. Mono duplicated to stereo decodes back to the mono render.
No panning, decorrelation, channel-local seed, channel-local normalization, or
`space` widening exists in the first candidate.

Natural stereo balance and local image remain complete diagnostics and
listening invariants. The exact algebra above is structural and terminal.

## Boundaries And Exact Length

Reuse the admitted Dream exterior geometry so a future upper overlap does not
start with a second boundary law. Define:

`N_b=clamp(nearest_power_of_two(round_half_up(2F/3)),8192,131072)`

and `H_b=N_b/2`. Power-of-two ties select the larger value.

Entry extent:

`E_head=min(ceil(F/200),floor(T/4))`.

Release extent:

`E_tail=min(2H_b,floor(T/4))`.

For extent `E>=2`, head factor is `sin((pi/2)y/(E-1))` and tail factor is
`sin((pi/2)(T-1-y)/(E-1))`. Extent one sets its sole endpoint to zero; extent
zero applies no factor. Multiply overlapping factors, emit exactly `[0,T)`,
and canonicalize exact-zero endpoints.

No append, wrap, reflection, resize-fill, edge hold, or material-dependent
fade is allowed. The validity-mask normalization owns source edges; the shared
fixed envelope owns output edges.

## Determinism, State, And Cost

Allocate returned output and all fixed working storage before frame zero.
Excluding borrowed input and returned output capacity, peak working state is at
most `4 MiB` for stereo and independent of duration. No allocation,
reallocation, lock, I/O, or blocking occurs after rendering starts.

Traversal is single-threaded. Launch order, counter addresses, interpolation,
summation order, comparisons, and zero canonicalization are fixed. Identical
requests are byte-identical on the frozen platform and toolchain identity.

Work is `O(C*T)` with at most `22` active grains and two interpolation taps per
grain, per channel, per output frame. Output duration dominates cost at `100x`;
the renderer has no duration-derived working array. Offline only. No audio-
thread execution or source fill is authorized.

## Conformance Authority

Candidate implementation follows Contract `085` Rule 11. Compile-linked
`RENDER_SPEC`, `SOURCE_SPEC`, `EVIDENCE_SPEC`, `MEMORY_SPEC`, and `RUN_SPEC`
own every formula, source, seed, matrix, assertion, receipt field, row count,
render count, and execution limit. Construction calls every non-rendering
oracle and proves that every acoustic assertion maps to one owner.

Structural owners are:

| ID | Owner | Rows | Renders | Frozen boundary |
| --- | --- | ---: | ---: | --- |
| S01 | request and allocation | 17 | 2 | complete valid/invalid matrix; rejection before output allocation |
| S02 | map and launches | 15 | 0 | checked map vectors; monotonic centres; exact endpoint launch; ratios `16`, `24`, `32`, `64`, `100` |
| S03 | counter and geometry | 13 | 0 | exact `mix64` vectors; duration range; active-grain maximum `22` |
| S04 | sampling and window | 22 | 0 | interpolation/mask vectors; window symmetry, positivity, and exact inactive support |
| S05 | normalization | 9 | 9 | denominator floor; convex silence, DC, and peak bounds; fixed summation order |
| S06 | boundaries | 12 | 12 | Dream-matched entry/release vectors; exact length and zero endpoints |
| S07 | linked stereo | 9 | 21 | exact duplicate, swap, polarity, anti-phase, and mono-decode algebra |
| S08 | determinism and memory | 4 | 7 | byte repeat; changed-seed activity; `4 MiB` counting-allocator ceiling; no render-time allocation |

Structural totals are exactly `101` rows and `51` renders. Row slices are:

- `S01`: empty success; valid `L=4096` mono `16x` and stereo `100x`;
  non-empty `L=H-1`; channels `0` and `3`; partial stereo frame; rates `7999`
  and `192001`; separate NaN and infinity inputs; non-empty zero target; empty
  non-zero target; targets `16L-1` and `100L+1`; direct dimension oracles above
  `2^53-1` for `L` and `T`. Only the two valid materialized requests render;
  the short source fails before allocation.
- `S02`: ratios `16`, `24`, `32`, `64`, and `100` crossed with sample rates
  `8000`, `48000`, and `192000`. Each row asserts checked map values at the
  first, middle, and last output frames, strict map monotonicity, exact regular
  launches, sorted/deduplicated terminal launch, and source-centre equality.
- `S03`: the three primitive and five complete-address vectors above; one
  `j=0..63` duration-range row; geometry at `8000`, `48000`, and `192000`;
  and one analytic-plus-exhaustive occupancy proof row.
- `S04`: interpolation and validity-mask vectors at `p=-1,-0.5,0,0.25,L-1,
  L-0.5,L`; for representative durations `12H`, `16H`, and `20H`, window
  vectors at centre, signed quarter support, the last active offset, and both
  inactive boundaries.
- `S05`: exact silence, DC `0.25`, and alternating `-0.5/+0.5` crossed with
  ratios `16`, `32`, and `100` at `F=48000`, `L=4096`.
- `S06`: ratios `16`, `32`, and `100` crossed with source lengths
  `H,H+1,2H,12H` at `F=48000`.
- `S07`: at each ratio, one mono/duplicate pair, one natural fixture with its
  swap and common negation, and one mono/anti-phase pair. That is three rows
  and seven renders per ratio.
- `S08`: byte repeat at mono `16x`; active seed change at mono `16x`; finite
  stereo `100x` with seed `u64::MAX`; counting-allocator stereo renders at
  `16x` and `100x`. The last row asserts identical planned working capacity,
  the `4 MiB` ceiling, and no render-time allocation.

Run the complete compile, construction, and `S01..S08` sequence until it passes
twice unchanged. Record every corrective conformance diff. A DSP, source,
seed, helper, metric, threshold, assertion, comparator, or listening-policy
choice stops for docs reassessment.

The clean conformance tree is committed and referenced at
`refs/signal-evidence/creative/layered-cloud/31-70-acoustic` before any
synthetic render or comparator output runs.

The tracked nextest profile is exactly:

```toml
[profile.layered-cloud]
retries = 0
fail-fast = false
test-threads = 1
slow-timeout = { period = "10800s", terminate-after = 1 }
failure-output = "immediate-final"
success-output = "never"
status-level = "all"
final-status-level = "all"
```

From one clean local commit, run compile, construction, and exact-name
`layered_cloud_structural_s01` through `s08` in numeric order with release
profile, `-j 1`, no fail-fast, and immediate failure output. Construction must
pass `1/1`; structural owners must pass `8/8`, `101/101` rows, and `51/51`
renders. Repeat the complete sequence unchanged and require identical counts.
Record the commit/tree, all candidate and evidence hashes, frozen specs,
`Cargo.lock`, `rustc -vV`, Effigy/nextest versions, OS, architecture, and the
corrective-diff ledger before creating the acoustic ref.

Every owner traverses its rows serially. Each row appends and syncs one
canonical JSON line before the next starts. Keys, in order, are `schema`,
`checkpoint`, `stage`, `round`, `owner`, `row_index`, `row_id`, `status`,
`render_count`, `output_frames`, `input_sha256`, `output_sha256`, `assertions`,
and `diagnostics`. The terminal summary records expected and completed owner,
row, and render counts. Receipts contain hashes, not samples.

## Synthetic Admission

All sources use `F=48000`, `L=32768`, exact ratios `16`, `32`, and `100`, and
`ADMISSION_SEED`. Let `G=256`. The guard is
`g(n)=sin^2((pi/2)n/(G-1))` for `n<G`, the mirrored expression for
`n>=L-G`, where `g(n)=g(L-1-n)`, and one otherwise. Source construction
evaluates the written formula in `f64`, multiplies by `g` where named, then
casts once to `f32`. Silence, DC, and impulses are unguarded. Define
`TEST=0x434c4f5544545354` and
`r(n)=2*((mix64(n xor TEST)>>11)/2^53)-1`.

Freeze these sources:

- exact silence
- DC `0.25`
- sinusoid `g(n)*0.5*sin(2*pi*375n/F)`
- chord `g(n)*(sin(2*pi*250n/F)+sin(2*pi*375n/F)+sin(2*pi*625n/F))/6`
- counter-addressed uniform noise `g(n)*0.5*r(n)`
- amplitude-modulated noise `g(n)*0.5*r(n)*sin^2(2*pi*n/F)`
- one impulse of `0.75` at `L/2`
- two impulses of `0.75` at `L/4` and `3L/4`
- stereo relation fixture: left
  `g(n)*(0.25*sin(2*pi*375n/F)+0.125*r(n))`; right
  `g(n)*(0.25*sin(2*pi*625n/F)-0.125*r(n))`

Acoustic owners run in this order:

| ID | Rows | Renders | Matrix | Terminal pass condition |
| --- | ---: | ---: | --- | --- |
| Y01 | 9 | 9 | silence, DC, uniform noise at all ratios | exact length and finiteness; silence bit-exact; DC follows only the frozen exterior factor within `2e-6`; output peak no greater than source peak plus four `f32` ULPs |
| Y02 | 6 | 6 | tone and chord at all ratios | complete finite dominant-frequency diagnostics; no missing target component; chord target components retain ascending frequency order; numeric pitch is diagnostic for Cloud listening |
| Y03 | 6 | 6 | one and two impulses at all ratios | every exact non-zero sample lies inside the geometry-derived support union; contribution centroids retain authored order; complete finite lobe-count, lobe-spacing, and off-centre-energy diagnostics |
| Y04 | 3 | 3 | amplitude-modulated noise at all ratios | every consecutive one-second block starting at `E_head` and wholly inside `[E_head,T-E_tail)` has a distinct byte hash; no zero denominator, exact repeated block, dropout, or non-finite sample |
| Y05 | 9 | 21 | duplicate, swap, common polarity, anti-phase, and natural stereo at all ratios | exact algebra for constructed transforms; complete finite whole, three-band, and mapped-window diagnostics for natural stereo |

Synthetic totals are exactly `33` rows and `45` renders. `Y05` uses the same
three relational rows per ratio as `S07`; its natural-fixture base owns the
whole, three-band, and mapped-window diagnostics. No owner may add a row or
render implicitly.

`Y02` uses the admitted DirectRenewalDream log-magnitude FFT peak estimator:
middle supported window, periodic Hann, zero-padding to the next power of two,
nearest expected peak search, and three-bin parabolic log interpolation.
Missing or non-finite evidence rejects; finite pitch delta is diagnostic.

`Y03` computes each event's support set and energy centroid
`sum_y(y*v(y)^2)/sum_y(v(y)^2)` from
the exact launch set, duration, window predicate, source interpolation mask,
and authored impulse position. The one-event render must be zero outside its
set. The paired render must be zero outside the union, and the two authored
centroids must remain strictly ordered. Connected non-zero lobe count, median
lobe spacing, and energy outside one launch hop of each centroid are mandatory
finite diagnostics, not invented numeric vetoes. Audible metallic replication
remains a long-form rejection.

For `Y04`, dropout means `1024` consecutive exact-zero frames wholly inside
`[E_head,T-E_tail)`. Consecutive one-second blocks begin at `E_head+kF`; only
complete blocks ending at or before `T-E_tail` participate.

`Y05` uses the existing whole/three-band and four-second/two-second-hop mapped
stereo diagnostics. Missing or non-finite evidence rejects. Finite natural-
stereo balance, correlation, dominance, and width values are listening inputs,
not invented numeric Cloud thresholds.

After the ref, run exact-name `layered_cloud_y01` through `y05` individually in
numeric order under the same release profile, `-j 1`, no fail-fast, and
immediate failure output. Each structural row has an internal `120 s` deadline;
each synthetic row has `900 s`. Timeout or a missing synced incremental receipt
fails the owner. Later owners do not start after a failure.

## Comparator And Listening Gate

After `Y01..Y05` pass, use the five retained `44.1 kHz`, `220500`-frame source
files and hashes from the DirectRenewalDream brief. Render mono and original
stereo at exact `16x`, `32x`, and `100x`.

Mono and stereo each contain exactly `15` source/ratio rows. Each row contains
three concealed outputs: Signal, PaulX, and Csound. Each pack therefore owns
`45` renders; the complete listening gate owns `30` rows and `90` renders.

Required comparators:

- PaulXStretch `1.6.0`, default, FFT control `16384`
- Csound at revision `0eaa07e3aee55f90e745f89294ddb52eec30345c`,
  `sndwarp`/`sndwarpst`, pointer mode, unit resample, documented Hann window,
  window `F/10`, overlap `15`, and random-width band `window/5`

Comparator constants describe external captures only. They do not alter the
Signal renderer. Drive each comparator pointer from source start to source end
over exact target duration, crop to `T`, record toolchain and decoded-output
hashes, and reject missing or non-finite captures.

Within each source/ratio row, source, candidate, PaulX, and Csound share one RMS
target reduced only enough to keep every peak at or below `0.95`. Conceal
identity. Review `32x` and `100x` first, then `16x`.

Mono passes only when:

- all `15` candidate rows are musically usable
- at least `8/10` primary `32x`/`100x` rows are preferred or tied against at
  least one comparator
- no source family loses both primary ratios
- no row has a click, dropout, static freeze, obvious periodic flutter,
  metallic replica, arbitrary level step, reversed event order, or unusable
  entry/tail behavior

Record cloud usefulness, source identity, tonal focus, motion, grain density,
periodicity, transient spread, low-frequency noise, event order, level, entry,
and tail. Metrics diagnose; listening decides.

Original-stereo review uses all `15` rows after mono passes. Hard controls are
exact length, finiteness, repeatability, the samplewise transform algebra, and
complete natural-stereo diagnostics. The operator may reject a speaker pre-
screen. Promotion requires an eligible independent listener; the
DirectRenewalDream waiver does not transfer. Every row must be usable, at
least `8/10` primary rows must be preferred or tied against one comparator,
and no source family may lose both primary ratios. Centre loss, one-sided pull,
unrelated motion, unstable width, low-frequency imbalance, or an image jump
rejects.

## Upper-Overlap Boundary

`LayeredCloud` supports every fixed ratio from `16x` through `100x`, uses the
Contract `085` map, and shares DirectRenewalDream's exterior envelope. It can
therefore supply its side of every future `16x..32x` probe.

The upper overlap remains paused. Admitted DirectRenewalDream renders exact
`16x` but no interior ratio. Do not hard-switch at `16x`, resample Dream, run a
second stretch pass, or infer a blend from one shared endpoint. A separately
versioned generalized Dream owner must pass continuous fixed-ratio admission
before upper-overlap implementation can open.

## Isolation, Rejection, And Admission

Batch 31.70 may create only:

- worktree `signal-candidate-31-70`
- branch `candidate/g10-031-layered-cloud`
- private module `crates/signal-dsp-stretch/src/creative_layered_cloud/`
- local evidence ref named above
- ignored outputs under `target/creative-stretch-layered-cloud-31-70/`

Existing dependencies only. No production renderer, public API, report binary,
fixture, routing, cache, artifact, dynamic-ratio, Loophole, or Chorus change is
allowed.

Any synthetic, mono, or stereo miss rejects the immutable checkpoint. Record
the dominant cause and complete persisted receipt. Do not tune or rerun after
acoustic identity. Delete the worktree, branch, build state, generated audio,
and candidate source after the required reassessment; retain the local evidence
ref only until that decision closes.

If every gate passes, later admission may copy only the private fixed-ratio
renderer, request/error boundary, regression owners, receipt schema, and one
new internal engine version. Product routing, macros, seed exposure, cache,
artifacts, dynamic ratio, and cross-repo surfaces remain separate.

## Batch 31.70 Pre-Conformance Reconciliation

The first construction audit stopped before candidate source. The original
request admitted every non-empty `L`, while `S06` required successful renders
at `L=1` and `L=H-1`. Unit-rate grains on the frozen `H`-spaced lattice cannot
cover every output frame of a source shorter than one hop; `W(y)` reaches zero
before the exterior envelope.

This brief now rejects non-empty `L<H` before output allocation and replaces
the two impossible success geometries with `2H` and `12H`. The renderer,
counter, launch lattice, sampling, normalization, boundary, stereo, acoustic,
listening, cleanup, and admission laws are unchanged. Structural authority is
now exactly `101` rows and `51` renders. No candidate DSP or acoustic output
existed before this correction.

## Batch 31.70 Evidence Result

Checkpoint `ee42f50c4c338db4af8a7feaa89bb8b21e8d0860`, tree
`cfc28c8c6c4095f0c91ae95d0724962656bcec97`, passed two unchanged complete
compile, construction `1/1`, and structural `8/8` rounds. Both structural
receipts completed `101/101` rows and `51/51` renders. The immutable checkpoint
then returned green for `Y01..Y05`, totaling `33/33` rows and `45/45` renders.

The synthetic receipt is invalid. Its compiled `Y05` helper calculated only
whole-buffer balance, correlation, and width. It did not implement the frozen
three-band or four-second/two-second-hop mapped-window diagnostics, and its
receipt persisted no natural-stereo diagnostic values. Construction therefore
failed its Rule 11 completeness duty even though it returned green before the
checkpoint was created.

No Cloud quality conclusion follows. Long-form mono, comparator-relative
stereo, and listening did not open. Do not repair, rerun, or promote this
checkpoint. Retain its isolated state only through Batch 31.71's docs-level
evidence-integrity decision.

## Sources

- [Csound `sndwarpst` manual](https://csound.com/manual/opcodes/sndwarpst/)
- [Csound pointer and balance guidance](https://csound.com/manual/opcodes/sndwarp/)
- [Pinned Csound `sndwarp`/`sndwarpst` source](https://github.com/csound/csound/blob/0eaa07e3aee55f90e745f89294ddb52eec30345c/Opcodes/sndwarp.c)
- [SuperCollider `Warp1` manual](https://docs.supercollider.online/Classes/Warp1.html)
- [Pinned SuperCollider `Warp1` implementation](https://github.com/supercollider/supercollider/blob/2f0803bcd2e551564e3fef8d5075816cbb685cd4/server/plugins/GrainUGens.cpp)
- [DirectRenewalDream boundary authority](./offline-creative-direct-renewal-dream-brief.md)

## Next Task

Run Batch 31.71 only. Audit every executable acoustic helper, assertion,
diagnostic, receipt field, and construction edge, then decide whether the
still-unjudged topology warrants one fresh audited identity. Do not repair or
rerun Batch 31.70. Keep admitted renderers, product routing, controls, cache,
dynamic ratio, Loophole, and Chorus unchanged. Do not push.
