# Offline Automatic ExactTargetTransparentDreamRouter Brief

Status: rejected and closed for current owners
Owner: dsp
Updated: 2026-07-25
Contract: `085`
Roadmap: `g10.035`, Batches 35.2-35.6

## Decision

Build one private fixed-ratio `ExactTargetTransparentDreamRouter`.

Let `N` be source frames and `T` the exact target frames. The route owns:

| Exact target | Owner |
| --- | --- |
| `ceil(N/2)..=4N` | exact-target Transparent |
| `4N..=8N` | one render-wide Transparent/Dream transition |
| `8N..=16N` | neutral Dream |

The endpoint ties belong to the pure owners. `T=4N` is byte-exact
Transparent. `T=8N` is byte-exact admitted Dream with `space=0.5` and
`ADMISSION_SEED=0x0123456789abcdef`.

The route is not a new stretch renderer. It keeps the two admitted acoustic
owners, evaluates both against one exact target and linear source map in the
interior band, then combines sample-aligned output once. It adds no transient
detector, phase law, recurrence, limiter, resampler, post-fade, or loudness
stage.

The candidate identity is:

`signal-exact-target-transparent-dream-router-v1`

`Automatic` remains private and unavailable to consumers until a later public
decision. Explicit Transparent, Dream, and Cyclic remain unchanged. Cyclic
never enters this route.

## Checkpoint Result And Gate Correction

Batch 35.3 freezes acoustic checkpoint
`50c3d028ae1d5b0d057e74899b84a1a27c0e0038`, tree
`0ff62f572eef222d38ac356d3874c973d78ba2d2`. Normal-profile stretch
regression passes `204/204`. Two release conformance rounds from unchanged
bytes pass construction `1/1` and structural `8/8`.

The first acoustic owner stops on the pure Transparent
`rademacher-noise` row at `N=96000`, `T=383999=4N-1`. Dispatch and output are
byte-exact Transparent. Its peak is `10.370356`, above the brief's universal
`8.0` ceiling. Byte parity and the ceiling cannot both hold for the admitted
owner. No pitch owner, later synthetic owner, long-form render, or listening
gate ran.

Batch 35.4 therefore classifies the checkpoint as evidence-invalid, not a
renderer pass or rejection. The corrected ownership is:

- pure Transparent rows require byte parity, exact length, finiteness,
  determinism, complete hashes, and the admitted Transparent integrity rules;
  they gain no route-specific absolute peak ceiling
- pure Dream rows require byte parity and the admitted Dream integrity rules,
  including Dream's existing absolute peak ceiling
- interior rows reject route-created overshoot above the larger
  sample-aligned arm peak by more than two `f32` ulps

This correction changes no owner output, route equation, map, weight, source,
seed, comparator, listening policy, or production surface. It authorizes one
exact source replay under new isolation identity. It does not authorize
candidate tuning or a second renderer.

## Exact Replay Result

Batch 35.5 freezes acoustic checkpoint
`db2a02d35f39a035e44803d0cc26861dcebe2534`, tree
`ab8bf005fe8fe72522e3edc23b617d2ac37b5cd8`, at
`refs/signal-evidence/creative/exact-target-transparent-dream-router/35-5-acoustic`.

Compile, two unchanged construction `1/1` and structural `8/8` rounds, and
non-acoustic regression `204/204` pass. The corrected identity/parity owner
passes all `150` rows. The formerly contradictory pure Transparent row
therefore clears under the corrected authority.

Pitch then rejects the checkpoint on low tone at `N=96000`,
`T=576000=6N`, `110 Hz`:

| Contribution | Error |
| --- | ---: |
| Transparent | `0.16404282837539305` cents |
| Dream | `6.277316077755877` cents |
| Automatic | `8.717736874188192` cents |

Automatic is `2.440420796432315` cents worse than the worse aligned arm. The
terminal allowance is `1` cent. No later synthetic owner, long-form render,
mono listening, or linked-stereo review runs.

This is valid renderer rejection. The fixed render-wide samplewise
linear-amplitude blend does not preserve the owners' tonal result at this
interior control. Do not change weight, interval, phase, alignment, gain,
window, threshold, estimator, or source around this checkpoint.

## Architecture Reassessment

Batch 35.6 rejects this route at product authority and closes Automatic for
the current owners.

| Apparent next mechanism | Decision |
| --- | --- |
| hard Transparent/Dream switch | not seamless; exposes a character and boundary discontinuity |
| another weight, interval, gain, alignment, or correlation law | repair of the rejected samplewise blend |
| time, frequency, or material masks between rendered owners | adds selection seams and reopens prohibited hybrid routing |
| one coherent complex or component synthesis field | a new complete renderer; reopens closed successor or component programs |
| diagnostic pitch or wider allowance | explicit product-gate change, not architecture evidence |

Rubber Band, Signalsmith, and Bungee support one internally coherent phase
field, not crossfading two completed waveforms. PaulXStretch supports Dream's
separate magnitude-renewal owner, not a coherent handoff to Transparent.
Retained evidence therefore supplies no unused build-ready route between the
admitted owners.

The product result is explicit mode ownership:

- Transparent remains the source-readable general stretch
- Dream remains the smooth creative `4N..=16N` owner
- Cyclic remains the commanded `2N..=8N` effect
- Automatic has no engine, API, discovery, identity, or cache semantics

A consumer may suggest one explicit mode from context. It must not hide a hard
switch behind Automatic or claim seamless behavior.

Reopening requires one source-backed complete owner spanning the transition
with one synthesis state, one map, linked stereo, exact length, bounded state,
and complete evidence, or an explicit operator change to the seamless or
terminal-gate product boundary.

## Immutable Inputs

The candidate starts from the Batch 35.2 closeout commit and records its commit
and tree before worktree creation. These production source hashes are frozen:

| File | SHA-256 |
| --- | --- |
| `signal-dsp-stretch/src/lib.rs` | `958901dff2540d14c7d1e8c0d063ec0dd36e7567b496c2a29b93f36ccc41cfcb` |
| `signal-dsp-stretch/src/phase_vocoder.rs` | `d02ed78c085eb3ff9b9e2299da935e7a422eb4601c1559b2117825e350205221` |
| `signal-dsp-stretch/src/creative.rs` | `20837fdd411f136447fb6e215ddee9595df5593b24362de3363b6638ebcf0515` |
| Dream `plan.rs` | `d5629ddfcce0052cbabb386701f54f98aa36e700ebce35be810cdb4a6f3b4e70` |
| Dream `analysis.rs` | `71a1ae1815078fd5916ac28547a961ce4ed6fadbd978435de9f18033945713ac` |
| Dream `stereo.rs` | `e159cdab1b674dd97e00658d62902719b4100e26744db57da914e2eb20c94ee3` |
| Dream `synthesis.rs` | `ca0e267617cfb49634f79d630bbded7c5570e5db2e7a13d3014c346afefdfe54` |
| Dream `mod.rs` | `0cc95ecc0b2b798c58eaf6208c350b655a7edba3fee8d58e1f6b574382eaf6d6` |
| `Cargo.lock` | `e3848a40d2ea1ff88a0e036df40d1fefa56c7aca950a95262c1d8c5668fd394d` |

Dream `plan.rs`, `analysis.rs`, `stereo.rs`, `synthesis.rs`, and `mod.rs`
remain byte-identical. The existing Transparent transform, selector
measurements, thresholds, windows, hops, phase behavior, normalization, and
crop equations remain byte-identical in result.

Permitted candidate changes are limited to:

- one private route module and private request/error types
- one private exact-target Transparent entry
- buffer ownership changes required by the frozen memory schedule
- candidate-only tests, receipt writers, and nextest profile

The exact-target entry may extract or parameterize existing code. It may not
change a rendered sample. Public types, existing dispatch, cache and artifact
identity, runtime, and other crates remain unchanged.

The exact replay must restore and prove these Batch 35.3 checkpoint hashes
before conformance:

| Surface | SHA-256 |
| --- | --- |
| `signal-dsp-stretch/src/lib.rs` | `3354ee7433f9a9c3e115f2b8bfff69e19f589abecad3818611cce03d465e1549` |
| `signal-dsp-stretch/src/phase_vocoder.rs` | `60b599c2d15f6a1d87ab54b3567fc134edbb9faa08be0246158979a2f09fb920` |
| route module | `d7629a4c6dcf7f9ffecbef232d4bd1fe5b2b5d7a9e1bd7d4d279fab31a1f3a02` |
| candidate nextest profile | `504cfa9780d0ee1de54d258a2de21f87b0be8eb6f42c5bc5635cf0e1e2e0efff` |
| conformance ledger | `a0b4fef20f4f7a710d65f70b376a9ca9be0c31cb8c369ca67749cb7599c392b1` |
| source manifest | `fc667570cad5aba4b23dc51973a5fed34c548f1b3e389d15c13c52284ab62ff3` |

Restore only candidate source, test, profile, conformance, and manifest bytes
from checkpoint `50c3d028`. Keep current canonical docs from `main`. A hash
miss, missing object, or required reconstruction by judgment stops for
reassessment.

## Exact Request

The private request contains:

- borrowed finite interleaved `f32` input
- `channels` equal to `1` or `2`
- sample rate `8000..=192000`
- authoritative `target_frames: usize`

There is no ratio field, character, `space`, cycle, seed, route weight, pitch,
or dynamic curve.

Validation order is fixed:

1. reject channels other than `1` or `2`
2. reject a sample rate outside `8000..=192000`
3. reject a partial interleaved frame
4. reject any non-finite input sample
5. compute `N=input.len()/channels`
6. reject `N` or `T` above `2^53-1`
7. accept `N=0,T=0` as empty success
8. reject `N=0,T>0` and `N>0,T=0`
9. require `2T>=N` with checked `u128` arithmetic
10. require `T<=16N` with checked `u128` arithmetic
11. require checked `T*channels`

Values are rejected before output allocation. They are never clamped. The
lower comparison is exactly `T>=ceil(N/2)`; no rounded ratio participates.

After validation:

- `T=N`: return the input frame region byte-exact
- `T<=4N`: call exact-target Transparent only
- `4N<T<8N`: render Transparent, render neutral Dream, then blend once
- `T>=8N`: call neutral Dream only

Branch comparisons use checked integers widened to `u128`. Floating point
never selects an owner.

## Exact-Target Transparent Owner

The private entry takes `input`, `channels`, and exact `T`. It derives
`r=T/N` only after integer dispatch.

Direction owns the promoted production path:

- `T<N`: `CompressionShortWindowSelector`
- `T=N`: byte-exact passthrough
- `T>N`: `ExpansionShortWindowSelector`

The selector first evaluates the current `2048/512` output and its unchanged
gate. If selected, it evaluates the current `1024/256` output. Mono and linked
mid/side paths keep their existing equations and thresholds. Short inputs keep
the existing linear fallback. No ratio-rounding constructor is used.

Exact target is passed directly to the existing kernel. The entry must equal:

`OfflineHighQualityStretcher::with_path(T/N, selected_path)`

for every case where the public ratio constructor rounds back to `T`.
Candidate parity also covers exact targets where binary ratio construction
would otherwise be ambiguous.

Buffer mechanics change without changing arithmetic:

- crop the phase-vocoder output in place; do not allocate a third cropped copy
- use one output-sized normalization/contribution allocation
- render the default result into the returned destination
- when a short window is selected, reuse the contribution allocation and
  overwrite the destination only after the gate decision
- keep reduction order and final `f32` stores unchanged

Any byte-parity miss stops the candidate before acoustic evidence. It is not
repairable by a tolerance, gain, fade, alignment shift, or changed selector.

## One Source/Output Map

The only continuous map uses frame-boundary coordinates:

`m(q)=qN/T`, for `0<=q<=T`

It is monotonic, maps `0` to `0`, maps `T` to `N`, and is represented with
checked integer products before one `f64` division.

Dream evaluates this map at output sample centres:

`x_j=m(y_j+0.5)-0.5`

with `y_j=jH`, exactly as admitted.

Transparent analysis launch `j` remains source-anchored at `a_j=jA`.
Its ideal output launch is `q_j=a_jT/N`; the existing synthesis launch is the
nearest integer output frame. The displacement is at most half an output
frame, is recomputed from `j` rather than accumulated, and is identical for
all linked channels. Centred zero padding and the existing crop place source
and output origin on the same map.

Both owners therefore use one linear source order and exact `[0,T)` output
lattice. Representation-specific window centres do not create another cursor.
No shift search, adaptive cursor, segment concatenation, post-resampling, or
second event timeline is permitted.

Structural proof must show:

- exact rational endpoints
- strict source-anchor increase
- Transparent launch error at most `0.5` output frame
- Dream source centres strictly increase
- identical map and launch sequence across linked channels
- no accumulated floating cursor in either owner

Failure closes this brief before acoustic execution.

## Transition Weight

Only `4N<T<8N` computes a weight. Define:

`u=log2(T/(4N))`

`w=u*u*(3-2u)`

`w` is the Dream amplitude contribution. Transparent contributes `1-w`.

Arithmetic is fixed:

1. convert exact `T` and exact checked `4N` individually to `f64`
2. divide once
3. call `f64::log2` once
4. evaluate multiplications left to right as written, without `mul_add`
5. require finite `0<w<1`

The candidate platform/toolchain identity owns the resulting bits. Both
channels and every output frame use the one render-wide `w`. These reference
vectors must pass within `2e-15`:

| `T/N` | `u` | `w` |
| ---: | ---: | ---: |
| `4` | `0` | `0` |
| `4.5` | `0.16992500144231237` | `0.07681051735897700` |
| `5` | `0.32192809488736235` | `0.24418532130324860` |
| `6` | `0.58496250072115619` | `0.62621712595841772` |
| `7` | `0.80735492205760406` | `0.90296255866681796` |
| `7.5` | `0.90689059560851848` | `0.97560631455458235` |
| `8` | `1` | `1` |

The endpoint rows are oracle rows only. Dispatch bypasses the blend there,
which guarantees byte-exact owner output independent of logarithm behavior.

Changing the interval, logarithm, polynomial, arithmetic order, or blend
meaning changes route identity.

## Level And Correlation Law

For each frame and channel in the interior:

`z=f32((1-w)*f64(transparent)+w*f64(dream))`

This is the complete level law.

Linear-amplitude weights are intentional:

- identical contributions remain identical
- each mixed sample lies in the two-arm convex hull, apart from final `f32`
  rounding
- the route cannot create a peak above both aligned contributions
- no measured correlation changes gain

Decorrelated arms can lose up to about `3 dB` near equal contribution.
Anti-correlated content can lose more. The route neither hides nor repairs
that risk. Synthetic level/correlation controls and concealed listening reject
an audible dip, comb, phase cancellation, or arbitrary energy redistribution.

There is no equal-power boost, correlation estimator, adaptive normalization,
loudness matching inside the renderer, limiter, compressor, make-up gain,
per-band gain, or post-render correction.

## Analysis, Event, Tonal, And Phase Ownership

Transparent keeps its admitted centred Hann STFT, tracked peaks, identity phase
locking, expansion transient resets, and promoted selectors. Dream keeps its
long-window native-channel magnitude view and counter-addressed phase renewal.

The router adds no material classifier. Simultaneous representation in the
interior is exactly the two complete admitted contributions evaluated at the
same `T` and map, with one fixed weight.

Transient ownership:

- Transparent keeps detection, reset, phase treatment, and replica behavior
- Dream keeps deliberate long-window smear with no event reset
- the router performs no event detection, reassignment, copy, or launch

Tonal ownership:

- Transparent keeps peak tracking and propagated phase
- Dream keeps current-frame magnitude with deterministic renewal
- the router has no peak, dormant-bin, reactivation, or phase state

Replica prevention comes from the owners' unchanged behavior, one monotonic
map, one visit to each contribution, and one samplewise mix. An audible
doubled attack, micro-echo, stutter, metallic repetition, or new ring rejects
the whole route. It does not authorize a transient or tonal patch.

## Linked Channels

All route decisions are channel-shared:

- one exact request and map
- one Transparent selector decision derived from the linked downmix
- one Transparent mid/side schedule
- one Dream native-channel schedule, seed field, and symmetric space law
- one transition weight
- one samplewise blend law
- one boundary and crop lattice

Dream always uses `space=0.5` and
`ADMISSION_SEED=0x0123456789abcdef`. The route exposes neither.

Mono input remains mono. Stereo output is interleaved in source channel order.
Duplicate, common-negated, anti-phase, channel-swap, delayed-pad, and
source-balanced fixtures remain mandatory. Independent per-channel routing,
weight, normalization, seed, map, selector, or random trajectory rejects.

## Boundaries And Exact Length

Each pure owner keeps its admitted exterior-zero and boundary behavior.
Interior contributions each emit exactly `[0,T)` before blending. Index `y`
from each contribution blends only with index `y`; no offset or delay search
is allowed.

Transparent keeps centred prefix/suffix padding, normalized overlap-add, and
exact crop. Dream keeps its admitted sine entry, longer terminal release, and
exact-zero first and last samples. The router adds no envelope.

Consequences are explicit:

- at `4N`, boundary output is byte-exact Transparent
- immediately above `4N`, Dream boundary character enters only by `w`
- immediately below `8N`, Transparent boundary character remains only by
  `1-w`
- at and above `8N`, output is byte-exact Dream
- exterior samples are positive zero
- final output contains exactly `T*channels` samples

Any click, edge energy step, premature fade, chopped release, or arbitrary
entry/tail redistribution in transition listening rejects.

## State, Memory, Determinism, And Cost

Allocation happens before processing. Render order is fixed:

1. allocate the final output
2. render Transparent into it
3. release Transparent duration-scaled scratch
4. allocate/render Dream as the sole output-sized contribution
5. blend into final and release Dream

Pure zones render only their owner. The exact memory ceiling, excluding
borrowed input, is:

`T*channels*sizeof(f32)` final output

plus:

`T*channels*sizeof(f32)` one contribution/normalization allocation

plus duration-independent owner state capped by existing owner limits. Dream
retains its `32 MiB` working-state ceiling. Transparent FFT, phase, peak, and
selector state is bounded by the admitted maximum window and does not scale
with duration after the normalization buffer is counted.

No third output-sized crop, default, short-window, mid/side, or blend buffer
may coexist. Instrumented peak allocation must prove the formula at mono and
stereo, short/default selector outcomes, `4N+1`, `6N`, and `8N-1`.

Execution is single-threaded with fixed traversal and reduction order. Same
request, source bytes, candidate identity, platform, and toolchain must produce
byte-identical output. Cost is the sum of the two unchanged owners in the
interior and one owner outside it. No audio-thread execution, I/O, lock,
source fill, callback state, or unbounded retry exists.

## Candidate Isolation

Batch 35.5 creates exactly:

- worktree: `/Users/tom/Dev/projects/signal-candidate-35-5`
- branch:
  `candidate/g10-035-exact-target-transparent-dream-router-replay`
- tracked authority:
  `candidate-evidence/g10-035/35-5/conformance.tsv`
- tracked source manifest:
  `candidate-evidence/g10-035/35-5/source-manifest.tsv`
- candidate-only nextest profile: `automatic-route`
- ignored evidence root:
  `target/automatic-stretch-route-35-5/`
- acoustic ref:
  `refs/signal-evidence/creative/exact-target-transparent-dream-router/35-5-acoustic`

Before creation, require the new worktree, branch, tracked evidence paths,
ignored root, and ref to be absent. Require the closed Batch 35.3 worktree,
branch, ref, and generated state to be absent. Record current `main`, restored
checkpoint/tree, every hash above, toolchain, OS, architecture, Effigy, and
nextest versions. Unknown or partial state stops; it is never overwritten or
reused.

No historical rejected renderer, route, test, receipt, threshold, or generated
audio may be recovered.

## Conformance Gate

Candidate tests use exact prefixes:

- `exact_target_transparent_dream_construction_`
- `exact_target_transparent_dream_structural_`
- `exact_target_transparent_dream_acoustic_`

Before any inspectable candidate audio:

1. compile the complete test binary
2. run construction and structural owners from one clean commit
3. repeat both unchanged
4. require equal row counts, normalized receipts, and hashes
5. freeze the acoustic ref directly at that commit

Construction owns source hashes, allowed files, forbidden surfaces, owner
inventory, request/error inventory, row counts, commands, and cleanup paths.

Structural owners and hard pass conditions are:

| Owner | Cases | Pass |
| --- | --- | --- |
| request domain | `N=0`; `T=0`; `N=1..64`; lower/upper neighbours; exact-integer and allocation overflow | integer oracle equality; rejection before allocation |
| dispatch | `N/2`, `N`, `4N-1`, `4N`, `4N+1`, `5N`, `6N`, `7N`, `8N-1`, `8N`, `8N+1`, `16N` | exact owner table; identity/input equality |
| map | all accepted dispatch targets | exact endpoints, strict monotonicity, launch displacement bound, linked equality |
| weight | endpoint, vector table, one-frame neighbours, arbitrary interior coprimes | finite monotonic `w`; vectors within `2e-15`; no float owner decision |
| Transparent parity | mono/stereo; short/default selector; short input; boundaries; all pure targets plus overlap probes | byte-exact current owner |
| Dream parity | mono/stereo at `8N`, `8N+1`, `10N`, `16N`; neutral defaults | byte-exact admitted owner |
| blend oracle | zero, constant, equal, opposite, impulse, alternating arms at all vector weights | exact frozen formula; convex-hull bound within two `f32` ulps |
| boundary/crop | `N=1,H-1,H,H+1,2H`; non-hop `T`; `4N/8N` neighbours | exact length, finite, owned fill, exterior zero |
| linked algebra | duplicate, negated, anti-phase, swap | one decision/map/weight; relation rows complete |
| memory/determinism | mono/stereo, both selector outcomes, pure/interior targets, two durations | byte repeat; no third output-sized allocation; fixed state ceiling |
| immutable surface | every frozen hash and permitted diff | exact match; no public or cross-repo change |

Any miss stops before acoustic identity. A compiler, visibility, type,
allocation, or receipt defect may be repaired only when this brief already
fixes the answer. A changed acoustic equation, selector, weight, threshold,
source, seed, comparator, assertion meaning, or gate requires docs-level
reassessment.

## Synthetic Gate

Use `F=48000`, `N=96000`, the retained Transparent synthetic sources, and the
admitted Dream mono/stereo sources. Targets are:

- pure controls: `N/2`, `N`, `2N`, `4N-1`, `4N`, `8N`, `8N+1`, `16N`
- boundary controls: `4N-1`, `4N`, `4N+1`, `8N-1`, `8N`, `8N+1`
- interior controls: `9N/2`, `5N`, `6N`, `7N`, `15N/2`

Every output must have exact length, finite samples, complete hashes, no
unowned `H`-frame zero run inside mapped active support, and byte-identical
repeat. Pure Transparent rows have no new absolute peak ceiling and must remain
byte-exact. Pure Dream rows retain Dream's admitted absolute peak ceiling.
Interior rows use the aligned-arm peak comparison below.

The gate is terminal on:

- pure-owner parity mismatch
- route peak above both aligned arm peaks by more than two `f32` ulps
- whole-render or three-band route energy more than `3.25 dB` below the
  uncorrelated arm-power prediction
- more than `1 dB` whole-render or three-band level step across either
  one-frame boundary trio
- pitch error worse than the worse aligned arm by more than `1 cent`
- event-centroid error outside the closed interval formed by the two arm
  centroids plus one output frame
- impulse-region or replica-region count greater than the worse arm
- transient crest above the larger aligned arm crest by more than `0.25 dB`
- unsupported-bin tonal energy or short-time tonal movement worse than the
  worse arm by more than `0.5 dB`
- uniform-noise autocorrelation or block-RMS CV greater than the worse arm by
  more than `0.05`
- new click, dropout, non-finite metric, or incomplete metric
- whole or three-band stereo balance error above `0.75 dB`
- balance spread above `0.50 dB`
- dominance reversal where source balance magnitude is at least `0.50 dB`
- channel-specific route decision, weight, map, or random field

Opposite-arm cancellation is reported separately. Any row exceeding the
`3.25 dB` limit rejects instead of opening adaptive gain.

Run owners in fixed order: identity/parity, pitch, event/replica, crest,
tonal/noise, level/boundary, linked stereo. Stop at the first failed owner.
Metrics reject integrity; they do not promote musical continuity.

## Long-Form Mono Gate

Only after synthetic passage, use the retained five exact `44.1 kHz`,
`220500`-frame sources and hashes:

- drums/percussion
- bass
- vocals
- pads/sustains
- full mix

Render Automatic at:

`4N-1`, `4N`, `4N+1`, `5N`, `6N`, `7N`, `8N-1`, `8N`, `8N+1`

This is `45` candidate renders. Prepare exact-target Transparent references
for every target through `8N` and admitted neutral Dream references for every
target from `4N`. Pure and endpoint parity is checked before concealment.

For `4N+1`, `5N`, `6N`, `7N`, and `8N-1`, create concealed, level-matched
Automatic/Transparent/Dream trios. For the two three-target boundary
neighbourhoods, create concealed continuity sequences with identical excerpt
positions. Level matching is audition-only and never modifies candidate
receipts.

Review all five families at `6N`, then `5N` and `7N`, then both boundary
neighbourhoods. Record:

- source readability and Dream smoothness
- combing, phasing, ringing, grain, and tonal focus
- attack definition, doubled attack, micro-echo, and stutter
- level and energy distribution
- event placement
- entry and tail behavior
- usefulness and preference

Pass requires:

- no unusable Automatic row
- Automatic preferred or tied against at least one explicit owner on at least
  `21/25` interior family/target rows
- no family losing all five interior targets
- no audible discontinuity in either boundary sequence
- no material loss of Transparent readability near `4N`
- no material loss of Dream smoothness near `8N`
- no audible comb, phase cancellation, doubled attack, micro-echo, replica,
  arbitrary level dip, image-like mono flutter, click, or chopped boundary

Listening is the promotion authority. Objective passage alone cannot pass the
candidate.

## Linked-Stereo Gate

Only after mono passage, render the five original stereo sources at:

`4N+1`, `5N`, `6N`, `7N`, `8N-1`

This is `25` candidate renders plus both explicit-owner references. Reuse the
synthetic hard stereo controls and report whole-render correlation, width,
balance, three-band balance, low-frequency balance, mapped-window dominance,
entry energy, and tail energy.

Concealed review rejects centre loss, image pull, balance shift, width jump,
unrelated channel motion, low-frequency pull, boundary image change, or a
stereo result less useful than both owners.

Default promotion requires an eligible independent listener. The Dream and
Cyclic waivers do not transfer. After all hard controls pass, the operator may
make one new Contract `085` checkpoint-scoped product decision; absent that
explicit decision or eligible review, promotion remains blocked.

Pass requires no unusable row, preferred or tied against at least one owner on
at least `21/25` rows, and no family losing every target.

## Stop, Rejection, Cleanup, And Admission

Stop before implementation if:

- exact-target Transparent cannot remain byte-exact
- the two owners cannot satisfy the one-map and one-lattice structural rows
- the two-buffer memory schedule cannot be met without public, runtime, cache,
  artifact, or cross-repo change

After acoustic identity, any hard objective or listening miss rejects the
checkpoint. Do not tune, repair, or rerun it. Record the dominant cause, first
failed gate, completed rows, exact commit/tree/ref, and comparator identities.

A contradiction between two mandatory gates invalidates evidence rather than
proving renderer quality. Stop immediately, run one docs-only ownership
reassessment, and change no renderer or acoustic evidence bytes. One exact
replay is allowed only when no renderer-quality or listening judgment occurred,
the correction removes the contradiction without selecting a new threshold,
the original identity closes, and the replay uses a fresh isolation identity
with hash-proved source.

On rejection:

- delete the candidate worktree and branch
- delete candidate test/config code, tracked ledgers, manifests, ignored
  receipts, renders, and build state
- retain the acoustic ref through one docs-only reassessment, then delete it
- admit nothing to `main`

Two complete route candidates failing for the same dominant cause close this
route shape for architectural reassessment. No weight, interval, fade, gain,
alignment, selector, window, threshold, or scalar sweep follows.

Only a complete pass opens a separate admission batch. The maximum admissible
production surface is:

- one private `exact_target_transparent_dream_router` module
- one private exact-target Transparent entry and byte-preserving buffer
  ownership refactor
- the frozen `signal-exact-target-transparent-dream-router-v1` identity
- focused request, parity, boundary, blend, memory, determinism, and linked
  stereo regression tests

Candidate evidence runners, nextest profile, sources, comparators, manifests,
receipts, audio, and acoustic ref stay out of `main`.

Public `Automatic`, cache and artifact identity, dynamic ratio, pitch,
RealtimePreview, runtime, UI, Loophole, and Chorus remain unadmitted.

## Readiness

The brief fixes every request, owner, map, schedule, weight, blend, boundary,
stereo, identity, memory, evidence, rejection, cleanup, and admission choice.
Batch 35.3 is evidence-invalid and closed. Batch 35.4 corrects gate ownership
without changing the candidate. Batch 35.5 completes one exact replay and
rejects this route at synthetic pitch. Batch 35.6 closes Automatic for the
current owners. Batch 35.8 confirms every worktree, branch, acoustic ref,
candidate source, nextest profile, tracked evidence path, ignored evidence
root, generated asset, and build state is absent. No implementation or
admission batch is ready.

## Next Task

No Automatic task is ready. Preserve this rejected brief as historical
evidence. Reopen only under the complete-owner or explicit product-boundary
conditions above.
