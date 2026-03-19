# 017 - Post-Warp Render, Cache, And Transform-Artifact Depth

Status: complete
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

- [x] define transform-artifact identity, invalidation, and reuse semantics
- [x] align the contract with render, media, and marker-analysis surfaces

### Batch 17.2 - Runtime Artifact Depth

- [x] implement stronger post-warp render and cache artifact behavior
- [x] keep readiness and recovery receipts aligned with the new artifact model

### Batch 17.3 - Focused Proof

- [x] add focused proofs for post-warp render, cache, and invalidation behavior

## Acceptance Criteria

- [x] Signal has explicit transform-artifact and post-warp cache semantics
- [ ] later preview and audition work can build on the same artifact model
- [x] hosts can inspect transform-artifact readiness and reuse state directly

## Risks And Mitigations

- Risk: artifact behavior diverges between preview and offline render.
- Mitigation: require one runtime-owned artifact contract across both paths.

## Evidence Requirements

- [ ] log each meaningful transform-artifact tranche
- [ ] run focused artifact and invalidation validation
- [ ] record deferred artifact breadth explicitly

## Batch 17.1 Outcome

Batch 17.1 freezes the first bounded post-warp render, cache, and
transform-artifact contract in
`docs/contracts/048-post-warp-render-cache-and-transform-artifact-contract.md`.

Signal now has one shared contract for:

- transform-artifact identity, readiness, invalidation, reuse, and degraded
  posture instead of host-local preview caches or private export scratch
  stores
- direct composition with the closed media-service, stretch-engine, and
  marker-analysis seams instead of inventing a second transform-cache
  authority
- one explicit handoff into runtime artifact depth so later preview and
  audition work stays additive on a shared runtime-owned substrate

That gives Batch 17.2 one fixed runtime target for the first real
transform-artifact baseline while keeping broader cache-retention,
preview-execution, and low-latency audition depth explicitly deferred.

## Batch 17.2 Outcome

Batch 17.2 materializes the first runtime-owned transform-artifact receipt
family in `signal-runtime`.

Signal now has one shared post-warp artifact baseline for:

- transform-artifact readiness, invalidation, cached-media readiness, and
  reuse posture derived from the closed clip-processing, stretch, marker, and
  media seams instead of host-local preview-cache heuristics
- direct export through runtime observation, supervisor, clip-render, offline
  render preview, and stable host-edge JSON surfaces instead of split
  render-only and preview-only artifact stories
- bounded per-clip artifact identity so later cache reuse, preview, and
  audition depth can widen from a typed runtime-owned substrate

That gives Batch 17.3 one fixed consumer target for downstream proof while
keeping broader cache retention, low-latency audition, and richer artifact
storage policy explicitly deferred.

## Batch 17.3 Outcome

Batch 17.3 closes the downstream-style consumer seam for the bounded
transform-artifact contract.

Signal now has one shared proof boundary for:

- public runtime consumption of transform-artifact readiness, invalidation,
  cached-media readiness, and reuse through observation, supervisor,
  clip-render, and offline-preview surfaces
- stable local and server host-edge export of the same runtime-owned
  transform-artifact truth instead of host-local preview-cache reconstruction
- machine-readable repo-owned boundary evidence through
  `signal-supervisor-tools` and `acceptance:transform-artifact-boundary`

This closes the bounded `g07.017` seam while keeping fuller cache-retention,
low-latency audition, and richer storage-policy depth explicitly deferred.

## Next Task

Continue `g08.001` with Batch 1.1 by freezing the runtime-owned live Linux
audio backend ownership and session-lifecycle contract before deeper ALSA,
JACK, and PipeWire runtime realization widens.
