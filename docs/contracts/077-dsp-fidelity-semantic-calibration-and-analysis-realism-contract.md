# 077 DSP Fidelity, Semantic Calibration, And Analysis Realism Contract

Status: draft
Owner: core-product
Updated: 2026-04-08
Related contracts: `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`, `docs/contracts/047-warp-marker-transient-anchor-and-tempo-assist-analysis-contract.md`
Related architecture: `docs/architecture/dsp-analysis-feature-reference.md`

## Purpose

Freeze the contract for raising Signal's reusable DSP and analysis substrate
beyond bounded placeholder fidelity in resampling and semantic-tag projection.

## Required shared guarantees

- resampling quality tiers must be explicit, selectable, and testable
- higher-quality modes must include proper anti-aliasing or band-limited
  behavior rather than only interpolation-choice switches
- semantic embedding/tagging must publish calibration and provenance policy
  instead of only hand-tuned weights

## Rules

- deterministic fast paths may remain, but they must be named as such and not
  silently represent the whole crate capability
- model or scoring evolution must preserve inspectable output and benchmark
  evidence
- quality upgrades must not collapse real-time safety expectations in control or
  lightweight analysis paths

## Required proof surfaces

- objective resampling quality benchmarks and acceptance thresholds
- semantic-tag corpus evaluation and calibration evidence
- interactive demo coverage under contract `079`

## Next Task

Use this contract for the `g09` DSP/analysis fidelity roadmap covering
`signal-dsp-resample` and `signal-analysis-embed`.
