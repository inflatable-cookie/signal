# 018 - Low-Latency Audition, Scrub, And Preview Transform Services

Status: planned
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

- [ ] define audition, scrub, and transform-preview semantics
- [ ] align preview output with stretch, artifact, and media-service surfaces

### Batch 18.2 - Service Baseline

- [ ] implement the first credible low-latency transform preview service baseline
- [ ] keep readiness and fallback receipts aligned with the contract

### Batch 18.3 - Focused Proof

- [ ] add focused proofs for preview and audition transform behavior

## Acceptance Criteria

- [ ] Signal has explicit low-latency transform preview semantics
- [ ] later browser and workflow work can reuse the same preview substrate
- [ ] hosts can observe preview readiness without host-local approximations

## Risks And Mitigations

- Risk: preview behavior diverges from offline or playback truth.
- Mitigation: bind it directly to the stretch engine and transform-artifact contract.

## Evidence Requirements

- [ ] log each meaningful transform-preview tranche
- [ ] run focused preview-service validation
- [ ] record deferred preview breadth explicitly

## Next Task

Continue `g07.019` by turning the widened multichannel, Linux, MIDI, control,
and stretch surfaces into integrated acceptance depth.

