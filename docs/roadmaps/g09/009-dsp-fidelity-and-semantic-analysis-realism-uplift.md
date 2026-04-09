# 009 - DSP Fidelity And Semantic-Analysis Realism Uplift

Status: active
Owner: core-product
Created: 2026-04-08
Depends on: g09.001
Vision tags: `DSP`, `ANALYSIS`, `FIDELITY`
Contract refs: `046`, `047`, `077`
Strict lane refs: `docs/specs/001-g09-lane-first-strict-adoption.md`, `docs/specs/batch-cards/009-g09-009-resampler-quality-tier-foundation.md`

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
- [ ] build benchmark and artifact-comparison harnesses for quality evaluation

### Batch 9.2 - Semantic Calibration Surface

- [ ] define corpus, expected tags, and confidence calibration policy for the
      semantic embedding path
- [ ] separate descriptor projection from scoring and confidence calibration
      where needed
- [ ] add explainable evidence for why tags and confidences were emitted

### Batch 9.3 - Proof And Demo

- [ ] add focused resampling and semantic regression gates
- [ ] publish machine-readable benchmark or corpus evidence
- [ ] wire interactive DSP/semantic demo scenarios into the demo substrate

## Acceptance Criteria

- [x] Signal exposes clear low/high-quality resampling posture
- [ ] semantic tagging has corpus-backed calibration and explainable evidence
- [ ] the crates no longer overclaim capability relative to their actual output

## Risks And Mitigations

- Risk: high-quality modes compromise realtime expectations.
- Mitigation: keep explicit mode selection and document performance posture.

- Risk: semantic calibration turns into unbounded model churn.
- Mitigation: freeze corpus and evaluation policy before tuning weights or
  models.

## Evidence Requirements

- [ ] log each DSP and semantic tranche
- [x] run `cargo check -p signal-dsp-resample`
- [ ] run `cargo check -p signal-analysis-embed`
- [ ] run `effigy health`

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

## Next Task

Continue the active strict lane from
`docs/specs/batch-cards/010-g09-009-resampler-proof-and-benchmark-surface.md`.
