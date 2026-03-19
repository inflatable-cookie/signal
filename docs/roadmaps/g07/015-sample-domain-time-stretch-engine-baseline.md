# 015 - Sample-Domain Time-Stretch Engine Baseline

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g06.017, g06.018
Vision tags: `STRETCH`, `MEDIA`, `EXECUTION`

## Problem

Signal already has a bounded tempo-map and warp realization surface, but not a
fuller sample-domain time-stretch engine path that Loophole can build on.

## Goals

- [ ] define the first reusable sample-domain time-stretch engine baseline
- [ ] keep stretch execution aligned with media identity, cache, and render semantics
- [ ] expose runtime-owned stretch readiness and degraded-state behavior

## Non-Goals

- [ ] no product-specific warp-marker editing UX
- [ ] no broad algorithm bakeoff detached from runtime needs

## Execution Plan

### Batch 15.1 - Stretch Contract

- [x] define sample-domain stretch execution semantics and fallback behavior
- [x] align the contract with current warp and clip-processing surfaces

### Batch 15.2 - Runtime Engine Baseline

- [x] implement the first credible sample-domain time-stretch path
- [x] keep render, preview, and diagnostics surfaces aligned with the new engine

### Batch 15.3 - Focused Proof

- [x] add focused proofs for stretch execution, readiness, and degraded behavior

## Acceptance Criteria

- [x] Signal has a real sample-domain time-stretch engine baseline
- [x] later marker, artifact, and preview work can build on the same engine truth
- [x] hosts observe stretch state through runtime-owned receipts

## Risks And Mitigations

- Risk: stretch work becomes pure algorithm exploration.
- Mitigation: keep it bound to runtime execution, receipts, and downstream needs.

## Evidence Requirements

- [x] log each meaningful stretch tranche
- [x] run focused stretch execution validation
- [x] record deferred stretch breadth explicitly

## Batch 15.1 Outcome

Batch 15.1 freezes the first bounded sample-domain stretch-engine contract in
`docs/contracts/046-sample-domain-time-stretch-engine-contract.md`.

Signal now has one shared contract for:

- stretch-engine class, readiness, degraded state, fallback, and scope instead
  of host-local preview transforms or private export DSP shells
- direct composition with the closed media-service, analysis-metadata,
  tempo-map, warp clip, and clip-processing seams instead of inventing a
  second transform authority
- one explicit handoff into marker-analysis, artifact, and audition work so
  later stretch depth stays additive on a shared runtime-owned substrate

That gives Batch 15.2 one fixed runtime target for the first real
sample-domain engine baseline while keeping marker-analysis, artifact cache,
and low-latency preview depth explicitly deferred.

## Batch 15.2 Outcome

Batch 15.2 materializes the first runtime-owned sample-domain stretch baseline
across observation, supervisor, render, preview, and stable host-edge
surfaces.

Signal now has one shared receipt family for:

- stretch-engine class, readiness, degraded state, and fallback instead of
  leaving stretch status implicit in warp clips or clip-processing errors
- clip render and offline-render preview receipts that expose the same stretch
  truth instead of rebuilding transform posture per surface
- stable host-edge and supervisor JSON export that carry stretch-engine state
  directly from runtime-owned clip-processing truth

That gives Batch 15.3 one fixed consumer seam to prove while keeping
marker-analysis, artifact cache, low-latency audition, and broader algorithm
depth explicitly deferred.

## Batch 15.3 Outcome

Batch 15.3 closes the bounded stretch-engine consumer seam.

Signal now has:

- focused downstream-style proof that `RuntimeStretchEngineSnapshot` remains
  consumable through public runtime, both stable host edges, and a
  machine-readable supervisor-tools boundary descriptor
- a repo-owned `acceptance:stretch-boundary` Effigy lane instead of a
  prose-only claim about stretch-engine class, readiness, degraded-state, and
  fallback behavior
- one explicit handoff into warp-marker, transient-anchor, and tempo-assist
  analysis depth without reopening host-local preview or render transform
  reconstruction

This closes `g07.015` as the bounded sample-domain stretch-engine milestone.
Marker-analysis, artifact-cache, low-latency audition, and broader
algorithm-support depth remain deferred instead of turning this baseline into
a hidden stretch-services queue.

## Next Task

Continue `g07.016` with Batch 16.1 by freezing the warp-marker,
transient-anchor, and tempo-assist analysis contract on top of the closed
sample-domain stretch-engine baseline before analysis-service depth widens.
