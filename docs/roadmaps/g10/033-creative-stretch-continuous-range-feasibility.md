# 033 - Creative Stretch Continuous-Range Feasibility

Status: active; Batch 33.3 ready
Owner: dsp
Created: 2026-07-24
Depends on: `g10.031`, `g10.032`
Governing contracts: `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`,
`docs/contracts/085-creative-time-stretch-product-and-routing-contract.md`
Vision tags: `DSP`, `STRETCH`, `CREATIVE`

## Problem

Signal now exposes two admitted creative characters:

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
- [ ] admit or reject one isolated fixed-ratio candidate
- [ ] decide public widening and routing only after acoustic admission

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

Status: ready

Implement one complete candidate in a disposable worktree. Admit structural
and synthetic interior-ratio evidence before concealed long-form listening.
Delete the candidate and its scaffolding on failure. Do not add a public router
in this batch.

## Batch 33.4 - Public Range And Routing Decision

Status: pending Batch 33.3

Only after fixed-ratio admission, decide whether the accepted owner warrants a
continuous public range and hidden same-character routing. Freeze cache and
consumer work separately. Dynamic ratio remains later work.

## Acceptance Criteria

- [x] current exact-ratio coverage is stated without implying continuous bands
- [x] `Dream` and `Cyclic` remain separate user-selected characters
- [ ] any future route owns Contract `085` continuity rather than crossfading by
  assertion
- [x] only one complete candidate can become executable
- [x] no implementation becomes ready before a complete brief

## Next Task

Execute Batch 33.3 only from
`offline-creative-continuous-direct-renewal-dream-brief.md`. Create one
isolated worktree, change only private target validation, complete two clean
conformance rounds, checkpoint once, then run the fixed acoustic gates. Do not
widen the public API or begin routing.
