# 078 Rhythm Continuity, Failure Containment, And Policy Normalization Contract

Status: active
Owner: core-product
Updated: 2026-04-09
Related contracts: `docs/contracts/047-warp-marker-transient-anchor-and-tempo-assist-analysis-contract.md`
Related architecture: `docs/architecture/dsp-analysis-feature-reference.md`

## Purpose

Freeze the contract for turning the rhythm engine from branch-heavy,
panic-sensitive state logic into a resilient, inspectable policy system.

## Required shared guarantees

- feature-worker failures must resolve to typed analysis failure or degraded
  output, not `join().unwrap()` crashes
- tempo and meter continuity policy must be data-driven enough to avoid near-
  duplicate branch families for each recommendation arm
- recommendation provenance and transition reasoning must stay inspectable for
  future tuning

## Rules

- continuity rules must be expressed as explicit policy tables, scorecards, or
  staged evaluators rather than ad hoc branch forks where feasible
- worker parallelism must preserve deterministic output and bounded failure
  containment
- rhythm policy modernization must not regress the already-closed tempo-assist
  and warp-analysis contracts

## Required proof surfaces

- corpus regressions for tempo and meter continuity
- targeted failure-injection tests for worker panic and partial-feature loss
- interactive rhythm demo coverage under contract `079`

## Next Task

Use this contract for the active strict `g09.010` lane, starting with
`docs/specs/batch-cards/013-g09-010-rhythm-worker-failure-containment.md`.
