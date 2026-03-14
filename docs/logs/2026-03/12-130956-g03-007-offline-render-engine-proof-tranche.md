# g03.007 - Offline Render Engine Proof Tranche

Date: 2026-03-12
Status: completed tranche
Roadmap: `docs/roadmaps/g03/007-offline-render-freeze-and-stem-export-pipeline.md`

## Summary

Completed the first engine-backed `g03.007` tranche in `signal-runtime`. The
runtime now owns an offline render result path that decodes runtime-cached
media, reapplies clip-processing treatment, executes the graph for main mix and
stem output, and exports freeze artifacts from the same rendered stem buffers
with recall handoff metadata attached.

## Shipped

- added runtime-owned offline render result surfaces:
  - `RuntimeOfflineRenderResult`
  - `RuntimeOfflineRenderStemResult`
  - `RuntimeOfflineFreezeArtifactResult`
- added a runtime offline render entry point that:
  - validates the existing request/preview contract
  - resolves tempo and automation by timeline block
  - decodes runtime-cached WAV media assets
  - reuses the clip-render seam for fade/gain treatment
  - executes the graph for main mix output
  - captures routed bus buffers for stem export
  - derives freeze artifacts from rendered stem buffers plus recall handoff ids
- extended `signal-graph` with requested bus capture so runtime can export stem
  buffers without inventing a second graph execution model
- added a focused runtime proof covering:
  - main mix rendering
  - track-lane stem rendering
  - freeze export from rendered stems
  - recall-handoff metadata preservation
  - no live engine-block counter churn during offline render

## Deferred

- export currently requires `request.export_sample_rate_hz` to match the runtime
  sample rate
- runtime media decode is currently WAV-only
- plugin-backed offline stages currently reuse cached render overrides instead
  of driving a dedicated offline sandbox execution path
- distributed/cloud render orchestration remains intentionally deferred and is
  still outside `g03.007`

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime`
- `git diff --check`
- `effigy health`
- `effigy test`
- `effigy validate`

## Next Task

Continue `g03.007` with artifact/parity hardening by turning the in-memory
render results into richer runtime-owned artifact/report receipts and by
closing the current sample-rate, media-format, and plugin-freshness gaps before
opening `g03.008`.
