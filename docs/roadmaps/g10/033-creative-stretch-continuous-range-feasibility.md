# 033 - Creative Stretch Continuous-Range Feasibility

Status: complete
Owner: dsp
Created: 2026-07-24
Depends on: `g10.031`, `g10.032`
Governing contracts: `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`,
`docs/contracts/085-creative-time-stretch-product-and-routing-contract.md`
Vision tags: `DSP`, `STRETCH`, `CREATIVE`

## Problem

At lane entry, Signal exposed two admitted creative characters:

- `Dream` at exact `4x`, `8x`, and `16x`
- `Cyclic` at exact `2x`, `4x`, and `8x`

These are useful fixed-ratio islands, not continuous bands. They also express
different user intent. Substituting Cyclic for Dream at a ratio boundary would
turn hidden renderer selection into an audible character change.

The historical automatic router cannot reopen until Signal proves continuous
fixed-ratio ownership and compatible source-map, boundary, stereo, and
deterministic-state behavior inside one character.

## Goals

Decide whether Signal has one source-backed path to continuous fixed-ratio
creative coverage before any router or candidate implementation starts.

The lane must end in one of three honest outcomes:

1. an admitted renderer can be generalized without changing its accepted
   character;
2. one materially different complete owner and overlap architecture is ready
   for a docs-first implementation brief;
3. continuous creative routing remains unavailable and the exact-ratio public
   surface stays frozen.

- [x] audit current public, private, and admitted coverage
- [x] select one character-preserving complete direction
- [x] freeze that direction as one executable implementation brief
- [x] admit or reject one isolated fixed-ratio candidate
- [x] decide public widening and routing only after acoustic admission

## Non-Goals

- no candidate DSP or harness changes during feasibility work
- no automatic switching between `Dream` and `Cyclic`
- no dynamic ratio, pitch, reverse, cache, artifact, runtime, or consumer work
- no reopening of rejected stretch families through parameter changes
- no change to transparent `OfflineHighQuality`
- no external production dependency

## Fixed Product Boundaries

- `character` remains explicit user intent.
- Hidden range selection may change an internal owner only inside the same
  character.
- `Cyclic` remains manual and bypasses any future Dream range router.
- Exact target frames and one monotonic source/output map remain authoritative.
- A continuous claim requires interior ratios, not only endpoint passes.
- Fixed-ratio transition evidence precedes dynamic-ratio work.
- Listening remains the creative quality authority.

## Execution Plan

## Batch 33.1 - Coverage And Compatibility Audit

Status: complete

Documentation only.

- [x] publish one exact matrix of public and private ratio ownership
- [x] distinguish character selection from hidden range-owner selection
- [x] audit the retained coherent, Dream, and Cyclic owners for source map,
  frame scheduling, boundary, normalization, linked-stereo, deterministic
  state, and target-length compatibility
- [x] test the architecture claims against Contract `085` Rules 1-7 without
  changing or executing candidate DSP
- [x] identify every missing interior-ratio and overlap owner
- [x] decide whether either admitted mechanism can plausibly generalize while
  preserving its accepted sound
- [x] select one complete source-backed direction or close the lane
- [x] keep the decision in the canonical architecture and roadmap spine

Stop if the audit produces only a blend of different characters, a repair of a
rejected family, an unowned transition, or a menu of local mechanisms.

Result:

- `OfflineHighQuality` accepts arbitrary positive ratios but remains a
  structurally incompatible Transparent owner
- private Cyclic accepts any exact target from identity through `8x`, but only
  exact `2x`, `4x`, and `8x` have acoustic and public admission
- Dream's target-driven map and scheduler are mechanically ratio-generic behind
  its exact `4x`/`8x`/`16x` validation gate
- select `ContinuousDirectRenewalDream` over exact fixed targets
  `4N <= T <= 16N`
- keep every admitted acoustic equation and anchor output unchanged
- keep both historical overlaps, public Cyclic widening, routing, cache,
  artifacts, dynamic ratio, and consumers out of scope

## Batch 33.2 - Complete Continuous Owner Brief

Status: complete

Freeze one `ContinuousDirectRenewalDream` renderer end to end: exact target
domain, unchanged map, schedule, analysis and synthesis state, boundaries,
linked stereo, determinism, bounds, fixed interior-ratio gates, concealed
listening pack, anchor byte-parity, rejection, and cleanup. The candidate may
change only the private ratio gate. Do not implement DSP in this batch.

- [x] freeze exact target arithmetic for every `4N <= T <= 16N`
- [x] bind immutable hashes for every admitted acoustic source file
- [x] freeze frame-adjacent endpoint, non-power-of-two, non-hop-divisible, and
  representative interior ratio cases
- [x] retain the existing structural and synthetic owners, extended to the
  frozen interior cases
- [x] freeze long-form mono and linked-stereo packs with explicit operator and
  eligible-listener ownership
- [x] require byte-exact `4x`, `8x`, and `16x` anchor parity
- [x] freeze bounded cost, deterministic state, rejection, cleanup, and minimal
  admission
- [x] run docs, Northstar, health, and validation gates

Stop if the brief requires any transform, window, hop, phase, seed, `space`,
blend, envelope, normalization, or post-process change. That would be a new
renderer, not ratio generalization.

Result:

- one canonical
  `offline-creative-continuous-direct-renewal-dream-brief.md`
- every exact integer target in `4N..=16N`
- fixed interior acoustic probes at `4.5x`, `6x`, `10x`, and `15.5x`
- immutable admitted acoustic source hashes and a gate-only production diff
- exact parent parity at `4x`, `8x`, and `16x`
- construction, structural, synthetic, concealed mono, and linked-stereo
  gates with explicit counts and ownership
- failure deletes the isolated candidate; success opens only a separate
  private minimal-admission batch

## Batch 33.3 - Isolated Fixed-Ratio Admission

Status: complete

Implement one complete candidate in a disposable worktree. Admit structural
and synthetic interior-ratio evidence before concealed long-form listening.
Delete the candidate and its scaffolding on failure. Do not add a public router
in this batch.

Result:

- checkpoint `0e9969ab`, tree `e5184e08`, passed two complete conformance
  rounds with identical normalized receipts
- structural admission completed `160/160` rows and `56/56` renders per round
- acoustic admission completed `154/154` rows and `138/138` renders
- exact `4x`, `8x`, and `16x` anchors remained byte-identical
- concealed mono passed `20/20` usable ties against PaulXStretch
- all `60` long-form stereo hard-control renders passed
- the operator accepted `20/20` neutral comparisons and preserve-to-widen
  trios, then waived independent review for checkpoint `0e9969ab`
- commit `73910aad` admits only private `4N..=16N` validation, internal
  renderer identity v2, and focused continuous regression owners
- candidate runners, comparators, receipts, listening assets, public API,
  routing, cache, artifacts, runtime, Loophole, and Chorus stayed out of
  `main`

## Batch 33.4 - Public Range And Routing Decision

Status: complete

Documentation only.

- [x] decide whether public `Dream` accepts every exact target in `4N..=16N`
- [x] replace exact-ratio-list semantics with one explicit continuous-range
  contract without changing `Cyclic`
- [x] decide whether any hidden same-character router remains necessary now
  that one private owner covers the complete admitted Dream range
- [x] freeze public errors, discovery/introspection, engine identity, and
  focused regression ownership
- [x] keep lower overlap, continuous Cyclic, cache, artifacts, dynamic ratio,
  runtime, UI, Loophole, and Chorus outside the admission
- [x] make one public implementation batch ready only if the complete surface
  has no unresolved semantic choice

Stop if public widening requires character substitution, a second renderer,
dynamic-ratio state, source-dependent selection, or consumer-owned policy.

Result:

- public Dream will accept every exact target in `4N..=16N`
- one `CreativeStretchRatioDomain` reports continuous Dream bounds and exact
  Cyclic ratios
- the discrete Dream ratio list and `supported_ratios()` are removed
- public behavior identity advances to `signal-creative-stretch-v3`
- existing request fields and error variants remain; out-of-range Dream
  targets return `UnsupportedTargetFrames` before allocation
- public Dream dispatches directly to the single admitted owner; no
  same-character router, overlap, blend, or fallback is added
- Batch 33.5 is frozen to `creative.rs`, `lib.rs`, rustdoc, exports, and
  focused public parity tests

## Batch 33.5 - Public Continuous Dream Admission

Status: complete

Implement only the public surface frozen by Batch 33.4. Preserve byte-exact
anchor behavior and the admitted private renderer. Do not add routing, cache,
artifacts, dynamic ratio, runtime, or consumer integration.

Result:

- public Dream accepts every exact target in `4N..=16N`
- public discovery reports continuous Dream bounds and exact Cyclic ratios
- behavior identity is `signal-creative-stretch-v3`
- only `creative.rs` and `lib.rs` changed
- focused public owners pass `11/11`
- retained private Dream owners pass `18/18`
- public/private parity is byte-exact across anchors, interior targets,
  one-frame boundaries, mono, stereo, and all admitted `space` values
- private Dream and Cyclic renderer trees remain unchanged
- no router, cache, artifact, dynamic ratio, runtime, UI, Loophole, or Chorus
  surface entered the batch

## Batch 33.6 - Continuous Creative Range Closeout

Status: complete

Close `g10.033`, publish the exact executable coverage matrix, and choose the
next planning checkpoint for the paused lower Dream overlap and separate
continuous Cyclic question. Do not merge those questions into this lane.

Result:

- the canonical public-surface architecture publishes the exact executable
  matrix and separates API acceptance from quality promotion
- public creative coverage is Dream at every exact target `4N <= T <= 16N`
  and Cyclic at exact `2N`, `4N`, or `8N`
- Dream and Cyclic remain explicit characters; shared `4x` and `8x` targets do
  not imply routing, blending, fallback, or acoustic equivalence
- Transparent `OfflineHighQuality`, prototype `RealtimePreview`, and
  render-plane `Repitch` remain separate owners under Contract `046`
- no creative dynamic ratio, automatic route, cache, artifact, runtime, UI,
  Loophole, or Chorus path is executable
- `g10.033` closes without candidate or harness code
- the next planning checkpoint is continuous Cyclic feasibility
- lower Dream remains paused because no compatible same-character owner exists

Continuous Cyclic is selected before lower Dream because the private Cyclic
owner already has general exact-target geometry through `8x`. That is only a
source-backed feasibility lead. It is not an acoustic claim, public widening,
or implementation authority. A new docs-first roadmap must decide the domain,
freeze character continuity and complete evidence, and stop if interior
targets change the accepted Cyclic effect.

## Acceptance Criteria

- [x] current exact-ratio coverage is stated without implying continuous bands
- [x] `Dream` and `Cyclic` remain separate user-selected characters
- [x] any future route owns Contract `085` continuity rather than crossfading by
  assertion
- [x] only one complete candidate can become executable
- [x] no implementation becomes ready before a complete brief
- [x] exact executable and non-executable coverage is published
- [x] the next planning checkpoint is selected without starting it

## Next Task

Execute `g10.034` Batch 34.3 only. Implement and execute the frozen
`ContinuousEventLedgerCyclic` candidate in one disposable worktree. Keep lower
Dream, public widening, routing, cache, artifacts, dynamic ratio, runtime, UI,
Loophole, and Chorus closed.
