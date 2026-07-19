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

Status: ready

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

## Batch 30.3 - Candidate Worktree

Status: blocked on Batch 30.2

- implement the complete candidate in one disposable branch or worktree
- keep instrumentation private to that worktree
- run structural and synthetic gates before generating listening audio
- generate the full long-form comparison pack only after those gates pass
- reject and delete the branch on failure; do not merge its scaffolding

## Batch 30.4 - Admission Or Closure

Status: blocked on Batch 30.3

If the candidate passes:

- complete mono and independent linked-stereo review
- merge the minimal renderer and regression surface
- remove the displaced baseline or retain it only as an explicit fallback
- update cache identity, promotion receipts, contract `046`, and product status

If it fails:

- record the dominant complete-system failure once
- remove the candidate branch
- reassess the architecture under Contract 084 before another implementation

## Completion Gate

- [ ] one complete candidate brief exists
- [ ] one complete candidate passes structural and synthetic gates
- [ ] long-form mono listening is competitive with the external reference
- [ ] linked-stereo evidence passes objective and independent listening review
- [ ] admitted production code contains no rejected-candidate scaffolding
- [ ] dynamic-ratio and RealtimePreview follow-on decisions are explicit

## Next Task

Run Batch 30.2. Produce one end-to-end successor brief from the retained Rubber
Band, Signalsmith, and operator evidence. Do not reopen Batch 29.7BE or add DSP
code to `main`.
