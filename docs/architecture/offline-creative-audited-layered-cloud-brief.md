# Offline Creative AuditedLayeredCloud Renderer Brief

Status: frozen; one fresh isolated implementation ready
Owner: dsp
Updated: 2026-07-22
Contract: `085`, Rule 11
Roadmap: `g10.031`, Batches 31.72-31.73

## Decision

Freeze one source-clean `AuditedLayeredCloud` renderer for fixed creative
expansion from `16x` through `100x`. It is one pointer-led granular system:
one exact source map, one deterministic launch lattice, bounded variable-length
unit-rate grains, one channel-shared validity normalization law, and the
admitted Dream exterior envelope.

The sound architecture is unchanged from the closed LayeredCloud brief. The
new identity exists because Batch 31.70 produced invalid evidence, not because
its unjudged audio suggested a renderer change. Batch 31.71 deleted that
worktree, branch, source, build state, generated output, and evidence ref. None
may be recovered or compared during implementation.

This brief replaces the closed brief as execution authority. It closes every
gap in the Batch 31.71 ledger before source exists: canonical compile-linked
specs, exact source hashes and vectors, assertion-to-owner construction,
per-row process deadlines, truthful receipts, complete synthetic diagnostics,
and executable comparator/listening assembly and validation.

The architecture is clean-room Signal work informed by Csound `sndwarpst` and
independently supported by SuperCollider `Warp1`. No upstream code, constants,
tables, random generator, or control flow transfers. No external production
dependency is introduced.

## Renderer Authority

### Request and map

The private request contains finite mono or interleaved stereo `f32` input,
`channels` equal to `1` or `2`, sample rate `F` in `8000..=192000`, exact
`target_frames=T`, and `seed: u64`. Source frame count is `L`.

Empty input with `T=0` returns empty. For non-empty input define
`H=max(1,round_half_up(F/64))` and require:

- complete interleaved frames
- `L>=H`
- `16L<=T<=100L`
- `L<=2^53-1` and `T<=2^53-1`
- checked input, output, launch, and byte-size arithmetic

All invalid requests fail before output allocation. Values are rejected, never
clamped. The renderer has no motion, detail, space, pitch, reverse, dynamic
ratio, public routing, cache, artifact, realtime, or product surface.

The sole source map is:

`x(y)=((y+0.5)L/T)-0.5`.

Evaluate `(2y+1)L/(2T)` with checked `u128`, convert once to `f64`, then
subtract `0.5`. Launch centres are `0,H,2H,...<T`, plus `T-1`, sorted and
deduplicated. Launch `j` at `y_j` owns `x_j=x(y_j)`. Centres are strictly
increasing. There is no jitter, alignment search, accumulated cursor, or
second timeline.

### Counter and grain geometry

`mix64` uses wrapping `u64` arithmetic:

1. `z=(z xor (z>>30))*0xBF58476D1CE4E5B9`
2. `z=(z xor (z>>27))*0x94D049BB133111EB`
3. return `z xor (z>>31)`

Freeze `CLOUD_TAG=0x434c4f55445f4455` and
`ADMISSION_SEED=0x4c41594552434c44`. For launch ordinal `j`:

`D_j=12H+(mix64(seed xor CLOUD_TAG xor j) mod (8H+1))`.

At output frame `y`, signed offset is `q=y-y_j`. The grain is active only
when `2*abs(q)<D_j`. Its window and unit-rate source coordinate are:

- `w_j(y)=0.5+0.5*cos(2*pi*q/D_j)`
- `p_j(y)=x_j+q`

At most `22` grains contribute: at most `21` regular `H`-lattice launches
under strict support `D<=20H`, plus the one terminal launch. Production uses
fixed capacity `22`; overflow is a processing error.

### Sampling and normalization

For `p`, let `i=floor(p)` and `u=p-i`. Samples outside `[0,L)` are positive
zero. For each channel:

- `s_c(p)=(1-u)a_c[i]+u*a_c[i+1]`
- `m(p)=(1-u)I(0<=i<L)+uI(0<=i+1<L)`
- `A_c(y)=sum_j w_j(y)s_c(p_j(y))`
- `W(y)=sum_j w_j(y)m(p_j(y))`
- `z_c(y)=A_c(y)/W(y)`

Accumulate in increasing launch order with ordinary `f64` multiply then add;
no reassociation, SIMD reduction, fused reduction, or compensated second path
is allowed. Require `W>=2^-20` at every non-empty frame. A miss is an error,
never a fill, hold, or repair. The shared non-negative normalization makes the
pre-envelope output a convex combination of valid source samples.

### Linked channels

Stereo reads native left and right samples with the same launches, durations,
coordinates, interpolation weights, windows, denominator, and exterior
factor. Apart from canonical positive zero, duplicate, channel swap, common
negation, and anti-phase commute samplewise. Rendering mono and duplicating it
equals rendering duplicated stereo, and decoding that stereo by
`f32((f64(L)+f64(R))/2)` equals the mono render.

There is no panning, decorrelation, channel-local seed, channel-local
normalization, or widening control.

### Boundary and exact length

Use the admitted Dream exterior geometry:

- `N_b=clamp(nearest_power_of_two(round_half_up(2F/3)),8192,131072)`, with
  power-of-two ties selecting the larger value
- `H_b=N_b/2`
- `E_head=min(ceil(F/200),floor(T/4))`
- `E_tail=min(2H_b,floor(T/4))`

For extent `E>=2`, head factor is `sin((pi/2)y/(E-1))`; tail factor is
`sin((pi/2)(T-1-y)/(E-1))`. Extent one zeros its sole endpoint; extent zero
does nothing. Multiply overlapping factors, emit exactly `[0,T)`, and
canonicalize exact-zero endpoints.

No append, wrap, reflection, resize-fill, edge hold, or material-dependent
fade exists. Validity normalization owns source edges; this fixed envelope
owns output edges.

### State, determinism, and cost

Allocate the returned output and fixed working storage before frame zero.
Excluding borrowed input and returned output capacity, stereo working state is
at most `4 MiB` and independent of `L`, `T`, and ratio. No allocation,
reallocation, lock, I/O, blocking, thread creation, or duration-derived array
occurs after rendering starts.

Traversal is single-threaded. Identical valid requests are byte-identical on
the frozen platform/toolchain. Cost is `O(C*T)`, with at most `22` grains and
two interpolation taps per grain/channel/frame. Offline only.

## Canonical Executable Authority

The isolated module owns exactly these compile-linked, immutable values:

| Spec | Sole ownership |
| --- | --- |
| `RENDER_SPEC` | request, map, launches, counter, grain, interpolation, normalization, stereo, boundary, crop |
| `SOURCE_SPEC` | every structural, synthetic, and long-form source formula, file name, channel count, frame count, seed, and SHA-256 |
| `VECTOR_SPEC` | exact counter, map, launch-count, interpolation, window, boundary, and occupancy vectors |
| `EVIDENCE_SPEC` | ordered owners, rows, renders, assertions, diagnostics, and pass/fail rules |
| `MEMORY_SPEC` | capacities, byte accounting, `4 MiB` ceiling, allocation phases |
| `RUN_SPEC` | test names, profiles, filters, order, deadlines, environment, receipt paths, expected totals |
| `COMPARATOR_SPEC` | source revisions, build/capture identity, commands, settings, crop, level match, concealment |
| `LISTENING_SPEC` | row order, fields, arithmetic, reviewer eligibility, terminal decisions |
| `CLEANUP_SPEC` | isolated identities, retained evidence lifetime, deletion and minimal-pass surface |

No helper contains a second source endpoint, seed, ratio, scalar, metric,
threshold, row count, assertion, diagnostic, receipt field, comparator setting,
or listening rule.

`ASSERTION_CATALOG` and `DIAGNOSTIC_CATALOG` assign a stable string ID and
implementing function pointer to every named check. `ROW_SPECS` contains one
entry per row with stage, owner, ordinal, exact test name, executor pointer,
source IDs, ratio, channels, render count, per-render frame count, assertion
IDs, diagnostic IDs, receipt-field mask, and deadline class.

Construction proves:

- all catalog IDs, row IDs, and test names are unique
- every row assertion and diagnostic resolves to exactly one function pointer
- every catalog entry is used by at least one frozen row or summary validator
- row and render totals equal this brief
- every receipt field is produced from its named owner, not a generic pass tag
- every referenced source regenerates to its frozen SHA-256
- every non-rendering oracle returns the frozen vector below
- comparator capture, concealment, level matching, mono decision, stereo
  decision, and terminal cleanup each have compile-linked owners
- each row is one test process governed by a tracked nextest timeout

Construction may call non-rendering oracles and source hash builders. It may
not render the candidate, run an acoustic row, invoke a comparator, create a
listening pack, or inspect audio.

## Frozen Exact Vectors

Primitive counter vectors are:

| Input | Result |
| --- | --- |
| `0` | `0x0000000000000000` |
| `1` | `0x5692161d100b05e5` |
| `u64::MAX` | `0xb4d055fcf2cbbd7b` |

Complete addresses at `H=750` are:

| `j` | `mix64(ADMISSION_SEED xor CLOUD_TAG xor j)` | `D_j` |
| ---: | --- | ---: |
| 0 | `0x21ff9c0ce71cf025` | 14565 |
| 1 | `0x07696f531ec8a5a5` | 14048 |
| 2 | `0xdd9ceeba647a4de2` | 9952 |
| 7 | `0xa9ddfca3dcb6d346` | 14687 |
| 11 | `0xbc8b4ab4f7f4abd6` | 12242 |

For `L=4096`, map results are frozen as `f64::to_bits()`:

| Ratio | `y=0` | `y=T/2` | `y=T-1` |
| ---: | --- | --- | --- |
| 16 | `0xbfde000000000000` | `0x409ffe2000000000` | `0x40affef000000000` |
| 24 | `0xbfdeaaaaaaaaaaab` | `0x409ffe1555555556` | `0x40affef555555555` |
| 32 | `0xbfdf000000000000` | `0x409ffe1000000000` | `0x40affef800000000` |
| 64 | `0xbfdf800000000000` | `0x409ffe0800000000` | `0x40affefc00000000` |
| 100 | `0xbfdfae147ae147ae` | `0x409ffe051eb851ec` | `0x40affefd70a3d70a` |

Exact total launch counts, including a distinct terminal launch, are:

| `F/H` | 16x | 24x | 32x | 64x | 100x |
| --- | ---: | ---: | ---: | ---: | ---: |
| `8000/125` | 526 | 788 | 1050 | 2099 | 3278 |
| `48000/750` | 89 | 133 | 176 | 351 | 548 |
| `192000/3000` | 23 | 34 | 45 | 89 | 138 |

`S04` uses source `[1.0,-0.5,0.25,0.75]`. Its exact `(sample,mask)` results
at `p=-1,-0.5,0,0.25,3,3.5,4` are `(0,0)`, `(0.5,0.5)`, `(1,1)`,
`(0.625,1)`, `(0.75,1)`, `(0.375,0.5)`, and `(0,0)`. For `D=12H,16H,20H`
at `H=750`, `w(0)=1`, `w(+-D/4)=0.5`, `w(q)>0` at
`q=+-(D/2-1)`, and both `q=+-D/2` are inactive. Transcendental window values
are compared to the written `f64` formula within `2^-48`; symmetry and the
inactive values are exact.

Boundary geometry vectors are:

| `F` | `N_b` | `H_b` | unclamped `E_head` | maximum `E_tail` |
| ---: | ---: | ---: | ---: | ---: |
| 8000 | 8192 | 4096 | 40 | 8192 |
| 48000 | 32768 | 16384 | 240 | 32768 |
| 192000 | 131072 | 65536 | 960 | 131072 |

For every `S06` extent, construction evaluates indices
`0,1,floor(E/2),E-2,E-1` from the formula independently of the renderer,
requires absolute error at most `2^-48`, monotonic entry, mirrored release,
and exact endpoint values. This is the complete entry/release vector rule, not
an endpoint-only check.

The occupancy proof enumerates all `F=8000..192000`, all integer
`D=12H..20H`, every output residue `y mod H`, and terminal residues
`(T-1) mod H=0..H-1`. It derives the regular count from the strict predicate,
adds the terminal only when distinct, and requires observed maximum `22`.
Comparing a hard-coded `21+1` to capacity is forbidden.

`VECTOR_SPEC` owns these exact UTF-8 bytes with LF after every row, including
the final row:

```text
mix64|0|0000000000000000
mix64|1|5692161d100b05e5
mix64|18446744073709551615|b4d055fcf2cbbd7b
duration|0|21ff9c0ce71cf025|14565
duration|1|07696f531ec8a5a5|14048
duration|2|dd9ceeba647a4de2|9952
duration|7|a9ddfca3dcb6d346|14687
duration|11|bc8b4ab4f7f4abd6|12242
map|16|bfde000000000000|409ffe2000000000|40affef000000000
map|24|bfdeaaaaaaaaaaab|409ffe1555555556|40affef555555555
map|32|bfdf000000000000|409ffe1000000000|40affef800000000
map|64|bfdf800000000000|409ffe0800000000|40affefc00000000
map|100|bfdfae147ae147ae|409ffe051eb851ec|40affefd70a3d70a
launch|8000|125|526|788|1050|2099|3278
launch|48000|750|89|133|176|351|548
launch|192000|3000|23|34|45|89|138
interp|-1|0|0
interp|-0.5|0.5|0.5
interp|0|1|1
interp|0.25|0.625|1
interp|3|0.75|1
interp|3.5|0.375|0.5
interp|4|0|0
boundary|8000|8192|4096|40|8192
boundary|48000|32768|16384|240|32768
boundary|192000|131072|65536|960|131072
occupancy|max|22
```

Its SHA-256 is
`b2312216510b7fc1079ab12039962abcc5fa9149720f7613e935d05e983c507c`.
Construction regenerates and hashes it; a byte, order, count, or value change
rejects before candidate rendering.

## Structural Admission

Every row is a separately named nextest test process. Owners and totals remain:

| ID | Rows | Renders | Required assertions |
| --- | ---: | ---: | --- |
| S01 request/allocation | 17 | 2 | valid/invalid result, exact error, checked dimensions, zero output allocation on every invalid row |
| S02 map/launches | 15 | 0 | exact map bits, every regular launch/count, terminal add-or-dedupe, monotonicity, `x_j=x(y_j)` |
| S03 counter/geometry | 13 | 0 | exact vectors, duration reach/range, derived exhaustive occupancy |
| S04 sampling/window | 22 | 0 | exact interpolation/mask, formula values, symmetry, positivity, both inactive boundaries |
| S05 normalization | 9 | 9 | `W>=2^-20`, exact silence/DC normalization before envelope, fixed launch order, convex peak |
| S06 boundaries | 12 | 12 | complete entry/release vectors, extent/crop, exact length and zero endpoints |
| S07 linked stereo | 9 | 21 | duplicate, swap, common negation, anti-phase, and mono-decode algebra |
| S08 determinism/memory | 4 | 7 | byte repeat, seed activity, finiteness, planned-capacity equality, ceiling, no render allocation |

Totals are exactly `101` rows and `51` renders. Row matrices are exactly those
in the closed brief: `S01` has empty success, valid `L=4096` mono `16x` and
stereo `100x`, and the fourteen named invalid/dimension rows; `S02` crosses
ratios `16,24,32,64,100` with rates `8000,48000,192000`; `S03` owns the eight
counter rows, duration range, three rate geometries, and occupancy; `S04` owns
the seven interpolation rows and five window assertions for each of three
durations; `S05` crosses silence, DC `0.25`, and alternating `-0.5/+0.5` with
`16x,32x,100x`; `S06` crosses those ratios with `L=H,H+1,2H,12H`; `S07`
owns three relational rows and seven renders per ratio; `S08` owns repeat,
seed change, extreme-seed stereo, and the two counting-allocation renders.

`S01` wraps the allocator before validation and proves both allocation count
and allocated bytes remain zero for every invalid row. `S05` records minimum
`W`, first failing frame or null, pre-envelope DC residual, fixed-order output
hash, and output peak. `S08` compares the complete planned-capacity struct for
`16x` and `100x`, not only total bytes.

## Synthetic Sources And Hashes

All use `F=48000`, `L=32768`, exact `16x`, `32x`, `100x`, and
`ADMISSION_SEED`. Let `G=256`. The guard is
`g(n)=sin^2((pi/2)n/(G-1))` for `n<G`, mirrored for `n>=L-G`, and one
otherwise. Evaluate the written formula in `f64`, multiply by the guard where
named, and cast once to `f32`. Silence, DC, and impulses are unguarded. Define
`TEST_TAG=0x434c4f5544545354` and
`r(n)=2*((mix64(n xor TEST_TAG)>>11)/2^53)-1`.

Little-endian `f32` source hashes for Rust `1.96.0` on
`aarch64-apple-darwin` are:

| Source | Formula | SHA-256 |
| --- | --- | --- |
| silence | `0` | `fa43239bcee7b97ca62f007cc68487560a39e19f74f3dde7486db3f98df8e471` |
| DC | `0.25` | `cbfac0f203c117852ee0c1bbac1033b7ded580f76850403de1db09412a252a40` |
| tone | `g*0.5*sin(2*pi*375n/F)` | `a4bf41139fd67afbb1171a6cdc22126a0f88a132b5e289f4fdd7564d56a3fc42` |
| chord | `g*(sin(250)+sin(375)+sin(625))/6` | `653db70c54e0e55a2328314665a5ad6962e30593f5667fdbc7825364695171f2` |
| uniform noise | `g*0.5*r(n)` | `508887a00d7de131c87ae621a04b4f3def5984eab438e01abc5a629c6c7d5473` |
| AM noise | `g*0.5*r(n)*sin^2(2*pi*n/F)` | `c8c43a2ea2cfdb0b0ef1d3ff2b9cd71c4859a93ab11a0fec1c08a34da77c85ce` |
| one impulse | `0.75` at `L/2` | `60b7437fbace983f609327c5eac49b338781a6fdb280cce790dc815139c61719` |
| two impulses | `0.75` at `L/4,3L/4` | `0ae48df05f4d18df5218922e5bba8841395542ada568dcaaf153a367bcf0cea7` |
| stereo relation | left `g*(0.25*sin(375)+0.125r)`; right `g*(0.25*sin(625)-0.125r)` | `e70b2593d2be472340c6e6feb99d5782aae7d2dd43f70812150a45184f79c501` |

Construction rejects a toolchain/platform source-hash mismatch before a
checkpoint. It cannot update the hashes locally.

## Synthetic Admission

| ID | Rows | Renders | Terminal pass |
| --- | ---: | ---: | --- |
| Y01 integrity | 9 | 9 | exact length/finiteness; bit-exact silence; DC exterior residual `<=2e-6`; peak no greater than source peak plus four `f32` ULPs |
| Y02 components | 6 | 6 | every authored component present by the frozen energy rule; finite persisted peak/pitch diagnostics; chord estimates strictly ordered |
| Y03 events | 6 | 6 | exact support containment; persisted finite centroids in authored order; complete lobe diagnostics |
| Y04 continuity | 3 | 3 | no zero-denominator/dropout; unique complete one-second blocks; persisted final interior remainder |
| Y05 linked stereo | 9 | 21 | exact transform algebra; complete persisted whole, three-band, and mapped-window diagnostics |

Totals are exactly `33` rows and `45` renders.

`Y02` takes exactly `131072` centered frames from
`[E_head,T-E_tail)`, applies a periodic Hann, and zero-pads to `1048576` FFT
points. Power is squared complex magnitude over the one-sided bins excluding
DC. For each expected `250`, `375`, or `625 Hz` component, component energy is
the inclusive bin sum within `+-4 Hz`; total energy is the complete one-sided
sum. A component is present only when both are finite, total is positive, and
`component/total>=2^-10`. This is an integrity floor: it rejects a missing
line and broadband-only substitution without turning pitch error into a Cloud
quality veto. The highest-power bin inside the band gets three-bin parabolic
log-power interpolation. Persist expected Hz, band bounds, component energy,
total energy, ratio, estimated Hz, and cents error for every component.

`Y03` derives event support from the exact launch set, duration, active
predicate, source interpolation mask, and impulse position. Output must be
exact zero outside the support or union. Persist each contribution centroid
`sum(y*v(y)^2)/sum(v(y)^2)`, denominator, connected non-zero lobe count,
median lobe spacing, and energy outside one launch hop of the centroid. The
two-event centroids must be strictly ordered. No numeric replica threshold is
invented; audible metallic repetition remains a listening rejection.

`Y04` scans every frame of `[E_head,T-E_tail)` once while maintaining the
current exact-zero run, so a `1024`-frame dropout crossing a one-second block
boundary or entering the final remainder cannot escape. It hashes every
complete consecutive `F`-frame block from `E_head`; hashes must be unique. It
also hashes and records the final non-empty shorter remainder with its exact
start and length. Persist minimum denominator, longest zero run and endpoints,
all complete-block hashes, and remainder metadata.

`Y05` uses natural stereo plus the duplicate, swap, common-negation, and
anti-phase constructions at each ratio. For natural stereo, persist candidate
and mapped-source values for the whole signal; bands `0..250`, `250..1500`,
and `1500..Nyquist Hz`; and four-second output windows at two-second hops.
Each mapped source window is `[floor(x(y_0)),ceil(x(y_1-1))+1)`, clamped to
`[0,L)`. Record left/right RMS, right-minus-left balance dB, dominance,
correlation, side/mid width, and `0..80 Hz` energy for both source and output,
plus their deltas. Omit a window only when both mapped source channels are
below `-60 dBFS` RMS or the final clipped output window is shorter than two
seconds; record the reason. Missing or non-finite required values reject.
Finite natural-stereo values are listening evidence, not new numeric vetoes.

## Runner, Deadlines, And Receipts

Track this exact profile in `.config/nextest.toml`:

```toml
[profile.audited-layered-cloud]
retries = 0
fail-fast = false
test-threads = 1
slow-timeout = { period = "120s", terminate-after = 1 }
failure-output = "immediate-final"
success-output = "never"
status-level = "all"
final-status-level = "all"

[[profile.audited-layered-cloud.overrides]]
filter = 'test(/^audited_layered_cloud_synthetic_/)'
slow-timeout = { period = "900s", terminate-after = 1 }

[[profile.audited-layered-cloud.overrides]]
filter = 'test(/^audited_layered_cloud_capture_/)'
slow-timeout = { period = "7200s", terminate-after = 1 }
```

Each structural and synthetic row is one explicit `#[test]` wrapper. Nextest,
not an in-test constant, kills a structural row after `120 s` and a synthetic
row after `900 s`. Capture rows get `7200 s`. Human listening has no machine
deadline; its compile-linked validator has `120 s`.

Each row catches assertion failure, writes one canonical receipt to a unique
temporary file, flushes and `sync_all`s it, atomically renames it to the row
path, syncs the parent directory, then returns pass or rethrows failure.
Timeout/process loss leaves the row missing, which the stage validator rejects.
No two processes append to one file.

Receipt schema `audited-layered-cloud/v1` has keys in this order:
`schema`, `checkpoint`, `tree`, `stage`, `round`, `owner`, `row_index`,
`row_id`, `status`, `render_count`, `output_frames`, `output_channels`,
`input_sha256`, `output_sha256`, `assertions`, `diagnostics`, `elapsed_ms`.
Render-shaped fields are arrays with exactly `render_count` entries;
`output_frames` always counts frames, never interleaved samples. Assertions are
ordered `{id,status}` objects naming every executed assertion. Diagnostics are
ordered `{id,value,unit,class}` objects, with `class` equal to `hard` or
`diagnostic`. Generic `frozen-owner` or unlabeled numeric arrays are invalid.

A static tracked manifest lists every expected receipt. A separate
compile-linked stage validator checks every file, SHA, assertion inventory,
diagnostic inventory, frame/channel count, row total, render total, status,
and checkpoint/tree identity, then writes and syncs the terminal summary.

From one clean isolated commit:

1. compile all `audited_layered_cloud` tests with profile
   `audited-layered-cloud`, release mode, and no execution
2. run construction alone; require `1/1`
3. run all `101` structural row tests with `-j 1`, no fail-fast, immediate
   failure output; run the structural receipt validator; require `101/101`
   rows and `51/51` renders
4. repeat steps 1-3 unchanged from the same commit/tree and require identical
   totals, outputs where repeated, capacities, and authority hashes
5. record candidate/evidence file hashes, all nine specs, manifest hash,
   `Cargo.lock`, `rustc -vV`, Effigy and nextest versions, OS, architecture,
   and the complete corrective-diff ledger
6. create the local acoustic ref before any synthetic row or inspectable
   candidate/comparator output exists

Any pre-checkpoint correction requiring a new DSP formula, scalar, source,
seed, helper algorithm, metric, threshold, assertion, comparator, or listening
choice stops for docs reassessment. Conformance fixes already answered here
may iterate and must be recorded.

After the ref, run `Y01..Y05` in order. Each owner command selects only its
row tests, then its receipt validator. Stop later owners on any failure.

## Comparator Capture And Concealed Listening

Use the five retained `44.1 kHz`, `220500`-frame sources. `SOURCE_SPEC` repeats
and verifies these complete-file SHA-256 values before decode:

| Family/file | SHA-256 |
| --- | --- |
| percussion `0000-drums_percussion-000002.wav` | `89e55b28c6ed36e26bf73f2024d301aaeedd07cca30b315f5530051c28f4e1e7` |
| bass `0004-bass-000236.wav` | `3d587007a5d9a683e82e14a184530a9a0f953e58fbc2fe3712a42aa86ecf9ad8` |
| vocals `0008-vocals-000010.wav` | `3d74a686dccf6dcdfedc57e0fc2b76a0d29374ba28afa1bb497172cc441f7ee9` |
| pads `0012-pads_sustains-000423.wav` | `a736a0e04ade9e879db954069c8c0b68842bd4d364eec69e62dfef1447763131` |
| full mix `0016-full_mix-000144.wav` | `caa5d0d7c51bc7e2d537d3d13dbe32055f0c2032c69bd8e3f28a38df96fafbf1` |

Capture exact `16x`, `32x`, and `100x` mono and original stereo: `30` rows.
Each row has Signal, PaulX, and Csound: `90` renders. Mono decode is samplewise
`f32((f64(L)+f64(R))/2)` before rendering.

Comparator authority is:

- PaulXStretch `v1.6.0`, commit
  `8ec191fdd7203354c79391cbc04c9fd83fa30ea0`, unmodified core, default
  processing, FFT control `16384`, driven by a capture-only wrapper
- Csound commit `0eaa07e3aee55f90e745f89294ddb52eec30345c`, `sndwarp` for mono and
  `sndwarpst` for stereo, pointer mode, unit resample, Hann table, window
  `round_half_up(F/10)`, overlap `15`, random-width
  `round_half_up(window/5)`, amplitude one

External trees and builds live only under the ignored evidence root. The
capture owner rejects dirty or wrong revisions, records recursive tracked-tree
hash, compiler/build commands, binary SHA-256, version output, wrapper source
SHA-256, command arguments, environment, exit status, source hash, decoded
pre-crop hash, crop count, and final hash. Wrapper plumbing may adapt file I/O
only; its source is compile-linked through `COMPARATOR_SPEC` and construction.
It cannot alter comparator DSP or normalize individual outputs.

Drive each comparator from source start to end over exact target duration,
then crop from the end to exactly `T`. Reject a missing, short, non-finite, or
wrong-rate/channel capture.

For each row, let the source and three rendered files have RMS `R_k` and peak
`P_k`. Reject `R_k=0`. Set the one shared target
`R*=min(R_source,min_{k in source+renders}(0.95*R_k/P_k))`, interpreting a
zero peak as invalid.
Scale every file by `R*/R_k`. This is the only listening-pack gain. Persist
pre/post RMS and peak, scale, and hashes.

Order the three labels by ascending SHA-256 of
`checkpoint || row_id || renderer_id || "ALC-CONCEAL-v1"`. Write the mapping
to a separate key file, hash and sync it, and expose only the labeled files and
blank decision schema. The pack validator proves `15` mono rows/`45` renders
and `15` stereo rows/`45` renders with unique labels, identical row targets,
exact lengths, finite samples, hashes, and complete metadata.

Mono review order is all `32x` and `100x` rows first, then `16x`. Each decision
records candidate usability; preferred/tied/lost against PaulX and Csound;
cloud usefulness, source identity, tonal focus, motion, grain density,
periodicity, transient spread, low-frequency noise, event order, level, entry,
tail; and named defects. The validator opens the key only after all `15`
decisions are present and immutable.

Mono passes only when every candidate row is usable, at least `8/10` primary
rows are preferred or tied against at least one comparator, no family loses
both primary ratios, and no row has a click, dropout, static freeze, obvious
periodic flutter, metallic replica, arbitrary level step, reversed event
order, or unusable entry/tail behavior.

Original-stereo review opens only after mono passes. It uses the same `15`
rows and fields plus centre stability, one-sided pull, related motion, width,
low-frequency balance, and image jumps. The operator may reject in speaker
pre-screen. Promotion requires a different eligible listener with functional
linked-stereo hearing; the admitted Dream waiver does not transfer. Every row
must be usable, at least `8/10` primary rows must be preferred or tied against
one comparator, and no family may lose both primary ratios. Missing or
ambiguous independent review blocks promotion.

Human listening is promotion authority. Metrics diagnose or reject missing
evidence; they cannot claim musical parity.

## Isolation, Rejection, Cleanup, And Pass Surface

Batch 31.73 may create only:

- worktree `/Users/tom/Dev/projects/signal-candidate-31-73`
- branch `candidate/g10-031-audited-layered-cloud`
- private module
  `crates/signal-dsp-stretch/src/creative_audited_layered_cloud/`
- tracked static evidence manifest and `.config/nextest.toml` profile
- local ref `refs/signal-evidence/creative/audited-layered-cloud/31-73-acoustic`
- ignored outputs under
  `target/creative-stretch-audited-layered-cloud-31-73/`

Existing Signal dependencies only. No public API, production renderer, report
binary, reusable fixture, route, cache, artifact, dynamic ratio, product
control, Loophole, or Chorus change is allowed.

Any structural, synthetic, mono, or stereo miss rejects the immutable
checkpoint. A second incomplete-evidence result also rejects and closes the
Cloud family. Record the dominant cause, stopped gate, complete persisted
receipts, and exact identity. Do not tune, repair, rerun, or rebind after
acoustic identity.

After the required docs-only decision, delete a rejected worktree, branch,
build state, private source, tests/config, generated audio, comparator builds,
and pack. Retain the local evidence ref only through that reassessment, then
delete it. Nothing from a rejection enters `main`.

A complete pass opens a separate minimal-admission batch for only the private
fixed-ratio renderer, request/error boundary, structural/synthetic regression
owners, diagnostic receipt schema, and one internal engine version. Comparator
tools, listening packs, seed exposure, routing, controls, cache, artifacts,
dynamic ratio, overlaps, and cross-repo work remain out of scope.

## Why This Is Plausible And What Remains Risky

The renderer directly targets the requested Cloud region: many long,
overlapping unit-rate views follow one source pointer instead of phase-locking
or repeating a short waveform cycle. Variable duration breaks a single audible
grain period; shared validity normalization removes arbitrary overlap gain;
the one map preserves event order; and linked weights preserve channel
relationships. The asymmetric admitted envelope gives a direct entry and
longer musical release without material-dependent heuristics.

Its unresolved risks are audible, not documentary: low-frequency build-up,
metallic grain replicas, excessive transient spread, static or periodic motion,
loss of tonal identity at `100x`, comparator capture cost, and natural-stereo
image drift despite exact algebra. The gate order exposes those risks without
letting a metric promote the result.

## Sources

- [Closed LayeredCloud brief and Batch 31.71 gap ledger](./offline-creative-layered-cloud-brief.md)
- [DirectRenewalDream boundary and source authority](./offline-creative-direct-renewal-dream-brief.md)
- [Creative source triangulation](../research/specimen-dossiers/creative-stretch-source-triangulation.md)
- [Csound `sndwarpst` manual](https://csound.com/manual/opcodes/sndwarpst/)
- [Pinned Csound implementation](https://github.com/csound/csound/blob/0eaa07e3aee55f90e745f89294ddb52eec30345c/Opcodes/sndwarp.c)
- [SuperCollider `Warp1` manual](https://docs.supercollider.online/Classes/Warp1.html)
- [Pinned SuperCollider implementation](https://github.com/supercollider/supercollider/blob/2f0803bcd2e551564e3fef8d5075816cbb685cd4/server/plugins/GrainUGens.cpp)

## Next Task

Run Batch 31.73 only. Create the exact fresh isolated worktree and implement
this brief from canonical docs. Complete compile, construction, and two clean
structural rounds before creating the acoustic ref. Stop for docs reassessment
on any unanswered authority choice. Do not recover Batch 31.70 source or
output, run acoustic work before the ref, change admitted renderers or product
surfaces, touch Loophole or Chorus, or push.
