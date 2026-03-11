---
title: g01.007 transport truth and engine clock tranche
status: complete
owner: core-product
updated: 2026-03-11
tags: [signal, runtime, transport, scheduler, roadmap, g01.007]
---

## Summary

Landed the first `g01.007` runtime tranche in `signal-runtime`.

The batch makes runtime the explicit owner of transport transition semantics and
block-clock reporting for engine execution:

- added runtime-owned transport transition kinds for start, stop, seek, tempo
  change, loop-state change, and loop wrap
- promoted transport epoch and transport/block-window reporting into
  `RuntimeTimelineSnapshot` and `RuntimeEngineBlockSnapshot`
- classified transport-driven prework invalidation with specific reasons instead
  of the old generic `TransportChanged` bucket
- recorded block start/end transport sample windows and loop-wrap state at the
  runtime seam during `process_engine_block(...)`
- exposed the new transport state in compact, multiline, and JSON runtime
  observation output
- added focused tests for transport invalidation classification, transport
  progression visibility, and loop-wrap reporting

## Files

- `crates/signal-runtime/src/interfaces.rs`
- `crates/signal-runtime/src/runtime.rs`
- `docs/roadmaps/g01/007-runtime-transport-scheduler-and-engine-processing-baseline.md`

## Validation

Passed:

- `cargo test -p signal-runtime --no-run`
- focused runtime transport tests added in this tranche passed as part of the
  crate test run:
  - `runtime_classifies_transport_invalidation_boundaries`
  - `runtime_records_transport_progression_in_timeline_and_engine_snapshot`
  - `runtime_records_loop_wrap_as_transport_boundary`

Mixed:

- `cargo test -p signal-runtime`
  - the new transport-focused tests passed
  - broader runtime forecast/prework tests and a few observation summary tests
    failed in the same run and need follow-up before `g01.007` closure

## Notes

The main unresolved risk is spillover into the older forecast/prework test
surface. The transport truth work is coherent on its own seam, but the broader
runtime crate still needs a cleanup pass so forecast-state application and
transport-authority logic do not fight over block-sequence and snapshot
assumptions.

## Next

Use this transport epoch and block-clock baseline to start `007.2`, but only
after a short stabilization pass that reconciles forecast/prework tests with the
new transport-authority model so scheduler enforcement can build on a clean
runtime baseline.
