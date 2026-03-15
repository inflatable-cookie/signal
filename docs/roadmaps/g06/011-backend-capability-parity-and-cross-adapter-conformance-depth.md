# 011 - Backend Capability Parity, Linux Plugin Support, And Cross-Adapter Conformance Depth

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g06.009, g06.010
Vision tags: `PLUGINS`, `BACKENDS`, `CONFORMANCE`

## Problem

Adding VST3 and AU baselines is not enough by itself. Signal still needs one
cross-adapter capability and conformance surface so consumers know which plugin
behaviors are genuinely portable and which remain adapter-private.

## Goals

- [ ] define cross-adapter capability parity expectations across CLAP, VST3,
  and AU
- [ ] make Linux-hosted plugin support explicit where CLAP or VST3 are
  expected to carry it, instead of leaving Linux platform breadth as an
  unstated side effect of adapter work
- [ ] make runtime-owned portability and fallback behavior explicit
- [ ] keep format-scoped plugin isolation behavior explicit where policy rules
  depend on adapter identity such as CLAP, VST3, or AU
- [ ] keep discovery, lifecycle, render, and failure semantics coherent across
  the widened adapter set

## Non-Goals

- [ ] no feature-matrix marketing artifact detached from runtime reality
- [ ] no product-local fallback rules

## Execution Plan

### Batch 11.1 - Capability Parity Contract

- [x] define the portable capability and fallback matrix across CLAP, VST3, and AU
- [x] classify what remains adapter-private after the widened baseline

### Batch 11.2 - Runtime Parity Depth

- [x] align discovery, lifecycle, render, and failure receipts across adapters
- [x] align adapter identity with the shared placement-policy surface so
  by-format isolation remains runtime-owned rather than host-invented
- [x] add explicit platform-coverage and unsupported-platform reporting where
  Linux differs from macOS or Windows adapter breadth
- [x] keep supervisor export and host-edge surfaces on one cross-adapter vocabulary

### Batch 11.3 - Cross-Adapter Proof

- [x] add focused proofs that the widened adapter set stays consumable through
  Signal-owned capability and export surfaces

## Acceptance Criteria

- [ ] Signal has an explicit cross-adapter capability parity surface
- [ ] Linux plugin support is explicit and inspectable at the same consumer
  boundary as format breadth
- [ ] by-format isolation policy remains explicit and reusable across the
  widened adapter set
- [ ] wider plugin support does not reopen host-local ownership
- [ ] later consumers can rely on one portable capability vocabulary

## Risks And Mitigations

- Risk: parity work devolves into adapter-specific edge-case sprawl.
- Mitigation: freeze one bounded portable capability/fallback contract first.
- Risk: consumers overread unsupported parity claims.
- Mitigation: require explicit runtime-owned fallback and unsupported-state receipts.

## Evidence Requirements

- [ ] log each meaningful parity tranche
- [ ] run focused cross-adapter conformance validation
- [ ] record explicit unsupported parity that remains out of scope

## Batch 11.1 Outcome

Batch 11.1 froze the first bounded cross-adapter parity contract in
`docs/contracts/022-backend-capability-parity-linux-plugin-support-and-cross-adapter-conformance-contract.md`.
The repo now has an explicit rule set for:

- separating portable, format-guarded, adapter-private, and unsupported scope
  across CLAP, VST3, and AU
- making Linux plugin breadth explicit as a guarded runtime-owned claim instead
  of an unstated side effect of adapter existence
- keeping parity authority inside `signal-plugin`, `signal-runtime`, and
  additive Signal-owned receipts rather than host-local portability matrices
- giving Batch 11.2 one fixed portability target before deeper runtime
  discovery, lifecycle, render, and platform-coverage receipt work begins

## Batch 11.2 Outcome

Batch 11.2 turned the parity contract into a real runtime-owned receipt family:

- `signal-runtime` now carries typed per-format parity coverage beside the
  existing discovery and lifecycle snapshots instead of leaving platform scope,
  placement-policy alignment, and failure counts implicit
- hosts now seed runtime-owned platform coverage for CLAP, VST3, and AU so
  Linux breadth and AU macOS scope are inspectable through the same receipt
  vocabulary
- discovery, lifecycle, render-readiness, and failure counts now align on one
  cross-adapter parity record through runtime, supervisor export, and stable
  host-edge reports
- Batch 11.3 can now focus on proving the widened parity receipt family is
  consumable rather than inventing a new vocabulary

## Batch 11.3 Outcome

Batch 11.3 closes the bounded cross-adapter parity milestone:

- public `signal-runtime` proofs now consume the widened CLAP, VST3, and AU
  parity receipt family directly through shared discovery and lifecycle
  surfaces
- both stable host edges now prove they forward the same parity coverage and
  platform-scope truth on `supervisor_report()` without host-local portability
  matrices
- `signal-supervisor-tools` now exposes a machine-readable
  `signal.runtime.cross-adapter-parity-boundary` descriptor and repo-owned
  `effigy acceptance:cross-adapter-parity-boundary` task
- `g06.012` can now widen generic MIDI, note-expression, and plugin-event
  depth on top of one closed cross-adapter capability baseline instead of
  reopening format-breadth ownership

## Next Task

Continue `g06.012` with Batch 12.1 by freezing the widened generic MIDI,
note-expression, and plugin-event vocabulary across CLAP, VST3, and AU before
runtime and adapter event-depth work begins.
