# 035 - Creative Stretch Product Coverage And Routing Audit

Status: active; Batch 35.6 complete; Batch 35.8 ready
Owner: core-product
Created: 2026-07-25
Depends on: `g10.030`, `g10.033`, `g10.034`
Governing contracts: `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`,
`docs/contracts/085-creative-time-stretch-product-and-routing-contract.md`
Vision tags: `DSP`, `STRETCH`, `CREATIVE`, `ROUTING`

## Problem

Signal has three useful stretch intents:

- Transparent through the frozen `OfflineHighQuality` renderer
- smooth neutral Dream over every exact target `4N..=16N`
- commanded Cyclic repetition over every exact target `2N..=8N`

The operator wants clean range-dependent algorithm selection, but the admitted
owners are not interchangeable quality tiers. Dream and Cyclic overlap while
expressing different musical intent. Transparent and Dream use different
analysis, scheduling, boundary, and control laws. A ratio switch or output
crossfade is not automatically seamless.

## Generation Runway

This lane advances the `g10` stretch runway from separately admitted owners to
one bounded product-routing decision.

The visible runway is:

1. coverage, intent, and compatibility audit
2. one complete automatic-route brief
3. one isolated fixed-ratio route candidate
4. gate-ownership reassessment and one exact replay if evidence is invalid
5. public API decision
6. minimal public admission only after objective and listening passage
7. docs-only closeout

Batch 35.6 is complete. Batch 35.8 is the only ready work. Public Automatic
implementation is cancelled.

## Goals

- [x] publish the exact current Transparent, Dream, and Cyclic ownership matrix
- [x] decide whether one automatic range intent is warranted
- [x] preserve explicit Transparent, Dream, and Cyclic choices
- [x] keep Cyclic outside automatic selection
- [x] freeze one automatic target envelope and transition interval for a
  complete brief
- [x] freeze exact route mechanics, identity, bounds, and evidence
- [x] reject one isolated fixed-ratio route on valid synthetic evidence
- [x] keep public API and product integration blocked after acoustic rejection

## Non-Goals

- no change to any admitted renderer
- no automatic Cyclic selection or Dream/Cyclic blend
- no `Spectral`, `Rough`, `Cloud`, or ratio above `16x`
- no dynamic ratio, character automation, pitch, reverse, or RealtimePreview
- no cache, artifact, runtime DTO, UI, Loophole, or Chorus work
- no external production dependency
- no candidate DSP, harness, fixture, report, or evidence asset on `main`

## Tested Product Shape

Batch 35.1 selected `Automatic` as a candidate opt-in intent, not a replacement
for explicit modes. It never became a public API.

Let `N` be source frames and `T` the exact requested output frames. The tested
envelope was `ceil(N/2) <= T <= 16N`, compared with checked integer
arithmetic:

| Target | Automatic owner |
| --- | --- |
| `0.5N..=4N` | frozen Transparent owner |
| `4N..=8N` | one fixed-ratio Transparent/Dream transition |
| `8N..=16N` | admitted neutral Dream owner |

At `4N`, the candidate was byte-exact Transparent. At `8N`, it was byte-exact
Dream with admitted neutral defaults. Interior targets used one
channel-shared, render-wide weight frozen by Batch 35.2.

Explicit modes remain:

- Transparent: unchanged `OfflineHighQuality` surface and controls
- Dream: public v4 `4N..=16N`, `space 0..=1`
- Cyclic: public v4 `2N..=8N`, cycle `5..90 ms`

Cyclic never entered Automatic. Its repetition and cycle duration remain
explicit musical choices. The rejected candidate exposed only exact duration;
its Dream contribution used neutral `space=0.5` and the admitted fixed seed.
Users select Transparent, Dream, or Cyclic directly.

## Execution Plan

## Batch 35.1 - Coverage And Routing Audit

Status: complete

Documentation only.

- [x] distinguish callable geometry from acoustic and product admission
- [x] audit exact target, map, scheduler, boundary, stereo, control, state, and
  identity ownership across all three owners
- [x] reject automatic Dream/Cyclic selection
- [x] reject a hard Transparent/Dream switch at `4x`
- [x] select one opt-in Transparent/Dream route over `0.5x..16x`
- [x] select `4x..8x` as the only candidate transition interval
- [x] preserve explicit choices and character-local controls
- [x] define complete-brief, evidence, rejection, cleanup, and public-admission
  boundaries
- [x] change documentation only

Result:

- automatic range selection is warranted as an opt-in neutral intent
- the admitted owners cannot be called seamless without a new complete route
- Cyclic remains manual and bypasses the route
- the route must use exact target frames, not floating threshold comparisons
- the route must introduce no second source timeline or post-stretch adapter
- Batch 35.2 is ready as documentation only
- no candidate or public implementation is ready

## Batch 35.2 - Complete Exact-Target Route Brief

Status: complete

Documentation only. Freeze one
`ExactTargetTransparentDreamRouter` without changing code.

- [x] freeze exact target validation and empty/identity behavior
- [x] freeze the private exact-target Transparent entry and prove which
  promoted selector owns compression and expansion
- [x] freeze one monotonic source/output map shared by both contributions
- [x] freeze render-wide log-ratio weight arithmetic, endpoint ownership, and
  ties without floating routing ambiguity
- [x] freeze correlation/level treatment without limiter, adaptive loudness,
  post-fade, or unbounded gain
- [x] freeze head/tail alignment, exterior padding, crop, and exact target
- [x] freeze linked-channel analysis, weights, normalization, and synthesis
- [x] freeze deterministic identity and neutral Dream defaults
- [x] cap peak memory at final output plus one output-sized contribution and
  duration-independent owner state
- [x] freeze structural, synthetic, boundary, long-form mono, and independent
  linked-stereo gates
- [x] freeze immutable candidate source identity, isolation, stop, repair,
  rejection, cleanup, and minimal admission
- [x] keep public API, cache, artifact, runtime, UI, Loophole, and Chorus
  blocked

The brief must contain no `decide later` gap. Stop if exact-target Transparent
requires changed acoustic equations, if the owners cannot share one map and
boundary lattice, or if bounded output staging requires a public or runtime
surface.

Authority:

- `docs/architecture/offline-automatic-exact-target-transparent-dream-router-brief.md`

Result:

- exact integer dispatch owns `ceil(N/2)..=16N`
- compression and expansion use their promoted Transparent selectors
- both owners share one linear boundary-coordinate map and exact output lattice
- the overlap uses one render-wide log-ratio smoothstep and a convex
  linear-amplitude mix
- pure `4N` and `8N` endpoints bypass the blend for byte parity
- buffer reuse caps staging at final output plus one output-sized contribution
- evidence, listening, rejection, cleanup, and minimal admission are complete
- no code or candidate state entered `main`
- Batch 35.3 is ready as one isolated candidate

## Batch 35.3 - Isolated Fixed-Ratio Route Candidate

Status: complete; evidence-invalid

Implement one complete route in a disposable worktree. Run:

1. compile, identity, exact length, finiteness, determinism, map, boundary,
   parity, memory, and linked-stereo structural controls
2. pitch, event placement, replica, crest, tonal, level, transition, and
   linked-stereo synthetic controls
3. concealed long-form mono at pure-owner and transition targets
4. eligible independent linked-stereo review unless Contract `085` records a
   new checkpoint-scoped product decision after all hard stereo controls

Mandatory transition probes include targets immediately below, at, and above
`4N` and `8N`, plus representative `5x`, `6x`, and `7x` interiors.

Reject on audible combing, phasing, doubled attacks, micro-echo, image pull,
level step, boundary discontinuity, arbitrary energy redistribution, loss of
Dream smoothness, or loss of Transparent source readability.

Result:

- checkpoint `50c3d028ae1d5b0d057e74899b84a1a27c0e0038`, tree
  `0ff62f572eef222d38ac356d3874c973d78ba2d2`
- normal-profile stretch regression passes `204/204`
- two unchanged release conformance rounds pass construction `1/1` and
  structural `8/8`
- the first acoustic owner stops on pure Transparent `rademacher-noise`,
  `N=96000`, `T=4N-1`
- byte parity is exact; owner peak `10.370356` conflicts with the brief's
  universal `8.0` ceiling
- no interior, later synthetic, long-form, or listening owner ran
- the checkpoint is not an acoustic pass or renderer rejection

## Batch 35.4 - Peak-Gate Ownership Reassessment

Status: complete

Documentation only.

- [x] classify checkpoint `50c3d028` as evidence-invalid
- [x] keep pure Transparent governed by byte parity and its admitted integrity
  rules without a new absolute peak ceiling
- [x] keep pure Dream governed by byte parity and its admitted integrity rules
- [x] keep interior route-created overshoot terminal against the larger
  sample-aligned arm peak plus two `f32` ulps
- [x] change no owner, route, map, weight, source, seed, comparator, threshold
  sweep, listening rule, code, or public surface
- [x] authorize one exact source replay under new isolation identity
- [x] close and remove the Batch 35.3 worktree, branch, acoustic ref, generated
  state, and evidence root after this docs commit

Result:

- the gate now separates inherited owner behavior from route-created behavior
- the correction resolves the contradiction without tuning the candidate
- checkpoint `50c3d028` remains the sole source authority for exact restoration
- Batch 35.5 is ready as one full replay from conformance
- public Automatic remains blocked

## Batch 35.5 - Exact Candidate Replay

Status: complete; rejected at synthetic pitch

Restore only the frozen candidate source, tests, nextest profile, conformance
ledger, and source manifest from checkpoint `50c3d028` into the newly named
disposable worktree. Keep current canonical docs from `main`.

- [x] require all old Batch 35.3 and new Batch 35.5 isolation identities absent
- [x] restore and hash-prove every source named by the corrected brief
- [x] record current `main`, restored checkpoint/tree, toolchain, platform,
  Effigy, and nextest identity
- [x] pass compile, construction, and structural conformance twice unchanged
- [x] freeze the new acoustic ref directly at the clean conformance commit
- [x] restart the complete corrected synthetic gate from identity/parity
- [x] stop on the first failure before later synthetic or listening work

No candidate equation, evidence source, comparator, seed, assertion other than
the corrected peak ownership, public API, or product integration may change.

Result:

- checkpoint `db2a02d35f39a035e44803d0cc26861dcebe2534`,
  tree `ab8bf005fe8fe72522e3edc23b617d2ac37b5cd8`
- compile, two construction `1/1` and structural `8/8` rounds, and non-acoustic
  regression `204/204` pass
- corrected identity/parity passes `150` rows
- pitch rejects low tone at `6N`, `110 Hz`
- Transparent error is `0.16404282837539305` cents, Dream error is
  `6.277316077755877` cents, and Automatic error is
  `8.717736874188192` cents
- Automatic is `2.440420796432315` cents worse than the worse arm against the
  frozen `1`-cent allowance
- no later synthetic owner, long-form render, mono listening, or linked-stereo
  review runs
- the worktree, branch, generated evidence, and build state are deleted after
  this closeout; the acoustic ref remains through Batch 35.6 reassessment
- nothing enters `main`

## Batch 35.6 - Public Route Decision And Architecture Reassessment

Status: complete

Documentation only. Reject the tested route shape. Decide whether Automatic
retains one materially different complete architecture path or closes in
favour of explicit Transparent, Dream, and Cyclic modes. Do not reinterpret
the pitch receipt, tune the blend, or start implementation.

Result:

- the tested route is rejected at product authority
- no materially different complete route is source-backed under current owner
  and closed-program boundaries
- hard switching fails the seamless product promise
- alignment, correlation, masks, band splits, material selection, or changed
  weights repair the rejected seam
- one coherent synthesis field is a new renderer and reopens closed successor
  or component programs
- Automatic closes for the current owners with no API, discovery, identity,
  cache, runtime, UI, Loophole, or Chorus surface
- explicit Transparent, Dream, and Cyclic remain unchanged
- the Batch 35.5 acoustic ref is deleted after this reassessment commit
- Batch 35.8 is ready as docs-only lane closeout

## Batch 35.7 - Minimal Public Admission

Status: cancelled; Automatic rejected

No public Automatic boundary exists. Nothing is implemented.

## Batch 35.8 - Lane Closeout

Status: ready

Publish the exact executable matrix, remove or confirm removal of isolated
state, reconcile Contract `085` and all front doors, and select one next
planning checkpoint.

## Acceptance Criteria

- [x] operator intent supports clean automatic range selection
- [x] Automatic remains optional and explicit modes survive
- [x] Cyclic remains outside automatic routing
- [x] one target envelope and one transition interval are selected
- [x] no current owner is called seamless without transition evidence
- [x] Batch 35.3 has one frozen, evidence-invalid checkpoint
- [x] Batch 35.4 corrects gate ownership without candidate tuning
- [x] Batch 35.5 produces one valid route rejection
- [x] Batch 35.6 closes Automatic for the current owners
- [x] only Batch 35.8 is ready
- [x] public Automatic work is cancelled
- [x] planning changes documentation only
- [x] the complete brief owns every map, scheduler, boundary, stereo, memory,
  identity, evidence, rejection, cleanup, and admission decision
- [x] listening remains promotion authority

## Next Task

Execute Batch 35.8 only as documentation. Confirm every Automatic worktree,
branch, ref, candidate source, evidence root, and generated asset is absent;
publish the final explicit stretch matrix; reconcile front doors; and close
`g10.035`.
