# g03.002 - Routed Metering And Observation Export Closure

Date: 2026-03-12
Owner: core-product
Roadmap: `docs/roadmaps/g03/002-runtime-metering-loudness-and-diagnostics-export-depth.md`

## Summary

Closed `g03.002` by attaching a typed routed metering export to the explicit
mixer-topology seam opened in `g03.001`.

Implemented in this tranche:

- `RuntimeMeteringSnapshot` now carries routed track-lane, bus-group,
  console-group, and send/return summaries in addition to flat meter sources
  and main-output loudness fields.
- meter accumulation remains realtime-safe and flat inside
  `RuntimeMeteringStateModel`; routed aggregation is derived later from
  `RuntimeExecutionTopologySummary` when runtime observation/export surfaces are
  requested.
- `RuntimeObservationReport` and `RuntimeSupervisorReport` now capture and
  render metering state in compact, multiline, and JSON exports.
- host-owned observation JSON now exposes the same runtime metering snapshot
  instead of forcing hosts to infer meter ownership from unrelated graph fields.

## Evidence

Focused proofs landed in:

- `crates/signal-runtime/src/runtime.rs`
  - `runtime_scheduler_topology_projects_into_runtime_reports`
  - `runtime_execution_topology_summarizes_send_return_routes_explicitly`
- `crates/signal-host-local/src/host.rs`
  - `local_host_shared_report_surfaces_topology_aware_host_io`

Those checks prove:

- routed meter vocabularies survive through runtime observation capture
- loudness-oriented fields remain exported beside routed peak/RMS summaries
- send/return ownership is visible in supervisor-facing metering export
- host-facing JSON can consume metering ownership directly from Signal runtime

## Deferred Scope

Still deferred on purpose:

- no mastering-grade offline report suite
- no per-route LUFS computation yet; routed summaries currently aggregate
  peak/RMS, latency, tail, bus ownership, and producer ownership while
  loudness remains a main-output/global runtime export

## Validation

Passed:

- `cargo fmt --all`
- `cargo test -p signal-runtime`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

Known external blocker, unchanged by this tranche:

- `cargo test -p signal-host-local` still fails because `LocalRuntimeHost`
  does not yet implement the newer `RuntimeSupervisorApi` methods
  `start_recording_capture`, `finish_recording_capture`,
  `cancel_recording_capture`, `reconcile_media_assets`, and
  `reconcile_warp_clips`

## Next Task

Execute `g03.003` by defining the first automation playback contract tranche:
expand reusable segment/smoothing/target semantics, then prove deterministic
multi-block automation playback through `signal-graph` and `signal-runtime`.
