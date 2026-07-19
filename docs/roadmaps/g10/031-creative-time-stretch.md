# 031 - Creative Time-Stretch

Status: active; Batch 31.2 ready
Owner: dsp
Created: 2026-07-19
Depends on: g10.030 closure
Governing contract: `085`
Vision tags: `DSP`, `STRETCH`, `CREATIVE`, `QUALITY`

## Problem

Signal has a competitive transparent stretch baseline but no first-party path
for intentional long expansion. At `8x`, transparent event reconstruction is
not the only useful goal: controlled smear, evolving spectral motion, and
cloud-like output are valid product behavior.

Exposing unrelated algorithms and their native controls would push DSP policy
into every consumer. A hard ratio switch would create audible and UI seams.

## Goal

Build one offline creative-stretch product surface that:

- centers `8x` and studies `4x`, `8x`, and `16x`
- presents stable intent controls rather than algorithm parameters
- routes smoothly from coherent slow motion to spectral dream and later cloud
- preserves exact duration, determinism, linked stereo, and bounded memory
- stays separate from `OfflineHighQuality` and RealtimePreview

## Non-Goals

- no transparent successor reopening
- no RealtimePreview or audio-thread work
- no Loophole or Chorus UI implementation
- no external production dependency
- no cyclic renderer in the first candidate
- no `100x+` texture/freeze implementation in the first lane
- no simultaneous diffusive, cloud, and cyclic experiment queue

## Batch 31.1 - Product And Architecture Study

Status: complete

- [x] reframed the target from `800x` to `800%` / `8x`
- [x] separated creative quality from transparent Contract `084` admission
- [x] studied polyphase spectral, diffusive spectral, layered PV/granular,
  granular texture, cyclic sampler, image-resynthesis, STN, and learned systems
- [x] selected one stable product parameter vocabulary
- [x] froze coherent, diffusive, and cloud range ownership with logarithmic
  overlap bands
- [x] selected `DiffuseSpectral` as the first new candidate family
- [x] froze stereo, determinism, exact-length, memory, cache, and cleanup rules
- [x] changed documentation only

Authority:

- `docs/architecture/offline-creative-time-stretch-study.md`
- `docs/contracts/085-creative-time-stretch-product-and-routing-contract.md`

## Batch 31.2 - Comparator Target Freeze

Status: ready; docs and listening evidence only

- capture primary references at `4x`, `8x`, and `16x`
- use the retained percussion, bass, vocal, pad/sustain, and full-mix sources
- level-match under one documented policy
- record character, motion, detail, stereo, periodicity, ringing, level, and
  preference notes
- probe fixed ratios around `2x`/`4x` and `16x`/`32x` transition bands where
  the comparator permits useful evidence
- choose the target diffusive character; do not average incompatible winners
- freeze explicit structural and listening rejection thresholds
- write one complete `DiffuseSpectral` brief only after the target is frozen

Primary references:

- PaulXStretch
- REAPER `Rrreeeaaa`
- CDP `SPECTSTR`
- Sloom `Wide` and `Narrow`
- SoundHack `++spiralstretch`
- Ableton `Texture`

Secondary character controls:

- Akaizer or REAPER `ReaReaRea`
- Photosounder or ARSS

No DSP, candidate harness, fixture, report mode, or public API enters `main`.

## Batch 31.3 - Diffusive Candidate Brief

Status: blocked on Batch 31.2

Freeze one buildable renderer: transform geometry, source map, phase-diffusion
state, magnitude evolution, control mapping, linked stereo, boundaries, exact
length, memory, determinism, tests, rejection, and cleanup. Do not implement it
in the same batch.

## Batch 31.4 - Isolated Diffusive Candidate

Status: blocked on Batch 31.3

Implement one complete candidate in a disposable worktree. Admit structural
and synthetic controls before long-form listening. Delete the worktree and
candidate scaffolding on failure.

## Later Batches

Closed until the diffusive owner passes:

- minimal production admission
- coherent/diffusive overlap
- `LayeredCloud` study and candidate
- diffusive/cloud overlap
- dynamic-ratio state continuity
- cache and product-path review
- optional cyclic character
- `100x+` texture/freeze range

## Completion Gate

- [x] one product architecture and governing contract exist
- [ ] comparator target character is frozen
- [ ] one complete diffusive brief exists
- [ ] one isolated diffusive candidate passes structural and synthetic gates
- [ ] long-form mono listening passes at `4x`, `8x`, and `16x`
- [ ] linked-stereo mechanics and independent listening pass
- [ ] one overlap band is audibly continuous
- [ ] only minimal admitted product surface enters `main`

## Next Task

Execute Batch 31.2 only. Freeze the comparator-backed target and rejection
thresholds. Stop before candidate DSP or harness implementation.
