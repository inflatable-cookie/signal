# 018 - Low-Latency Audition, Scrub, And Preview Transform Services

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g07.015, g07.017
Vision tags: `PREVIEW`, `STRETCH`, `MEDIA`

## Problem

Loophole's future browser and editing depth needs preview and audition services
that stay aligned with the stretch engine instead of using shallow host-local approximations.

## Goals

- [ ] define low-latency audition, scrub, and preview transform semantics
- [ ] keep preview behavior aligned with sample-domain stretch and artifact truth
- [ ] expose host-visible preview readiness and degraded-state behavior

## Non-Goals

- [ ] no full browser-remote preview contract here
- [ ] no product-specific editing workflow surface

## Execution Plan

### Batch 18.1 - Preview Contract

- [x] define audition, scrub, and transform-preview semantics
- [x] align preview output with stretch, artifact, and media-service surfaces

### Batch 18.2 - Service Baseline

- [x] implement the first credible low-latency transform preview service baseline
- [x] keep readiness and fallback receipts aligned with the contract

### Batch 18.3 - Focused Proof

- [x] add focused proofs for preview and audition transform behavior

## Acceptance Criteria

- [x] Signal has explicit low-latency transform preview semantics
- [x] later browser and workflow work can reuse the same preview substrate
- [x] hosts can observe preview readiness without host-local approximations

## Risks And Mitigations

- Risk: preview behavior diverges from offline or playback truth.
- Mitigation: bind it directly to the stretch engine and transform-artifact contract.

## Evidence Requirements

- [x] log each meaningful transform-preview tranche
- [x] run focused preview-service validation
- [x] record deferred preview breadth explicitly

## Batch 18.1 Outcome

- Signal now has one explicit runtime-owned contract for low-latency audition,
  scrub preview, preview service class, readiness, degraded state, fallback,
  and artifact alignment instead of host-local preview players or product-local
  browser shells.
- the authority line is explicit: media-service, stretch-engine,
  marker-analysis, and transform-artifact truth remain the anchors, which
  prevents later preview work from reopening a second transform or cache
  substrate.
- Batch 18.2 can now materialize the first credible preview-service receipt
  family without reopening preview semantics or host-local playback ownership.

## Batch 18.2 Outcome

- `signal-runtime` now owns the first bounded `RuntimePreviewTransformServiceSnapshot`
  family, derived directly from the closed media-service, stretch-engine,
  marker-analysis, and transform-artifact seams instead of host-local preview
  playback state.
- runtime observation, supervisor export, clip-render results, offline render
  contract preview, and both stable host-edge JSON paths now carry the same
  preview service class, readiness, degraded state, fallback, active audition,
  and scrub-supported truth.
- Batch 18.3 can now focus on downstream-style proof and machine-readable
  acceptance rather than reopening preview semantics or host-local preview
  ownership.

## Batch 18.3 Outcome

- the bounded low-latency preview seam is now proven through public runtime,
  both stable host edges, and the machine-readable
  `signal.runtime.preview-transform-boundary` descriptor instead of only
  runtime-internal and host-internal baseline tests.
- Effigy now owns `acceptance:preview-transform-boundary` as the repo-owned
  rerun lane for low-latency audition, scrub support, preview readiness,
  degraded-state, and fallback proof.
- `g07.018` is now closed, and the active queue moves to `g07.019` for
  integrated acceptance depth across the widened multichannel, Linux,
  controller, and stretch surfaces.

## Next Task

Continue `g08.001` with Batch 1.1 by freezing the runtime-owned live Linux
audio backend ownership and session-lifecycle contract before deeper ALSA,
JACK, and PipeWire runtime realization widens.
