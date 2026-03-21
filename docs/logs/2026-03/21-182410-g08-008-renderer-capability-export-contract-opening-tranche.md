# 2026-03-21 - g08.008 renderer capability and immersive export contract opening tranche

## Summary

Completed `g08.008` Batch 8.1 by freezing the first runtime-owned
renderer-capability negotiation and immersive export contract instead of
leaving that boundary implicit under the closed immersive room-policy and
deployment-monitoring seams.

## Changes

- added
  `docs/contracts/059-renderer-capability-negotiation-and-immersive-export-contract.md`
  to freeze renderer-capability posture, capability authority, immersive export
  class, export authority, and immersive export outcome
- updated the active milestone in
  `docs/roadmaps/g08/008-renderer-capability-negotiation-and-immersive-export-baseline.md`
  to mark Batch 8.1 complete and move the queue to Batch 8.2
- updated the contract index and shared next-task pointers in
  `docs/contracts/README.md`, `docs/roadmaps/g08/README.md`,
  `docs/roadmaps/README.md`, `docs/roadmaps/generation-index.md`, and
  `docs/architecture/graph-runtime-feature-reference.md`

## Validation

- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Residual Risk

This tranche freezes meaning, not runtime realization. Renderer-capability
receipts, immersive export receipts, and shared consumer proof still belong to
later `g08.008` batches.

## Next Task

Continue `g08.008` with Batch 8.2 by materializing the first runtime-owned
renderer-capability negotiation and immersive export receipts, then align
stable host-edge export to the same bounded model.
