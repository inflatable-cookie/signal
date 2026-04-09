# 009 - DSP Fidelity And Semantic-Analysis Realism Uplift

Status: draft
Owner: core-product
Created: 2026-04-08
Depends on: g09.001
Vision tags: `DSP`, `ANALYSIS`, `FIDELITY`
Contract refs: `046`, `047`, `077`

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

- [ ] define quality modes and their expected anti-aliasing or band-limited
      behavior
- [ ] keep existing fast paths explicit as low-quality or control-safe modes
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

- [ ] Signal exposes clear low/high-quality resampling posture
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
- [ ] run `cargo check -p signal-dsp-resample`
- [ ] run `cargo check -p signal-analysis-embed`
- [ ] run `effigy health`

## Next Task

Continue with `g09.010` and apply the same explicitness to the rhythm engine's
failure containment and policy logic.
