# 001 - Canonical Multichannel Layout And Channel-Role Substrate

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g06.011, g06.015
Vision tags: `ROUTING`, `MULTICHANNEL`, `CONTRACTS`

## Problem

Signal currently carries channel counts and some layout meaning, but Loophole's
next routing and spatial depth needs one reusable multichannel vocabulary for
layout, channel roles, bus intent, and safe fallback behavior.

## Goals

- [ ] define canonical multichannel layout and channel-role meaning
- [ ] align graph, plugin, hardware, and render surfaces to one layout substrate
- [ ] keep hosts observing runtime layout truth instead of inventing their own mapping

## Non-Goals

- [ ] no product-specific mixer UX or speaker-visualization work
- [ ] no final immersive-format certification surface yet

## Execution Plan

### Batch 1.1 - Layout Contract

- [ ] define canonical layouts, channel roles, and custom-layout fallback rules
- [ ] align the contract with existing graph, hardware, and plugin receipts

### Batch 1.2 - Runtime Alignment

- [ ] thread the new layout and role meaning through runtime-owned snapshots
- [ ] keep adapter and host consumers on the same channel-role vocabulary

### Batch 1.3 - Public Proof

- [ ] add focused proof that downstream consumers can inspect multichannel truth
  without host-local reinterpretation

## Acceptance Criteria

- [ ] Signal has one explicit multichannel layout and channel-role substrate
- [ ] later sidechain, spatial, and plugin-I/O work can build on the same base
- [ ] hosts no longer need to infer channel meaning from raw counts alone

## Risks And Mitigations

- Risk: layout semantics stay too abstract for execution work.
- Mitigation: map them directly onto runtime, plugin, and hardware receipts.

## Evidence Requirements

- [ ] log each meaningful multichannel tranche
- [ ] run focused contract validation and public-boundary proof
- [ ] record deferred layout cases explicitly

## Next Task

Continue `g07.002` by applying the new layout substrate to sidechain and
secondary-input execution depth.

