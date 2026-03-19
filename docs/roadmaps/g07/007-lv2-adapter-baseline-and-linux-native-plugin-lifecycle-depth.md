# 007 - LV2 Adapter Baseline And Linux-Native Plugin Lifecycle Depth

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g06.011
Vision tags: `PLUGINS`, `LINUX`, `LV2`

## Problem

Chorus still points to LV2 as part of the Linux plugin story, but Signal's
current planned breadth stops at CLAP, VST3, and AU.

## Goals

- [ ] introduce the first real LV2 adapter baseline
- [ ] make Linux-native plugin breadth explicit rather than implied
- [ ] keep lifecycle, capability, and sandbox meaning aligned with the existing
  backend-neutral plugin contract

## Non-Goals

- [ ] no Linux-only product workflow behavior
- [ ] no adapter-private behavior promoted accidentally

## Execution Plan

### Batch 7.1 - LV2 Contract Alignment

- [x] map LV2-specific details onto the backend-neutral capability and lifecycle contract
- [x] record explicit contract gaps before runtime realization widens

### Batch 7.2 - Runtime Adapter Baseline

- [x] add the first LV2 adapter path with runtime-owned discovery, lifecycle, and transport integration
- [x] keep supervisor export and host-edge surfaces aligned with the new path

### Batch 7.3 - Conformance Proof

- [x] add focused proofs for Linux-native LV2 discovery, lifecycle, and export behavior

## Acceptance Criteria

- [ ] Signal has a real LV2 adapter baseline
- [ ] Linux-native plugin breadth is explicit at the runtime boundary
- [ ] LV2 lifecycle and capability surfaces align with the shared contract

## Risks And Mitigations

- Risk: LV2 work reopens adapter-private ownership.
- Mitigation: force widened behavior through the backend-neutral contract.

## Evidence Requirements

- [ ] log each meaningful LV2 tranche
- [ ] run focused LV2 discovery, lifecycle, and export validation
- [ ] record deferred LV2 breadth explicitly

## Batch 7.1 Outcome

Batch 7.1 freezes the first LV2 adapter alignment boundary in
`docs/contracts/038-lv2-adapter-baseline-and-linux-native-plugin-lifecycle-contract.md`.

Signal now has one explicit contract for:

- how LV2 discovery, bundle or manifest traversal, URI identity, and Linux-native
  support must collapse into shared runtime-owned discovery receipts
- how LV2 lifecycle and continuity must reuse the existing backend-neutral
  plugin contract family instead of creating a Linux-only wrapper taxonomy
- which gaps remain real before runtime realization widens, including the lack
  of shared LV2 backend identity, a Rust LV2 adapter crate, and runtime-owned
  Linux-native scan or load receipts

That gives Batch 7.2 one fixed target for real adapter work without drifting
into host-local Linux plugin ownership or adapter-private lifecycle semantics.

## Batch 7.2 Outcome

Batch 7.2 turns the contract into a real Linux-native runtime path.

Signal now has:

- a shared `PluginFormat::Lv2` backend identity and a real
  `crates/signal-plugin-lv2` adapter crate instead of only roadmap intent
- Linux-native LV2 scan roots, URI identity, manifest-path, capability, and
  session-planning fixtures that collapse into the existing runtime-owned
  discovery receipts rather than a host-private catalog
- server-host scan and sandbox wiring that records LV2 discovered-type,
  lifecycle, instance-state, transport, and parity receipts through the same
  runtime-owned surfaces already used by the other plugin formats
- explicit Linux-only platform coverage for LV2 so runtime-owned parity and
  platform scope no longer imply Linux plugin breadth indirectly

That closes the runtime adapter baseline and leaves Batch 7.3 a narrower proof
job instead of more realization work.

## Batch 7.3 Outcome

Batch 7.3 closes the shared LV2 consumer seam.

Signal now has:

- a public runtime proof that Linux-native LV2 discovery, lifecycle, transport,
  and Linux-only platform scope remain consumable through shared runtime
  observation and supervisor reports
- a stable server host-edge proof that Linux LV2 discovery and sandbox truth
  remain visible on supervisor export without adapter-local reconstruction
- a machine-readable `signal.runtime.lv2-boundary` descriptor and repo-owned
  acceptance lane so downstream consumers can inspect the shared proof seam
  directly

That closes `g07.007` as a bounded LV2 baseline and moves the active queue to
Linux cross-adapter parity and sandbox policy depth.

## Next Task

Continue `g07.008` with Batch 8.2 by aligning lifecycle, render, failure, and
placement receipts across Linux adapters so supervisor export and stable
host-edge surfaces stay on one Linux plugin vocabulary.
