# 030 - Stretch Consolidation And Completion

Status: complete - frozen baseline retained
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

Signal needed one stable competitive baseline and a bounded path to assess one
complete successor, not a library of failed experiments.

## Goal

Finish the Signal-native OfflineHighQuality successor program by:

- freezing the current production behavior as the fallback baseline
- removing rejected renderers and experiment-only report surfaces
- retaining a compact Signal/external comparator and long-form listening pack
- developing any complete successor outside the production branch
- admitting a successor only if it wins the fixed evidence sequence
- closing on the baseline when reassessment finds no qualifying renderer family

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

Status: complete - rejected before implementation

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

Result:

- [x] created one disposable worktree from `358ec19e`
- [x] checked the frozen event refinement against the first structural gate
- [x] proved all `256` `H=256` impulse phases commit at `e-15`, not `e`
- [x] stopped before renderer, fixture, report, or listening-pack work
- [x] deleted the untouched candidate worktree and branch

Dominant failure:

- the frozen 16-sample energy-rise metric has a 16-sample maximum plateau for
  an isolated impulse
- the frozen earlier-sample tie break selects the first plateau sample
- exact event placement therefore fails by `15` source samples by construction

## Batch 30.6 - Fixed-Ratio Admission Or Family Closure

Status: complete - family closed

If Batch 30.5 passes:

- generate and complete the fifteen-row long-form mono blind pack
- complete linked-stereo structural evidence and independent listening
- merge only the minimal admitted renderer and regression surface
- update cache identity and promotion receipts deliberately

If Batch 30.5 fails:

- [x] recorded the dominant structural contradiction once
- [x] deleted the candidate worktree and branch
- [x] retained the production baseline byte-exact
- [x] closed the multiresolution phase-vocoder successor family under Rule 7

## Batch 30.7 - Non-Phase-Vocoder Feasibility Study

Status: complete - no candidate promoted

- [x] tested WSOLA and source-synchronous overlap-add against polyphonic,
  replica, timing, and linked-stereo ownership
- [x] reconciled the direct subband sinusoidal option with Signal's pinned
  SBSMS source-feasibility results
- [x] tested deterministic sines/transients/noise decomposition against joint
  timing, recombination, boundary, and stereo ownership
- [x] tested learned waveform synthesis against target ratios, determinism,
  memory, channel ownership, training, and dependency constraints
- [x] found no family with a source-backed reason to clear every Contract `084`
  gate
- [x] changed documentation only; no candidate, harness, fixture, or report
  surface entered `main`

Authority:

- `docs/architecture/offline-time-stretch-non-phase-vocoder-feasibility.md`

## Batch 30.8 - Product Review

Status: cancelled - no fixed-ratio promotion

Dynamic ratio, independent pitch, cache, artifact, and product paths retain
their frozen production behavior. RealtimePreview source fill remains paused
in `g10.028`. No successor product review is authorized without a future
fixed-ratio promotion.

## Architecture Checkpoint

Status: resolved - baseline closure

The non-phase-vocoder study found no family that plausibly owns the full
source map, polyphonic coherence, transient, linked-stereo, boundary, exact
length, determinism, and bounded-memory problem. `g10.030` closes on the
competitive frozen production baseline.

## Completion Gate

Promotion path, not reached:

- [x] one complete candidate brief exists
- [ ] one complete candidate passes structural and synthetic gates
- [ ] long-form mono listening is competitive with the external reference
- [ ] linked-stereo evidence passes objective and independent listening review
- [ ] admitted production code contains no rejected-candidate scaffolding

Closure path, complete:

- [x] dynamic-ratio and RealtimePreview follow-on decisions are explicit
- [x] rejected candidate surfaces are absent from `main`
- [x] repeated event-placement failure closed the phase-vocoder successor family
- [x] non-phase-vocoder feasibility covered every Contract `084` ownership
  boundary
- [x] no reviewed family justified a new complete candidate
- [x] the frozen baseline remains the only production renderer

## Next Task

None in the OfflineHighQuality successor lane. Retain the frozen baseline.
Reopen only when the feasibility decision records new whole-system evidence
that satisfies its explicit triggers. `g10.028` remains paused.
