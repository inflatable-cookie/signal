# 001 - Canonical Multichannel Layout And Channel-Role Substrate

Status: complete
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

- [x] thread the new layout and role meaning through runtime-owned snapshots
- [x] keep adapter and host consumers on the same channel-role vocabulary

### Batch 1.3 - Public Proof

- [x] add focused proof that downstream consumers can inspect multichannel truth
  without host-local reinterpretation

## Acceptance Criteria

- [x] Signal has one explicit multichannel layout and channel-role substrate
- [x] later sidechain, spatial, and plugin-I/O work can build on the same base
- [x] hosts no longer need to infer channel meaning from raw counts alone

## Risks And Mitigations

- Risk: layout semantics stay too abstract for execution work.
- Mitigation: map them directly onto runtime, plugin, and hardware receipts.

## Evidence Requirements

- [x] log each meaningful multichannel tranche
- [x] run focused contract validation and public-boundary proof
- [x] record deferred layout cases explicitly

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

## Batch 1.2 Outcome

Batch 1.2 turns that vocabulary into real runtime-owned receipts:

- execution-topology planned-node and summarized-node receipts now carry raw
  layout, canonical layout, channel-role, and bus-intent meaning
- host hardware and external-I/O receipts now expose explicit input and output
  channel truth plus canonical multichannel summaries
- plugin discovery and plugin-chain stage receipts now surface default
  multichannel input and output meaning instead of leaving it adapter-local
- focused runtime and shared-host tests now prove the widened receipt family is
  real before the public boundary batch lands

This closes runtime alignment and leaves the last milestone step as explicit
public proof instead of more DTO churn.

## Batch 1.3 Outcome

Batch 1.3 closes the shared consumer seam for the new multichannel substrate:

- public runtime proof now verifies canonical layout, channel-role, bus-intent,
  and default plugin multichannel I/O truth through runtime-owned reexports
- stable local and server host edges now prove the same multichannel receipts
  remain consumable through `supervisor_report()` without host-local
  reinterpretation
- `signal-supervisor-tools --describe-multichannel-boundary` and
  `effigy acceptance:multichannel-boundary` now provide the machine-readable
  descriptor and repo-owned acceptance seam for this boundary
- `g07.001` is closed, so sidechain and secondary-input work can build on one
  proven canonical multichannel substrate instead of reopening layout meaning
  or proof shape

This milestone now closes as a meaningful substrate batch rather than another
DTO-only increment.

## Next Task

Continue `g07.002` with Batch 2.2 by materializing runtime-owned sidechain
source, target, attachment-policy, and fallback receipts across live and
offline routing surfaces without reopening host-local routing ownership.
