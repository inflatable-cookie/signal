# g03.007 - Offline Render Contract Preview Tranche

Date: 2026-03-12
Status: completed tranche
Roadmap: `docs/roadmaps/g03/007-offline-render-freeze-and-stem-export-pipeline.md`

## Summary

Opened `g03.007` with the reusable contract batch in `signal-runtime`. This
tranche defines typed offline render request, stem target, freeze artifact, and
contract-preview surfaces that consume the runtime-owned topology, tempo,
clip-processing, and plugin recall handoff state already established by
`g03.005` and `g03.006`.

## Shipped

- added `RuntimeOfflineRenderRequest`, `RuntimeOfflineRenderStemTarget`, and
  `RuntimeOfflineFreezeArtifactRequest` as the runtime-owned offline consumer
  request seam
- added `RuntimeOfflineRenderContractPreview`,
  `RuntimeOfflineRenderStemPreview`, and
  `RuntimeOfflineFreezeArtifactPreview` so callers can inspect resolved render
  scope and dependencies before a full offline engine path exists
- resolved stem targets against runtime-owned routed topology for:
  - main mix
  - track lanes
  - bus groups
  - console groups
  - send/return groups
- resolved freeze artifact recall dependencies through
  `RuntimePluginRecallHandoffSnapshot` and stable handoff stage ids instead of
  report parsing or host-local copied recall state
- added a focused runtime proof that offline render contract preview reuses:
  - topology summaries
  - clip-processing snapshots
  - tempo-map state
  - recall handoff selection

## Deferred

- this tranche does not yet implement the actual offline render, freeze, or
  stem-processing engine path
- artifact production, render scheduling, and freeze execution still belong to
  `g03.007` Batch 7.2
- `g03.008` remains intentionally unopened until the render path is real enough
  to profile and harden

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime`
- `git diff --check`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`

## Next Task

Continue `g03.007` with Batch 7.2 by implementing the first credible offline
render path on top of this request/preview contract, then prove freeze and stem
output on the same substrate without feature-specific forks.
