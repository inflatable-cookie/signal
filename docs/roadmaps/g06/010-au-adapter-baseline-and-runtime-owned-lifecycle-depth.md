# 010 - AU Adapter Baseline And Runtime-Owned Lifecycle Depth

Status: planned
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

- [ ] map AU-specific details onto the shared capability and lifecycle contract
- [ ] record explicit contract gaps before deeper runtime realization

### Batch 10.2 - Runtime AU Baseline

- [ ] add the first AU adapter path with runtime-owned discovery, lifecycle,
  transport/session integration, and failure receipts
- [ ] keep supervisor export and host-edge surfaces aligned with the new path

### Batch 10.3 - Conformance Proof

- [ ] add focused proofs showing the AU path remains consumable through
  Signal-owned runtime/export surfaces without private host glue

## Acceptance Criteria

- [ ] Signal has a real AU adapter baseline
- [ ] AU behavior aligns with the shared plugin contract rather than product
  wrappers
- [ ] bounded AU/CLAP/VST3 scope is now materially real

## Risks And Mitigations

- Risk: AU support turns into a host-local macOS wrapper.
- Mitigation: keep discovery, lifecycle, and failure receipts runtime-owned.
- Risk: AU-specific quirks widen the public contract accidentally.
- Mitigation: classify AU-private details explicitly before promotion.

## Evidence Requirements

- [ ] log each meaningful AU tranche
- [ ] run focused validation for runtime-owned AU discovery/lifecycle/export
- [ ] record explicit deferred AU breadth that remains out of scope

## Next Task

Continue `g06.011` by reconciling the now-wider CLAP, VST3, and AU paths into
one stronger cross-adapter capability and conformance story.
