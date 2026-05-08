# 009 - DSP Fidelity And Semantic-Analysis Realism Uplift

Status: complete
Owner: core-product
Created: 2026-04-08
Depends on: g09.001
Vision tags: `DSP`, `ANALYSIS`, `FIDELITY`
Contract refs: `046`, `047`, `077`
Strict lane refs: `docs/specs/001-g09-lane-first-strict-adoption.md`, `docs/roadmaps/g09/batch-cards/013-g09-010-rhythm-worker-failure-containment.md`

## Problem

The resampler and semantic-tagging surfaces are both useful but presently too
bounded: one is interpolation-only, and the other is largely heuristic scoring
without calibration or explicit quality posture.

## Goals

- [ ] introduce explicit resampling quality tiers with proper high-quality
      behavior
- [ ] calibrate and benchmark semantic-tag inference instead of relying only on
      hand-tuned weights
- [ ] keep fast deterministic paths available without overstating their
      fidelity

## Non-Goals

- [ ] no full ML platform or remote inference service
- [ ] no product-local tagging UX

## Execution Plan

### Batch 9.1 - Resampler Quality Architecture

- [x] freeze the first resampler-quality seam as the next ready batch
- [x] define quality modes and their expected anti-aliasing or band-limited
      behavior
- [x] keep existing fast paths explicit as low-quality or control-safe modes
- [x] build benchmark and artifact-comparison harnesses for quality evaluation

### Batch 9.2 - Semantic Calibration Surface

- [x] freeze the first semantic-calibration seam as the next ready batch
- [x] define corpus and expected top-tag posture for the
      semantic embedding path
- [x] define confidence calibration policy for the
      semantic embedding path
- [x] separate descriptor projection from scoring and confidence calibration
      where needed
- [x] add explainable evidence for why tags and confidences were emitted

### Batch 9.3 - Proof And Demo

- [ ] add focused resampling and semantic regression gates
- [ ] publish machine-readable benchmark or corpus evidence
- [ ] wire interactive DSP/semantic demo scenarios into the demo substrate

## Acceptance Criteria

- [x] Signal exposes clear low/high-quality resampling posture
- [x] semantic tagging has corpus-backed confidence calibration and explainable
      evidence
- [x] the crates no longer overclaim capability relative to their actual output

## Risks And Mitigations

- Risk: high-quality modes compromise realtime expectations.
- Mitigation: keep explicit mode selection and document performance posture.

- Risk: semantic calibration turns into unbounded model churn.
- Mitigation: freeze corpus and evaluation policy before tuning weights or
  models.

## Evidence Requirements

- [ ] log each DSP and semantic tranche
- [x] run `cargo check -p signal-dsp-resample`
- [x] run `cargo check -p signal-analysis-embed`
- [x] run `effigy health`

## Batch 9.1 Tranche 1 Outcome

The resampler now has an honest quality-tier foundation. `Nearest` and
`Linear` remain explicit fast deterministic modes, while `BandLimited` adds a
windowed-sinc path that performs low-pass smoothing instead of interpolation
alone. Focused tests prove chunked and offline outputs still match and that the
new high-quality path materially attenuates alias-prone downsampling input.

## Updated Reassessment Outcome

The next honest `g09.009` seam is still inside resampling, not semantics yet.
What remains unproven is the quality posture itself: the roadmap still wants a
benchmark or artifact-comparison surface before semantic calibration becomes
the active seam.

## Batch 9.1 Tranche 2 Outcome

The resampler proof surface is now explicit and reusable. The crate publishes a
machine-readable quality comparison report covering `Nearest`, `Linear`, and
`BandLimited`, and the frozen proof tests show that the high-quality path is
not just differently named: it materially attenuates alias-prone content while
keeping deterministic chunked/offline equivalence.

## Final Reassessment Outcome

The next honest `g09.009` seam is semantic calibration. Resampler quality
posture is now explicit enough that further resampler-only work would be churn;
the remaining fidelity gap is the heuristic semantic-tagging path and its lack
of frozen corpus-backed calibration evidence.

## Batch 9.2 Tranche 1 Outcome

The semantic calibration baseline is now explicit. The built-in semantic model
publishes explainable per-tag evidence, diagnostics record the emitted top
label, and the frozen tone/noise/pulse corpus now has a machine-readable
calibration report that fixes expected top-tag and confidence posture instead
of relying only on threshold checks and debug output.

## Updated Semantic Reassessment Outcome

There is still one honest `g09.009` seam left before handing off to
`g09.010`: confidence calibration policy. The corpus and explainable evidence
are now frozen, but confidence is still a lightweight heuristic blend over
margin and embedding activity, so the next batch should make that posture more
explicit rather than broadening into rhythm work early.

## Batch 9.2 Tranche 2 Outcome

The semantic confidence posture is now explicit and testable. Confidence
calibration records named components in diagnostics, the frozen semantic corpus
asserts confidence-ordering expectations in addition to tag/evidence posture,
and the semantic lane now has enough corpus-backed explainability to stop
claiming quality through heuristics alone.

## Final Reassessment Outcome

`g09.009` is complete. The next honest strict seam is `g09.010` Batch 10.1:
worker failure containment in `signal-analysis-rhythm`, starting with the
production `join().unwrap()` path in onset feature extraction.

## Next Task

Continue the active strict lane from
`docs/roadmaps/g09/batch-cards/013-g09-010-rhythm-worker-failure-containment.md`.
