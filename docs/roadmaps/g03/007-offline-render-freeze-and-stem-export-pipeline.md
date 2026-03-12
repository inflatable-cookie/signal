# 007 - Offline Render, Freeze, And Stem Export Pipeline

Status: active
Owner: core-product
Created: 2026-03-12
Depends on: g03.005, g03.006
Vision tags: `ENGINE`, `RENDER`, `EXPORT`

## Problem

Signal still lacks one deliberate offline render and freeze pipeline that
reuses the same clip, automation, warp, and plugin-chain semantics as live
runtime execution. Without that, export behavior will diverge from engine truth
or remain trapped in app-local code.

## Goals

- [ ] define one reusable offline render/freeze/stem export engine path
- [ ] reuse live engine timing, clip, and plugin semantics rather than parallel logic
- [ ] expose enough reporting and artifact metadata for downstream hosts to consume safely

## Non-Goals

- [ ] no product-specific export dialog or workflow work
- [ ] no cloud/distributed render orchestration here

## Execution Plan

### Batch 7.1 - Offline Render Contract

- [x] define reusable render requests, stem targets, freeze artifacts, and export result surfaces
- [x] keep live and offline timing/processing behavior aligned by contract

### Batch 7.2 - Render Engine Proof

- [x] implement and validate a first credible offline render path using the same engine substrate
- [x] cover freeze/stem cases without forking separate per-feature processing logic

### Batch 7.3 - Artifact And Parity Hardening

- [ ] turn in-memory render results into richer artifact/report receipts for downstream hosts
- [ ] close the current parity gaps around export sample-rate conversion, broader media formats, and plugin-render freshness

## Progress Notes

- 2026-03-12: opened `g03.007` and completed Batch 7.1 in
  `signal-runtime` by adding typed offline render request, stem target,
  freeze artifact, and contract-preview surfaces that resolve against
  runtime-owned topology, clip-processing, tempo-map, and plugin recall
  handoff state without reintroducing host-local recall ownership.
- 2026-03-12: completed Batch 7.2 by adding a first runtime-owned offline
  render engine path that decodes runtime-cached WAV media, reuses
  clip-processing treatment, executes the graph for main mix/stem output, and
  produces freeze artifacts from the same rendered stem buffers plus recall
  handoff metadata.

## Acceptance Criteria

- [ ] Signal owns one credible offline render/freeze/stem substrate
- [ ] offline results align with live engine timing and processing semantics
- [ ] hosts can consume render results without becoming the render engine

## Risks and Mitigations

- Risk: offline render becomes a second engine with diverging semantics.
- Mitigation: route render through the same typed timing, clip, and chain contracts.

## Evidence Requirements

- [x] log the render/freeze tranche
- [x] run focused validation for offline render parity and artifact metadata
- [x] record any intentionally deferred distributed-render scope

## Next Task

Continue `g03.007` with Batch 7.3 by turning the in-memory offline render
results into richer runtime-owned artifact/report receipts and closing the
current parity gaps around sample-rate conversion, broader media decode, and
offline plugin-render freshness before opening `g03.008`.
