# 007 - Offline Render, Freeze, And Stem Export Pipeline

Status: complete
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

- [x] turn in-memory render results into richer artifact/report receipts for downstream hosts
- [x] close the export sample-rate conversion gap without moving export ownership into hosts
- [x] close the remaining parity gaps around broader media formats and plugin-render freshness

### Batch 7.4 - Manifest And Host Parity Boundary

- [x] promote artifact/report receipts into a stronger typed manifest/report bundle for downstream packaging
- [x] define the explicit runtime-to-host offline plugin execution boundary for cases that exceed Signal-owned stage modeling

### Batch 7.5 - Delegated Execution Receipts

- [x] define the runtime-to-host delegated offline plugin execution request/result receipt contract for stages that cannot stay inside the Signal-owned stage model
- [x] fold manifest delivery and delegated-execution outcomes into one downstream-ready runtime bundle without reintroducing host-local render ownership

### Batch 7.6 - Delegated Execution Materialization

- [x] drive delegated execution request materialization and receipt application through one end-to-end runtime-owned render handoff
- [x] prove delegated-stage report/manifest export without requiring host-specific supervisor parsing or a parallel offline bundle

### Batch 7.7 - Delegated Executor Bridge

- [x] define the runtime-owned delegated executor output/merge contract for host-only plugin stages that can affect offline audio parity
- [x] prove one delegated executor fixture can feed runtime-owned finalization without creating a parallel export pipeline

### Batch 7.8 - Host Adapter Integration

- [x] wire one concrete delegated executor adapter against the runtime-owned request/outcome contract without reconstructing offline reports or manifests in host code
- [x] prove host-side delegated execution can round-trip through runtime preparation, merge, and finalization on the same delivery bundle

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
- 2026-03-12: advanced Batch 7.3 by adding runtime-owned offline artifact and
  report receipts, optional artifact/report materialization under a runtime
  request-owned root path, export sample-rate conversion for main/stem/freeze
  buffers, and focused proof that downstream hosts can consume those receipts
  without becoming the render engine.
- 2026-03-12: completed Batch 7.3 by broadening offline media decode beyond
  WAV through runtime-owned cache decoding, tightening cached plugin override
  use to fresh latest-block captures only, and proving that stale live plugin
  renders fall back to the Signal-owned plugin stage model instead of freezing
  offline output on stale host-local state.
- 2026-03-12: completed Batch 7.4 by promoting offline artifact/report
  receipts into a typed runtime-owned manifest bundle and by exposing an
  explicit offline plugin execution boundary that tells later consumers which
  stages stay inside the Signal-owned stage model versus which ones would need
  host-delegated execution, without forcing them to parse supervisor export.
- 2026-03-12: completed Batch 7.5 by adding typed delegated offline plugin
  execution request/result receipts derived from the runtime-owned execution
  boundary and by folding those delegated outcomes back into the offline
  render manifest bundle so later consumers can carry one runtime-authored
  delivery contract.
- 2026-03-12: completed Batch 7.6 by routing delegated execution receipt
  application back through the same runtime-owned artifact/report
  materialization path, so delegated-stage report export and manifest delivery
  stay aligned without host-specific supervisor parsing or a parallel offline
  bundle.
- 2026-03-12: completed Batch 7.7 by adding a runtime-owned delegated
  executor outcome/merge contract and by proving a delegated executor fixture
  can replace main-mix, stem, and freeze outputs before runtime-owned
  finalization rewrites the same artifact/report delivery bundle.
- 2026-03-12: completed Batch 7.8 by wiring `signal-host-local` through the
  runtime-owned delegated request/outcome surface, delegating offline receipt
  and merge handling back into `signal-runtime`, and proving a concrete host
  adapter can round-trip delegated execution through runtime preparation,
  manifest/report rewrite, and artifact finalization without rebuilding host-
  local export surfaces.

## Acceptance Criteria

- [x] Signal owns one credible offline render/freeze/stem substrate
- [x] offline results align with live engine timing and processing semantics
- [x] hosts can consume render results without becoming the render engine

## Risks and Mitigations

- Risk: offline render becomes a second engine with diverging semantics.
- Mitigation: route render through the same typed timing, clip, and chain contracts.

## Evidence Requirements

- [x] log the render/freeze tranche
- [x] run focused validation for offline render parity and artifact metadata
- [x] record any intentionally deferred distributed-render scope

## Next Task

Open `g03.008` with Batch 8.1 by defining the first runtime-owned profiling
and soak harness contracts on top of the finished offline render and delegated
execution substrate, starting with reusable execution-timing receipts rather
than host-local benchmark output.
