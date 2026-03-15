# 009 - VST3 Adapter Baseline And Runtime-Owned Lifecycle Depth

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g05.001, g06.003
Vision tags: `PLUGINS`, `BACKENDS`, `VST3`

## Problem

Signal's plugin contract is backend-neutral, but actual adapter realization is
still CLAP-first. Loophole's feature front still needs a real VST3 path inside
Signal's runtime-owned lifecycle model.

## Goals

- [ ] introduce the first real VST3 adapter baseline inside Signal-owned crates
- [ ] keep lifecycle, fault, discovery, and capability meaning aligned with the
  existing backend-neutral contract
- [ ] make the VST3 path credible on Linux as well as other supported host
  platforms rather than leaving Linux plugin support implicit
- [ ] avoid pushing VST3 ownership into product-local wrappers

## Non-Goals

- [ ] no product-specific plugin browser or preset UX
- [ ] no format-specific behavior promoted to the shared contract by accident

## Execution Plan

### Batch 9.1 - VST3 Adapter Contract Alignment

- [x] map VST3-specific details onto the existing backend-neutral capability
  and lifecycle contract
- [x] record any explicit contract gaps before runtime realization widens

### Batch 9.2 - Runtime Adapter Baseline

- [x] add the first VST3 adapter path with runtime-owned discovery, lifecycle,
  and transport/session integration
- [x] cover platform-specific scan/load paths needed for Linux-hosted VST3 use
  without changing the shared runtime contract
- [x] keep supervisor export and host-edge receipts aligned with the new path

### Batch 9.3 - Conformance Proof

- [x] add focused proofs showing the VST3 path remains consumable through
  Signal-owned runtime/export surfaces without host-local reconstruction

## Acceptance Criteria

- [x] Signal has a real VST3 adapter baseline
- [x] the VST3 path includes explicit Linux-hosted plugin coverage rather than
  only package-map intent
- [x] VST3 lifecycle and capability surfaces align with the shared contract
- [x] later cross-adapter breadth can build on runtime-owned receipts

## Risks And Mitigations

- Risk: VST3 work reopens format-specific ownership.
- Mitigation: force all widened surfaces through the existing backend-neutral contract.
- Risk: CLAP-first assumptions leak into VST3 behavior silently.
- Mitigation: require explicit conformance proof on the widened path.

## Evidence Requirements

- [x] log each meaningful VST3 tranche
- [x] run focused validation for runtime-owned VST3 discovery/lifecycle/export
- [x] record explicit deferred VST3 breadth that remains out of scope

## Batch 9.1 Outcome

Batch 9.1 froze the first VST3-specific alignment contract in
`docs/contracts/020-vst3-adapter-baseline-and-runtime-owned-lifecycle-contract.md`.
The repo now has an explicit rule set for:

- keeping VST3 capability, discovery, and lifecycle meaning inside the existing
  backend-neutral Signal-owned contract family
- treating component/controller split and module/class traversal as
  adapter-private realization detail rather than a second shared lifecycle
  system
- making Linux-hosted VST3 scan/load coverage explicit work for Batch 9.2
  rather than package-map intent
- recording the current realization gaps before the runtime adapter baseline
  lands

## Batch 9.2 Outcome

Batch 9.2 landed the first real runtime-owned VST3 adapter baseline:

- added a new `signal-plugin-vst3` crate with bounded discovery metadata,
  platform scan roots, class/controller pairing, and shared-memory session
  planning
- wired `signal-host-local` and `signal-host-server` to feed VST3 discovery
  through runtime-owned scan receipts instead of host-local catalogs
- made Linux-hosted VST3 scan/load coverage explicit through the server-host
  path using Linux VST3 roots rather than package-map implication
- recorded VST3 sandbox bring-up through existing runtime-owned lifecycle,
  instance-state, and transport receipts so the new path extends the shared
  contract instead of forking it
- kept public-conformance proof as the next separate batch rather than mixing
  implementation and boundary-freeze work together

## Batch 9.3 Outcome

Batch 9.3 closed the shared VST3 consumer boundary:

- added downstream-style public runtime proof for VST3 discovery and lifecycle
  truth through `signal-runtime` reexports alone
- proved both stable host edges forward the same VST3 truth through
  `supervisor_report()` without adapter-local reconstruction
- added a machine-readable `signal.runtime.vst3-boundary` descriptor and
  repo-owned `effigy acceptance:vst3-boundary` task so the proof stays
  inspectable and runnable
- kept deferred scope explicit: richer VST3 event, unit, and program-list depth
  still belong to later cross-adapter work rather than this baseline
- closed `g06.009` and moved the plugin breadth lane forward to AU baseline
  work in `g06.010`

## Next Task

Continue `g06.010` with Batch 10.1 by mapping AU-specific discovery,
lifecycle, and macOS-scoped capability detail onto the shared backend-neutral
plugin contract before runtime-owned AU realization widens.
