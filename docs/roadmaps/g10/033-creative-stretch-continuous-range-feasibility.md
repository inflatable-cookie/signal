# 033 - Creative Stretch Continuous-Range Feasibility

Status: active; Batch 33.1 ready
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

## Goal

Decide whether Signal has one source-backed path to continuous fixed-ratio
creative coverage before any router or candidate implementation starts.

The lane must end in one of three honest outcomes:

1. an admitted renderer can be generalized without changing its accepted
   character;
2. one materially different complete owner and overlap architecture is ready
   for a docs-first implementation brief;
3. continuous creative routing remains unavailable and the exact-ratio public
   surface stays frozen.

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

## Batch 33.1 - Coverage And Compatibility Audit

Status: ready

Documentation only.

- [ ] publish one exact matrix of public and private ratio ownership
- [ ] distinguish character selection from hidden range-owner selection
- [ ] audit the retained coherent, Dream, and Cyclic owners for source map,
  frame scheduling, boundary, normalization, linked-stereo, deterministic
  state, and target-length compatibility
- [ ] test the architecture claims against Contract `085` Rules 1-7 without
  changing or executing candidate DSP
- [ ] identify every missing interior-ratio and overlap owner
- [ ] decide whether either admitted mechanism can plausibly generalize while
  preserving its accepted sound
- [ ] select one complete source-backed direction or close the lane
- [ ] keep the decision in the canonical architecture and roadmap spine

Stop if the audit produces only a blend of different characters, a repair of a
rejected family, an unowned transition, or a menu of local mechanisms.

## Batch 33.2 - Complete Continuous Owner Brief

Status: pending Batch 33.1

Proceed only if Batch 33.1 selects one complete source-backed direction.
Freeze one renderer and any same-character overlap end to end: map, schedule,
analysis and synthesis state, boundaries, linked stereo, determinism, bounds,
interior-ratio gates, transition probes, listening pack, rejection, and
cleanup. Do not implement DSP in this batch.

If no direction survives Batch 33.1, close this roadmap instead.

## Batch 33.3 - Isolated Fixed-Ratio Admission

Status: pending Batch 33.2

Implement one complete candidate in a disposable worktree. Admit structural
and synthetic interior-ratio evidence before concealed long-form listening.
Delete the candidate and its scaffolding on failure. Do not add a public router
in this batch.

## Batch 33.4 - Public Range And Routing Decision

Status: pending Batch 33.3

Only after fixed-ratio admission, decide whether the accepted owner warrants a
continuous public range and hidden same-character routing. Freeze cache and
consumer work separately. Dynamic ratio remains later work.

## Acceptance

- current exact-ratio coverage is stated without implying continuous bands
- `Dream` and `Cyclic` remain separate user-selected characters
- any future route owns Contract `085` continuity rather than crossfading by
  assertion
- only one complete candidate can become executable
- no implementation becomes ready before a complete brief

## Next Task

Execute Batch 33.1 only. Audit continuous fixed-ratio ownership and
compatibility. Do not implement DSP, a router, cache, artifacts, or consumer
integration.
