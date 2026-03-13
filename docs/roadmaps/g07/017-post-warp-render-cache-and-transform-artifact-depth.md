# 017 - Post-Warp Render, Cache, And Transform-Artifact Depth

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g07.015, g07.016
Vision tags: `STRETCH`, `RENDER`, `CACHE`

## Problem

Sample-domain stretch and marker depth are not enough unless render and cache
artifacts can preserve that truth across reuse, preview, and export.

## Goals

- [ ] define post-warp render, cache, and transform-artifact semantics
- [ ] keep artifact identity and invalidation aligned with stretch execution truth
- [ ] expose runtime-owned transform-artifact readiness and reuse state

## Non-Goals

- [ ] no product-local cache browser or artifact UX
- [ ] no final archival media-policy breadth here

## Execution Plan

### Batch 17.1 - Artifact Contract

- [ ] define transform-artifact identity, invalidation, and reuse semantics
- [ ] align the contract with render, media, and marker-analysis surfaces

### Batch 17.2 - Runtime Artifact Depth

- [ ] implement stronger post-warp render and cache artifact behavior
- [ ] keep readiness and recovery receipts aligned with the new artifact model

### Batch 17.3 - Focused Proof

- [ ] add focused proofs for post-warp render, cache, and invalidation behavior

## Acceptance Criteria

- [ ] Signal has explicit transform-artifact and post-warp cache semantics
- [ ] later preview and audition work can build on the same artifact model
- [ ] hosts can inspect transform-artifact readiness and reuse state directly

## Risks And Mitigations

- Risk: artifact behavior diverges between preview and offline render.
- Mitigation: require one runtime-owned artifact contract across both paths.

## Evidence Requirements

- [ ] log each meaningful transform-artifact tranche
- [ ] run focused artifact and invalidation validation
- [ ] record deferred artifact breadth explicitly

## Next Task

Continue `g07.018` by building low-latency audition, scrub, and preview
transform services on top of the new engine and artifact substrate.

