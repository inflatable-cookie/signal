# 2026-03-22 - g08.017 immersive acceptance contract opening tranche

## Summary

Opened `g08.017` by freezing the shared immersive render and monitoring
acceptance contract on top of the already-closed room-policy,
deployment-monitoring, renderer-export, and spatial consumer seams.

## Work completed

- added `docs/contracts/068-immersive-render-and-monitoring-acceptance-contract.md`
  to freeze the shared acceptance authority line, grouped scenario families,
  and required versus advisory versus deferred policy for immersive acceptance
- marked Batch 17.1 complete in
  `docs/roadmaps/g08/017-immersive-render-and-monitoring-acceptance-depth.md`
- rolled the contract, roadmap, and feature-reference indexes forward so the
  next active step is the grouped descriptor and acceptance lane

## Validation

- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Next Task

Continue `g08.017` with Batch 17.2 by wiring the first repo-owned descriptor
and acceptance lane for the shared immersive render and monitoring seam while
keeping renderer-native and workflow-native depth explicit and non-blocking.
