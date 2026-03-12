# g03.007 - Delegated Offline Plugin Receipts Tranche

Date: 2026-03-12
Status: completed tranche
Roadmap: `docs/roadmaps/g03/007-offline-render-freeze-and-stem-export-pipeline.md`

## Summary

Completed Batch 7.5 in `signal-runtime`. The offline render surface now
derives delegated plugin execution requests from the runtime-owned plugin
execution boundary, accepts typed delegated execution receipts back onto the
same render result, and folds those outcomes into the runtime-owned manifest
bundle instead of leaving delegated stages in a host-local side channel.

## Shipped

- added typed delegated offline plugin execution request/result receipt DTOs
  alongside the existing runtime-owned offline render manifest and plugin
  execution boundary
- derived delegated stage requests directly from host-delegated stages in the
  runtime-owned execution boundary, preserving stage identity, recall payload,
  and override freshness without export parsing
- folded delegated execution request metadata and optional delegated receipts
  into `RuntimeOfflineRenderManifest` so downstream packaging still consumes
  one runtime-authored delivery bundle
- extended offline render report JSON with delegated stage counters so report
  output reflects delegated execution state without inventing a second report
  path
- added focused runtime proofs for delegated request filtering and delegated
  receipt application back into the manifest bundle

## Deferred

- the runtime still does not materialize a true end-to-end delegated offline
  plugin sandbox execution pass for stages that exceed the Signal-owned stage
  model
- current delegated proofs use the runtime-owned contract directly; they do not
  yet demonstrate a full delegated render handoff flowing through a real host
  executor
- graph projection still requires at least one stage per node, so
  host-delegated stage materialization remains a later runtime boundary proof
  rather than an exercised graph shape

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime`

## Next Task

Continue `g03.007` with Batch 7.6 by materializing the delegated offline
plugin execution handoff end-to-end and by proving delegated-stage
report/manifest export through the same runtime-owned delivery bundle before
opening `g03.008`.
