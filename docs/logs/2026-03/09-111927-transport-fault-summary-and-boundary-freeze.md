---
title: Transport Fault Summary And Boundary Freeze
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, supervisor, transport-faults]
---

# Summary

Added a shared `transport_fault_summary` to the runtime supervisor surface so
the canonical top-level transport-fault boundary is explicit in code and
exported as data, rather than only implied by which milestones appear in the
sequence.

# Changes

- Added `TransportFaultBoundaryMode` and `TransportFaultSummary` to
  `signal-runtime`.
- `RuntimeObservationReport` now carries `transport_fault_summary`, derived
  from the canonical `transport_fault_sequence`.
- The shared summary freezes the current top-level boundary as
  `FaultAdjacentOnly` and reports:
  - total transport-fault events
  - counts by source
  - counts by phase
  - first/last epoch
  - first/last block sequence
- Updated compact, multiline, and JSON supervisor rendering so the boundary
  mode and summary counts are visible without requiring downstream tooling to
  parse the full event sequence.
- Re-exported the new shared summary and boundary types from `signal-runtime`.
- Updated runtime and supervisor-tool fixtures so the summary shape is pinned
  alongside the sequence shape.
- Updated the README, package map, and supervisor export contract so the
  top-level transport-fault surface is explicitly described as fault-adjacent
  only, not a generic transport/session trace.

# Validation

- `cargo check -p signal-runtime -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-supervisor-tools --no-run`
- `cargo fmt --all`
- `git diff --check`

# Next Task

Decide whether the current `FaultAdjacentOnly` top-level transport-fault
boundary is now stable enough to treat as frozen for schema version 1, or
whether a second higher-level non-fault transport summary should be added for
healthy-path transport/session visibility without weakening the current fault
surface.
