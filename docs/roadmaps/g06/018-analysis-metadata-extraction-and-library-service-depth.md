# 018 - Analysis Metadata Extraction And Library-Service Depth

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g06.017
Vision tags: `MEDIA`, `ANALYSIS`, `LIBRARY`

## Problem

Waveform and preview services are necessary, but Loophole and future consumers
also need richer reusable asset metadata and analysis services for browsing,
placement, preview, and later intelligence workflows.

## Goals

- [ ] define reusable analysis metadata and library-service semantics
- [ ] expose typed asset descriptors beyond simple waveform/preview readiness
- [ ] support later product workflows and advisory intelligence without
  reimplementing analysis services locally

## Non-Goals

- [ ] no product-local tagging UX or recommendation interface
- [ ] no broad ML/classification generation here

## Execution Plan

### Batch 18.1 - Metadata Contract

- [ ] define the first reusable asset-metadata and analysis-service descriptor family
- [ ] align metadata ownership with the earlier media-service baseline

### Batch 18.2 - Service Depth

- [ ] materialize the chosen metadata and library-service outputs in
  Signal-owned crates and exports
- [ ] keep runtime, supervisor, and host-edge surfaces on the same typed descriptors

### Batch 18.3 - Consumer Proof

- [ ] add focused proofs that downstream consumers can rely on analysis metadata
  without product-local extraction pipelines

## Acceptance Criteria

- [ ] Signal has reusable analysis metadata and library-service depth
- [ ] later products can consume asset-analysis descriptors through Signal-owned surfaces
- [ ] advisory feature work has stronger runtime/media inputs to build on

## Risks And Mitigations

- Risk: metadata extraction scope balloons into product intelligence features.
- Mitigation: keep the milestone on reusable descriptors and services only.
- Risk: metadata semantics drift from waveform/preview service state.
- Mitigation: require explicit alignment with `g06.017` readiness and invalidation.

## Evidence Requirements

- [ ] log each meaningful analysis-metadata tranche
- [ ] run focused validation for library-service descriptors
- [ ] record deferred intelligence breadth explicitly

## Next Task

Continue `g06.019` by turning the widened runtime and feature surface into a
deliberate fault-injection and acceptance substrate.
