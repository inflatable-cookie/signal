# 2026-03-16 - g06.017 Batch 17.2 runtime media-service baseline

## Summary

Turned the frozen `g06.017` media-service contract into a real shared runtime
surface by wiring media pipeline and media service state into runtime
observation, supervisor export, and shared host-report paths.

## Work completed

- widened `RuntimeObservationReport` to carry:
  - `media_pipeline_snapshot`
  - `media_service_snapshot`
- extended `RuntimeSupervisorReport` JSON and multiline rendering to export the
  same media indexing, invalidation, waveform, and preview receipts
- aligned shared host report JSON with the same runtime-owned media-service
  snapshots instead of leaving media state as a direct-API-only subsystem
- added focused runtime, local-host, and server-host tests for the widened
  media-service report path

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_observation_and_supervisor_reports_surface_media_service_baseline -- --nocapture`
- `cargo test -p signal-host-local local_host_shared_report_surfaces_runtime_media_service_baseline -- --nocapture`
- `cargo test -p signal-host-server server_host_shared_report_surfaces_runtime_media_service_baseline -- --nocapture`

## Deferred scope

Batch 17.2 still does not claim:

- a public consumer-boundary descriptor or acceptance task for the media seam
- richer metadata extraction or library-service breadth
- product-local browser or catalog workflows

## Next Task

Continue `g06.018` with Batch 18.1 by freezing the first reusable
analysis-metadata and library-service descriptor family on top of the closed
media-service boundary.
