# 010 - AU Adapter Baseline And Runtime-Owned Lifecycle Depth

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g05.001, g06.003, g06.009
Vision tags: `PLUGINS`, `BACKENDS`, `AU`

## Problem

Chorus still explicitly cares about bounded AU/CLAP/VST3 support, but Signal
does not yet have a real AU adapter path. The next feature-depth runway needs a
runtime-owned AU baseline instead of leaving AU as package-map intent only.

## Goals

- [ ] introduce the first real AU adapter baseline inside Signal-owned crates
- [ ] keep AU lifecycle, discovery, capability, and failure meaning aligned
  with the backend-neutral plugin contract
- [ ] avoid product-local AU wrappers becoming the source of truth

## Non-Goals

- [ ] no Audio Unit UI/window management product work
- [ ] no macOS-only host convenience surface promoted by accident
- [ ] no claim that AU is a Linux-capable path; AU remains macOS-scoped even as
  broader Linux plugin support lands through other adapters

## Execution Plan

### Batch 10.1 - AU Contract Alignment

- [x] map AU-specific details onto the shared capability and lifecycle contract
- [x] record explicit contract gaps before deeper runtime realization

### Batch 10.2 - Runtime AU Baseline

- [x] add the first AU adapter path with runtime-owned discovery, lifecycle,
  transport/session integration, and failure receipts
- [x] keep supervisor export and host-edge surfaces aligned with the new path

### Batch 10.3 - Conformance Proof

- [x] add focused proofs showing the AU path remains consumable through
  Signal-owned runtime/export surfaces without private host glue

## Acceptance Criteria

- [x] Signal has a real AU adapter baseline
- [x] AU behavior aligns with the shared plugin contract rather than product
  wrappers
- [x] bounded AU/CLAP/VST3 scope is now materially real

## Risks And Mitigations

- Risk: AU support turns into a host-local macOS wrapper.
- Mitigation: keep discovery, lifecycle, and failure receipts runtime-owned.
- Risk: AU-specific quirks widen the public contract accidentally.
- Mitigation: classify AU-private details explicitly before promotion.

## Evidence Requirements

- [x] log each meaningful AU tranche
- [x] run focused validation for runtime-owned AU discovery/lifecycle/export
- [x] record explicit deferred AU breadth that remains out of scope

## Batch 10.1 Outcome

Batch 10.1 froze the first AU-specific alignment contract in
`docs/contracts/021-au-adapter-baseline-and-runtime-owned-lifecycle-contract.md`.
The repo now has an explicit rule set for:

- keeping AU capability, discovery, lifecycle, and failure meaning inside the
  existing backend-neutral Signal-owned contract family
- treating AudioComponent traversal, subtype or manufacturer filtering,
  property negotiation, and instance bring-up as adapter-private realization
  detail rather than a second shared lifecycle system
- making macOS-scoped AU scan and load coverage explicit work for Batch 10.2
  rather than package-map intent
- recording the current realization gaps before the runtime adapter baseline
  lands
- keeping product-local AU wrappers outside the authority chain so the runtime
  and future `signal-plugin-au` adapter remain the source of truth

## Batch 10.2 Outcome

Batch 10.2 landed the first real runtime-owned AU adapter baseline:

- added a new `signal-plugin-au` crate with bounded Audio Unit discovery
  metadata, macOS component roots, component identity, and shared-memory
  session planning
- wired `signal-host-local` and `signal-host-server` to feed AU discovery
  through runtime-owned scan receipts instead of host-local catalogs
- made macOS-scoped AU scan and load coverage explicit through both host paths
  using Audio Unit component roots rather than package-map implication
- recorded AU sandbox bring-up through existing runtime-owned lifecycle,
  instance-state, and transport receipts so the new path extends the shared
  contract instead of forking it
- kept public-conformance proof as the next separate batch rather than mixing
  implementation and boundary-freeze work together

## Batch 10.3 Outcome

Batch 10.3 closed the AU consumer boundary on top of the new adapter baseline:

- added downstream-style public `signal-runtime` proof that AU discovery and
  lifecycle truth stay consumable through shared runtime surfaces
- added stable host-edge proofs for both `signal-host-local` and
  `signal-host-server` so AU discovery and lifecycle state flow through
  `supervisor_report()` without host-private AU ledgers
- added the machine-readable `signal.runtime.au-boundary` descriptor in
  `signal-supervisor-tools`
- added the repo-owned `effigy acceptance:au-boundary --repo .` validation seam
- closed `g06.010` and moved the plugin-breadth lane forward to backend
  capability parity instead of leaving AU proof as an implicit follow-up

## Next Task

Continue `g06.011` with Batch 11.1 by freezing the backend capability parity,
Linux plugin-support, and cross-adapter conformance contract on top of the now
closed CLAP, VST3, and AU runtime-owned adapter boundaries.
