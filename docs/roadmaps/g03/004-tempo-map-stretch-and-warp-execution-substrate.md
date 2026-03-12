# 004 - Tempo Map, Stretch, And Warp Execution Substrate

Status: complete
Owner: core-product
Created: 2026-03-12
Depends on: g03.003
Vision tags: `ENGINE`, `TIME`, `WARP`

## Problem

Signal’s current timing model and `g02` rhythm work can inform tempo-aware
behavior, but the reusable engine substrate still lacks a clear tempo-map and
warp execution contract. Products will otherwise keep treating stretch and warp
as app-local playback hacks instead of one reusable runtime behavior.

## Goals

- [x] define reusable tempo-map and clip/region warp execution semantics for Signal-owned crates
- [x] keep timing intent and warp realization explicit at the engine boundary
- [x] provide readiness and degradation surfaces that later clip/render work can depend on

## Non-Goals

- [x] no advanced algorithm portfolio yet
- [x] no product-specific warp editing workflows

## Execution Plan

### Batch 4.1 - Timing And Warp Contract

- [x] define tempo-map ownership, warp modes, and realized playback state surfaces
- [x] decide what must remain generic Signal substrate versus later host/product policy

### Batch 4.2 - Runtime Warp Proof

- [x] implement the first credible stretch/warp execution path on top of runtime timing and media/cache readiness
- [x] validate degraded and fallback reporting for unsupported or not-ready cases

## Acceptance Criteria

- [x] reusable tempo-map and warp execution semantics are explicit in Signal
- [x] runtime-owned readiness or degradation is observable without host-local inference
- [x] later clip processing and offline render can build on the same timing seam

## Risks and Mitigations

- Risk: tempo intent and warp realization collapse into one opaque playback field.
- Mitigation: keep execution truth and timing intent distinct in the contract.

## Evidence Requirements

- [x] log the warp-contract and runtime tranche
- [x] run focused runtime tests for tempo/warp transitions and degraded cases
- [x] capture any algorithm-quality caveats that remain intentionally deferred

## Next Task

Execute `g03.005` by defining reusable fade, gain-shape, and ordered
clip-treatment semantics, then prove nondestructive clip processing against
warped timing and automation-aware cases.
