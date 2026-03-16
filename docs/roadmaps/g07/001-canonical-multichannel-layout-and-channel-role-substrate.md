# 001 - Canonical Multichannel Layout And Channel-Role Substrate

Status: active
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

- [x] define canonical layouts, channel roles, and custom-layout fallback rules
- [x] align the contract with existing graph, hardware, and plugin receipts

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

## Batch 1.1 Outcome

Batch 1.1 freezes the first reusable multichannel vocabulary in
`docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md`.
That contract makes canonical layouts, channel roles, bus intent, and
custom-layout fallback Signal-owned meaning instead of leaving later sidechain,
spatial, Linux, and complex plugin-I/O work to infer semantics from raw
channel counts.

It also gives Batch 1.2 one fixed target:

- canonical layouts now include `Mono`, `Stereo`, `Lcr`, `Quad`,
  `Surround5_0`, `Surround5_1`, and `Surround7_1`
- channel roles and bus intents now have explicit bounded shared vocabularies
- `ChannelLayout::Count` is still valid as a primitive, but no longer carries
  enough meaning to stand in for multichannel semantics by itself
- custom layouts must now stay explicit and conservative through
  `Discrete(index)` fallback instead of being guessed into a surround mapping

## Next Task

Continue `g07.001` with Batch 1.2 by threading the canonical multichannel
layout and channel-role meaning through runtime-owned topology, hardware, and
plugin-facing receipts before the public proof batch closes the milestone.
