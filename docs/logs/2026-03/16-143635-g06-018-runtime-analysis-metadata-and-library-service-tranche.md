# g06.018 Batch 18.2 - Runtime Analysis Metadata And Library-Service Tranche

Date: 2026-03-16
Roadmap: `docs/roadmaps/g06/018-analysis-metadata-extraction-and-library-service-depth.md`
Contract: `docs/contracts/029-analysis-metadata-extraction-and-library-service-contract.md`

## Summary

Materialized the first real runtime-owned analysis-metadata and library-service
descriptor family on top of the closed `g06.017` media-service seam.
`signal-runtime` now derives bounded loudness and character descriptors during
media reconciliation, exports them through observation and supervisor reports as
`RuntimeMediaLibraryServiceSnapshot`, and keeps the same typed descriptor family
visible on both shared host report paths.

## Delivered

- added runtime-owned descriptor state and family-coverage DTOs:
  - `RuntimeMediaLibraryServiceSnapshot`
  - `RuntimeMediaLibraryAssetDescriptor`
  - `RuntimeMediaLoudnessDescriptor`
  - `RuntimeMediaCharacterDescriptor`
- widened the media pipeline state model so analysis metadata is:
  - `Ready` for decodable analyzed assets
  - `Invalidated` when media truth breaks
  - `Unavailable` when indexed media remains non-analyzable
- made loudness and character the first real payload families
- kept rhythm, tonal, and embedding explicitly `Deferred`
- aligned runtime observation, supervisor export, and shared local/server host
  reports to the same library-service descriptor family

## Validation

- `cargo fmt --all`
- `effigy test --plan --repo .`
  - known repo constraint: still falls through to unsupported `ctest --plan`
- `cargo test -p signal-runtime runtime_media_service_snapshot_tracks_ready_previewable_and_invalidated_assets -- --nocapture`
- `cargo test -p signal-runtime runtime_observation_and_supervisor_reports_surface_media_service_baseline -- --nocapture`
- `cargo test -p signal-host-local local_host_shared_report_surfaces_runtime_media_service_baseline -- --nocapture`
- `cargo test -p signal-host-server server_host_shared_report_surfaces_runtime_media_service_baseline -- --nocapture`
- `cargo test -p signal-supervisor-tools --no-run`

## Deferred

- public consumer-boundary proof still belongs to Batch 18.3
- richer rhythm, tonal, and embedding payload depth remains deferred
- product-local library UX, tags, and intelligence workflows remain out of
  scope
