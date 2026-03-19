# 016 - Warp-Marker, Transient-Anchor, And Tempo-Assist Analysis Depth

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g07.015, g06.018
Vision tags: `STRETCH`, `ANALYSIS`, `MEDIA`

## Problem

A sample-domain stretch engine still needs reusable analysis depth for markers,
anchors, and tempo assistance before downstream editing workflows become credible.

## Goals

- [ ] define reusable warp-marker, transient-anchor, and tempo-assist analysis surfaces
- [ ] keep analysis output aligned with the stretch engine and media identity
- [ ] expose host-visible analysis readiness and invalidation state

## Non-Goals

- [ ] no product-specific editing gestures or marker UI
- [ ] no ML-heavy media intelligence breadth here

## Execution Plan

### Batch 16.1 - Analysis Contract

- [x] define marker, anchor, and tempo-assist analysis semantics
- [x] align the output with media identity and stretch execution needs

### Batch 16.2 - Service Depth

- [x] implement the first credible marker and anchor analysis depth
- [x] keep readiness and invalidation receipts aligned with the analysis contract

### Batch 16.3 - Focused Proof

- [x] add focused proofs for marker, anchor, and tempo-assist analysis behavior

## Acceptance Criteria

- [ ] Signal has explicit marker and anchor analysis surfaces
- [ ] later editing and transform-artifact work can build on the same analysis truth
- [ ] hosts can observe analysis readiness without local guesswork

## Risks And Mitigations

- Risk: analysis depth drifts from actual stretch execution needs.
- Mitigation: bind it to engine, media, and invalidation receipts directly.

## Evidence Requirements

- [ ] log each meaningful marker-analysis tranche
- [ ] run focused analysis-service validation
- [ ] record deferred analysis breadth explicitly

## Batch 16.1 Outcome

Batch 16.1 freezes the first bounded warp-marker, transient-anchor, and
tempo-assist analysis contract in
`docs/contracts/047-warp-marker-transient-anchor-and-tempo-assist-analysis-contract.md`.

Signal now has one shared contract for:

- warp-marker, transient-anchor, tempo-assist, readiness, degraded-state, and
  invalidation meaning instead of host-local marker tools or private editor
  timing heuristics
- direct composition with the closed media-service, analysis-metadata, and
  sample-domain stretch-engine seams instead of inventing a second
  transform-analysis authority
- one explicit handoff into runtime analysis-service depth so later
  artifact-cache, preview, and audition work stays additive on a shared
  runtime-owned substrate

That gives Batch 16.2 one fixed runtime target for the first real marker and
anchor analysis baseline while keeping artifact-cache, low-latency audition,
and broader timing-intelligence depth explicitly deferred.

## Batch 16.2 Outcome

Batch 16.2 materializes the first runtime-owned warp-marker, transient-anchor,
and tempo-assist receipt family directly on the shared runtime substrate.

Signal now exposes:

- `RuntimeMarkerAnalysisSnapshot` and per-clip marker-analysis receipts derived
  from the closed clip-processing, stretch-engine, warp, and media-library
  seams instead of host-local marker tools
- typed readiness, invalidation, tempo-assist posture, and bounded marker or
  anchor counts through runtime observation, supervisor export, and stable
  host-edge JSON
- one focused baseline for later proof, artifact-cache, and audition work
  without reopening product-local marker ownership

Batch 16.2 stays intentionally bounded: marker counts are derived from the
shared runtime analysis descriptors that already exist today, not a new editor
engine or beat-grid authoring model.

## Batch 16.3 Outcome

Batch 16.3 closes the shared marker-analysis consumer seam.

Signal now proves:

- public runtime consumers can inspect runtime-owned warp-marker,
  transient-anchor, tempo-assist, readiness, and invalidation truth through
  `RuntimeObservationReport` and `RuntimeSupervisorReport` without host-local
  stretch-analysis reconstruction
- both stable host edges forward the same runtime-owned marker-analysis
  receipts through supervisor export instead of rebuilding marker heuristics
  per host
- `signal-supervisor-tools` now exposes the machine-readable
  `signal.runtime.marker-analysis-boundary` descriptor, and Effigy now owns
  `acceptance:marker-analysis-boundary` as the repo-owned rerun lane

This closes the bounded `g07.016` milestone. Fuller editor-grade marker tools,
beat-grid authoring, artifact-cache depth, and low-latency audition remain
explicit later work.

## Next Task

Continue `g07.017` with Batch 17.1 by freezing the post-warp render, cache,
and transform-artifact contract on top of the now-closed stretch and
marker-analysis boundaries before runtime artifact depth widens.
