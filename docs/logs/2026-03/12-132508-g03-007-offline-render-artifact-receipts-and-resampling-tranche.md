# g03.007 - Offline Render Artifact Receipts And Resampling Tranche

Date: 2026-03-12
Status: completed tranche
Roadmap: `docs/roadmaps/g03/007-offline-render-freeze-and-stem-export-pipeline.md`

## Summary

Advanced `g03.007` Batch 7.3 in `signal-runtime` by turning the first offline
render proof path into a runtime-owned artifact/report receipt surface. The
runtime can now optionally materialize main-mix, stem, freeze, and report
artifacts under a request-owned output root while preserving authoritative
runtime render accounting and exporting at a requested sample rate.

## Shipped

- extended runtime-owned offline render request and result surfaces with:
  - artifact root path selection
  - runtime frame count versus exported frame count
  - typed artifact receipts
  - typed report receipts
- added runtime-owned export sample-rate conversion for:
  - main mix output
  - stem output
  - freeze artifact output
- added optional runtime artifact materialization that writes:
  - WAV artifacts for main mix, stems, and freeze outputs
  - a JSON render report summarizing emitted artifacts and frame counts
- kept artifact/report ownership in runtime contracts rather than requiring
  downstream hosts to infer file layout or parse supervisor-facing export
  surfaces
- added a focused runtime proof covering:
  - offline render export at a non-runtime sample rate
  - populated artifact and report receipts
  - written artifact metadata that matches the exported sample rate
  - stable runtime-frame accounting separate from exported frame counts

## Deferred

- runtime media decode remains WAV-only
- plugin-backed offline stages still reuse cached render overrides instead of
  executing a fresher dedicated offline plugin pass
- distributed/cloud render orchestration remains intentionally outside
  `g03.007`

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime`
- `git diff --check`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`

## Next Task

Continue `g03.007` with the remaining Batch 7.3 parity work by broadening
runtime media decode beyond WAV and replacing cached plugin render overrides
with a fresher offline plugin execution path before opening `g03.008`.
