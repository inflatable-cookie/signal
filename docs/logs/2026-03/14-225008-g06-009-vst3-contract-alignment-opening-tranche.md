# g06.009 - VST3 Contract Alignment Opening Tranche

Date: 2026-03-14
Milestone: `g06.009`
Batch: `9.1`
Status: complete

## Summary

Opened the VST3 lane with one bounded contract batch. Signal now has an
explicit VST3-specific mapping onto the existing backend-neutral plugin,
lifecycle, and continuity contract family before any real adapter baseline
widens the runtime path.

## What changed

- added
  `docs/contracts/020-vst3-adapter-baseline-and-runtime-owned-lifecycle-contract.md`
- froze one VST3-specific authority chain across:
  - `signal-plugin` format-neutral capability and lifecycle meaning
  - `signal-runtime` discovery, lifecycle, continuity, and export receipts
  - future `signal-plugin-vst3` adapter realization detail
  - stable host and supervisor export surfaces as observers rather than
    reclassifiers
- mapped the main VST3-specific realization seams onto shared Signal-owned
  meaning:
  - class or category detail must collapse into runtime-owned discovery
    receipts
  - component or controller split remains adapter-private realization rather
    than a second shared lifecycle system
  - VST3 bus and event topology must widen format-neutral capability surfaces
    additively if consumer-visible depth is needed later
  - state, recall, and continuity stay runtime-owned even if VST3 realization
    requires split processor or controller handling
- made Linux-hosted VST3 scan/load coverage an explicit Batch 9.2 requirement
  rather than package-map intent
- recorded the main pre-realization gaps:
  - no Rust `signal-plugin-vst3` adapter crate yet
  - no real runtime-owned VST3 discovery/load path yet
  - no explicit Linux-hosted VST3 scan/load proof yet
  - richer VST3 unit, program-list, and event depth remains deferred

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Deferred

- the first real `signal-plugin-vst3` runtime adapter baseline
- explicit Linux-hosted VST3 discovery and load proofs
- additive shared DTOs for any VST3-specific controller or processor mismatch
  evidence
- broader cross-adapter parity work, which remains queued for later `g06`
  milestones

## Next

Continue `g06.009` with Batch 9.2 by implementing the first real VST3 adapter
path with runtime-owned discovery, lifecycle, Linux-hosted scan/load coverage,
and aligned supervisor or host-edge export.
