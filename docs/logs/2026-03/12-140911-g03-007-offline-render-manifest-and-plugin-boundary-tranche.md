# g03.007 - Offline Render Manifest And Plugin Boundary Tranche

Date: 2026-03-12
Status: completed tranche
Roadmap: `docs/roadmaps/g03/007-offline-render-freeze-and-stem-export-pipeline.md`

## Summary

Completed Batch 7.4 in `signal-runtime`. Offline render results now ship
through one runtime-owned manifest bundle instead of separate top-level
artifact/report receipts, and the runtime now exposes an explicit offline
plugin execution boundary that tells later consumers which stages remain inside
the Signal-owned stage model versus which ones would require host-delegated
execution.

## Shipped

- replaced separate offline artifact/report receipt fields with a typed
  `RuntimeOfflineRenderManifest` bundle on `RuntimeOfflineRenderResult`
- kept manifest ownership inside `signal-runtime` so downstream packaging can
  consume one runtime-authored delivery surface rather than reconstructing it
  from export fragments
- added runtime-owned offline plugin execution boundary surfaces that describe
  stage identity, recall state, sandbox identity, override freshness, and
  whether a stage remains Signal-modeled or would need host delegation
- added a dedicated preparation API and focused runtime proof so later offline
  consumers can inspect the plugin execution contract without parsing
  supervisor/export reports
- aligned existing offline render tests with the manifest bundle and boundary
  surfaces, including stale-override and churn-sensitive cases

## Deferred

- offline render still does not execute a true delegated host-plugin sandbox
  pass for stages that exceed the Signal-owned stage model
- delegated execution request/result receipts are not yet folded back into the
  manifest bundle as one downstream-ready delivery contract
- multichannel export and packaging workflows beyond the current runtime-owned
  manifest surface remain outside this tranche

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime`
- `git diff --check`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`

## Next Task

Continue `g03.007` with Batch 7.5 by defining the delegated offline plugin
execution request/result receipt contract and by folding delegated outcomes
into the runtime-owned manifest bundle before opening `g03.008`.
