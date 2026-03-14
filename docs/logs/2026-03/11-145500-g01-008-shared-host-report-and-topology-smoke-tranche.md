---
title: g01.008 shared host report and topology smoke tranche
status: complete
owner: core-product
created: 2026-03-11
updated: 2026-03-11
tags: [signal, hardware, host, runtime, roadmap, g01, g01.008]
---

## Summary

Completed the first `008.3` diagnostics tranche by adding a shared runtime/host
report surface for host I/O state and then validating it against the existing
topology-aware local host path in both steady-state and timeout-recovery cases.

## What landed

- added shared host-report types in `signal-runtime`:
  - `RuntimeHostHardwareSummary`
  - `RuntimeHostAudioPumpSummary`
  - `RuntimeHostIoSummary`
  - `RuntimeHostObservationReport`
  - `RuntimeHostSupervisorReport`
- `signal-host-local` now maps its negotiated hardware contract plus output pump
  state into that shared report layer instead of forcing consumers to stitch
  together `LocalRuntimeHostSummary`, runtime observation output, and topology
  metadata separately
- the new shared host report explicitly carries:
  - hardware/backend identity and negotiated stream shape
  - backend diagnostic counters and health
  - host pump transfer counts and stream state
  - graph-id consistency between the host pump and runtime engine snapshot
- `signal-host-local` now exposes:
  - `host_observation_report()`
  - `host_supervisor_report()`
- the `signal-host-local` CLI now renders the shared host supervisor report
  compactly instead of manually flattening host/runtime diagnostics

## Validation surface

- added a steady-state shared-report test proving the topology-aware local host
  path still exports the intended track/bus/output shape through the shared host
  report
- added a timeout-recovery shared-report test proving:
  - timeout-driven xrun state is visible in the shared report
  - the host pump remains running after recovery
  - the topology-aware track/bus/output summary remains intact after recovery

## Contract outcome

- `LocalRuntimeHostSummary` remains justified as the boot/result summary for the
  local host shell
- richer host/device diagnostics now live in a shared runtime-owned report shape
  that other host or supervisor surfaces can consume without reinterpreting the
  topology-aware local host path
- this tranche does not yet simulate device loss or backend restart failure at
  the CoreAudio trust edge; that remains the open `008.3` item

## Validation

- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-host-local local_host_shared_report_surfaces_topology_aware_host_io -- --nocapture`
- `cargo test -p signal-host-local local_host_shared_report_tracks_timeout_recovery_without_losing_topology -- --nocapture`
- `cargo test -p signal-runtime`
- `cargo test -p signal-host-local`
- `effigy health`
- `effigy validate`
- `effigy test`

## Next

Finish the remaining `008.3` device-loss and restart-handling work by adding
explicit host fault injection or simulated backend-diagnostic transitions on top
of the shared host report surface, then validate that restart attempts,
restart failures, and device-loss state stay coherent with runtime-owned
transport and degradation reporting.
