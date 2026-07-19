# 030 - Stretch Consolidation And Completion

Status: active
Owner: dsp
Created: 2026-07-19
Depends on: g10.029
Governing contracts: `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`, `docs/contracts/084-stretch-candidate-isolation-and-promotion-contract.md`
Vision tags: `DSP`, `STRETCH`, `QUALITY`

## Problem

The stretch program produced useful boundary fixes, production selectors,
comparator evidence, listening evidence, and source research. It then drifted
into hundreds of narrow proofs. Most were rejected, but their code and active
planning state remained in Signal. The production renderer barely changed
while the crate and roadmap grew dramatically.

Signal needs one stable competitive baseline and one complete successor lane,
not a library of failed experiments.

## Goal

Finish the Signal-native stretch program by:

- freezing the current production behavior as the fallback baseline
- removing rejected renderers and experiment-only report surfaces
- retaining a compact Signal/external comparator and long-form listening pack
- developing one complete successor outside the production branch
- admitting only a successor that wins the fixed evidence sequence

## Non-Goals

- no Loophole or Chorus planning
- no render-plane integration
- no RealtimePreview source-fill work
- no scalar sweeps or isolated mechanism batches
- no external production dependency
- no claim of parity from objective metrics alone

## Batch 30.1 - Consolidate The Baseline

Status: complete

- [x] preserve the current OfflineHighQuality output and public product paths
- [x] remove the rejected frequency-adaptive research family
- [x] remove rejected hybrid, H/R/P, phase-gradient, adaptive-timeline,
  fixed-map peak, tail, stability, tracked-peak, and magnitude-slew renderers
- [x] remove hidden public review methods and candidate-only tests
- [x] replace the experiment report with the compact external comparator and
  blind listening pack
- [x] retain byte-exact, package, allocation, and missing-docs validation
- [x] supersede the Batch 29.7BE coefficient-only continuation

Commits:

- `43e9a96a` Remove rejected frequency-adaptive research
- `1d1b02f1` Consolidate stretch renderer and quality harness

Result:

- `66,500` deleted lines across the two code batches
- production bit-exact baseline unchanged
- `signal-dsp-stretch` source reduced to about `14,835` lines
- full retained package suite passes

## Batch 30.2 - Freeze One Complete Successor

Status: complete

Write one implementation brief. It must use the retained source studies and
operator evidence, but it must describe a complete renderer rather than
another local proof.

The brief must freeze:

- transform and simultaneous-resolution topology
- global source/output map and frame scheduling
- transient detection, reset/reassignment, placement, and replica policy
- tonal peak/phase propagation and dormant-state behavior
- linked-stereo decision and relationship ownership
- exact boundary, output-length, memory, and determinism rules
- one fixed synthetic gate and one long-form blind pack
- explicit rejection criteria and cleanup behavior

No DSP implementation lands in `main` in this batch.

Result:

- [x] froze `SourceAnchoredMultiresolutionPhaseField` as the only successor
- [x] assigned transform, map, scheduler, transient, tonal, stereo, boundary,
  memory, determinism, rejection, cleanup, and admission ownership
- [x] froze structural, synthetic, long-form mono, and independent stereo
  gates
- [x] resolved the historical `1.25x` note-checker ratio against Contract
  `084`: candidate admission uses `0.75x`, `1.5x`, and `2.0x`
- [x] changed documentation only; production DSP and harness behavior remain
  frozen

Authority:

- `docs/architecture/offline-time-stretch-successor-brief.md`

## Batch 30.3 - Candidate Worktree

Status: complete - rejected

- implement the complete candidate in one disposable branch or worktree
- keep instrumentation private to that worktree
- run structural and synthetic gates before generating listening audio
- generate the full long-form comparison pack only after those gates pass
- reject and delete the branch on failure; do not merge its scaffolding

Result:

- [x] created one disposable worktree from `13539f27`
- [x] implemented the complete fixed-ratio mono and linked-stereo candidate
- [x] passed identity, length, finiteness, map, coverage, boundary,
  determinism, fixed-memory, and linked-stereo structural controls
- [x] passed isolated-tone pitch at `0.75x`, `1.5x`, and `2.0x`
- [x] stopped on the first anti-replica failure before tonal, long-form, or
  listening work
- [x] removed the disposable candidate branch, implementation, tests, and
  instrumentation

Dominant failure:

- centered middle-scale flux committed an isolated impulse `896` source
  samples before the actual event entered the fixed `H/2` refinement interval
- the `0.75x` render placed the primary `128` samples late and produced a
  `0.17113242` secondary peak at projection offset `+257`; the `-24 dB`
  ceiling was `0.063095726`
- same-centre one-tick short reassignment cannot repair that early detector
  commit without changing the frozen architecture

## Batch 30.4 - Reassess Candidate Architecture

Status: complete

- [x] retained the Batch 30.3 failure as one complete-system rejection
- [x] rejected a detector-radius, threshold, reset-tick, or row repair
- [x] replaced the canonical brief with
  `EventSealedMultiresolutionPhaseField`
- [x] moved event detection ahead of synthesis through fixed lookahead
- [x] inserted finalized event samples into the absolute source lattice
- [x] gave every event sample one non-zero analysis-window owner and one mapped
  synthesis-window owner
- [x] froze detector, scheduler, transient, tonal, stereo, boundary, memory,
  gate, rejection, cleanup, and admission behavior
- [x] changed documentation only; production DSP and harness behavior remain
  frozen

Authority:

- `docs/architecture/offline-time-stretch-successor-brief.md`

## Batch 30.5 - Event-Sealed Candidate Worktree

Status: ready

- create one disposable worktree from the Batch 30.4 commit
- implement the complete brief without changing production routing
- keep renderer, fixtures, diagnostics, and reports private to the worktree
- run structural controls, including exact event-token and one-owner window
  invariants
- run the complete synthetic gate only after structural admission
- stop before generating long-form audio
- reject and delete the candidate on any miss
- close the multiresolution phase-vocoder family if event placement or replicas
  fail again

## Batch 30.6 - Fixed-Ratio Admission Or Family Closure

Status: blocked on Batch 30.5

If Batch 30.5 passes:

- generate and complete the fifteen-row long-form mono blind pack
- complete linked-stereo structural evidence and independent listening
- merge only the minimal admitted renderer and regression surface
- update cache identity and promotion receipts deliberately

If Batch 30.5 fails:

- record the dominant complete-system failure once
- delete the candidate worktree and branch
- retain the production baseline
- close the multiresolution phase-vocoder family when Rule 7 applies

## Batch 30.7 - Product Review

Status: blocked on fixed-ratio promotion

- review dynamic ratio and independent pitch composition
- decide explicit fallback behavior for unreviewed paths
- update Contract `046`, artifact/cache posture, and product status
- keep RealtimePreview source fill in `g10.028`

## Completion Gate

- [x] one complete candidate brief exists
- [ ] one complete candidate passes structural and synthetic gates
- [ ] long-form mono listening is competitive with the external reference
- [ ] linked-stereo evidence passes objective and independent listening review
- [ ] admitted production code contains no rejected-candidate scaffolding
- [ ] dynamic-ratio and RealtimePreview follow-on decisions are explicit

## Next Task

Run Batch 30.5 from the Batch 30.4 commit in one disposable worktree. Implement
`EventSealedMultiresolutionPhaseField` exactly. Stop after structural and
synthetic gates decide whether long-form listening audio may be generated.
