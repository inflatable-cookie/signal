# 008 - Linux Cross-Adapter Plugin Parity And Sandbox Policy

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g07.007, g06.003
Vision tags: `PLUGINS`, `LINUX`, `CONFORMANCE`

## Problem

Adding LV2 alone is not enough. Linux consumers still need one explicit view of
which plugin behaviors are portable, how sandbox policy applies, and where
unsupported-state receipts should appear.

## Goals

- [ ] define Linux cross-adapter plugin parity across CLAP, VST3, and LV2
- [ ] align Linux plugin breadth with the shared sandbox and placement-policy model
- [ ] keep runtime-owned portability and fallback behavior explicit

## Non-Goals

- [ ] no marketing feature matrix detached from runtime reality
- [ ] no product-local fallback or scan policy

## Execution Plan

### Batch 8.1 - Linux Parity Contract

- [x] define portable capability, fallback, and sandbox-policy expectations on Linux
- [x] classify what remains adapter-private after the widened baseline

### Batch 8.2 - Runtime Parity Depth

- [x] align lifecycle, render, failure, and placement receipts across Linux adapters
- [x] keep supervisor export and host-edge surfaces on one Linux plugin vocabulary

### Batch 8.3 - Cross-Adapter Proof

- [x] add focused proofs for Linux plugin parity and sandbox-policy behavior

## Acceptance Criteria

- [ ] Signal has an explicit Linux cross-adapter parity surface
- [ ] sandbox policy remains reusable across the widened Linux adapter set
- [ ] later consumers can rely on one portable Linux plugin vocabulary

## Risks And Mitigations

- Risk: Linux parity work devolves into adapter sprawl.
- Mitigation: freeze one bounded portable contract first.

## Evidence Requirements

- [ ] log each meaningful Linux parity tranche
- [ ] run focused Linux cross-adapter conformance validation
- [ ] record explicit unsupported Linux parity explicitly

## Batch 8.1 Outcome

Batch 8.1 freezes the bounded Linux plugin parity and sandbox-policy contract
in `docs/contracts/039-linux-cross-adapter-plugin-parity-and-sandbox-policy-contract.md`.

Signal now has one explicit Linux-facing contract for:

- what counts as portable, guarded, adapter-private, and unsupported across
  CLAP, VST3, and LV2 on Linux
- how Linux sandbox and placement-policy meaning must reuse the existing
  runtime-owned shared-sandbox and continuity contract instead of creating a
  Linux-only wrapper taxonomy
- which Linux adapter behaviors still remain deferred, including richer
  extension depth and the broader ALSA, JACK, and PipeWire backend story

That gives Batch 8.2 one fixed target for runtime receipt work without drifting
into host-local portability matrices or adapter-private sandbox policy.

## Batch 8.2 Outcome

Batch 8.2 turns the Linux parity contract into a real runtime-owned receipt
family.

Signal now has:

- widened Linux parity data on the existing runtime-owned discovery and
  lifecycle surface, including Linux-specific parity band, Linux support,
  preferred sandbox outcome, strict-sandbox default, render-capable type
  counts, in-process versus shared versus isolated counts, restarting counts,
  and rebindable counts
- one runtime derivation path for those receipts so `RuntimePluginDiscoverySnapshot`,
  `RuntimePluginScanReceipt`, and `RuntimePluginLifecycleSnapshot` stay aligned
  instead of growing separate Linux-only host summaries
- Linux host coverage wired for the widened parity surface, with focused server
  proofs on VST3 and LV2 and runtime-side proof that CLAP, VST3, and LV2 now
  share one Linux-facing placement, render, and failure vocabulary

That leaves Batch 8.3 with a narrower public proof job instead of more runtime
receipt shaping.

## Batch 8.3 Outcome

Batch 8.3 closes the Linux parity boundary through shared runtime, the stable
server host edge, and one machine-readable supervisor-tools descriptor.

Signal now has:

- a focused downstream-style runtime proof that Linux-specific parity band,
  Linux support, preferred sandbox outcome, strict-sandbox default, restart,
  rebindability, and failure posture remain consumable through public runtime
  observation and supervisor surfaces
- a stable Linux-facing server host-edge proof that the same runtime-owned
  Linux plugin vocabulary survives `supervisor_report()` without host-local
  Linux portability matrices
- a machine-readable `signal.runtime.linux-plugin-parity-boundary` surface and
  repo-owned `effigy acceptance:linux-plugin-parity-boundary` lane so later
  consumers and roadmap work can inspect the proof seam directly

That closes `g07.008` on one bounded Linux plugin vocabulary and leaves
`g07.009` to widen Linux hardware backend portability rather than reopening
adapter parity meaning.

## Next Task

Continue `g07.009` with Batch 9.1 by freezing the runtime-owned Linux audio
backend portability contract across ALSA, JACK, and PipeWire on top of the
now-closed Linux plugin parity and sandbox-policy boundary.
