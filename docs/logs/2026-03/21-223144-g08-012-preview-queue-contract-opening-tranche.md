# 2026-03-21 - g08.012 preview queue contract opening tranche

## Summary

Opened `g08.012` Batch 12.1 by freezing the first runtime-owned
preview-browser queue, media audition orchestration, and transform-scheduling
contract on top of the closed preview-device seam.

## Work completed

- added
  `docs/contracts/063-preview-browser-queue-media-audition-and-transform-scheduling-contract.md`
  to freeze the authority line, shared vocabulary, rules, deferred scope, and
  Batch 12.1 outcome for preview queueing, audition orchestration, and
  transform scheduling
- updated
  `docs/roadmaps/g08/012-preview-browser-queue-media-audition-and-transform-scheduling-depth.md`
  to mark Batch 12.1 complete and point the queue at Batch 12.2
- rolled the new contract and next-step references through the contract index,
  roadmap indexes, and graph-runtime feature reference

## Validation

- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Next task

Continue `g08.012` with Batch 12.2 by materializing the first runtime-owned
preview-browser queue, media audition orchestration, and transform-scheduling
receipts, then align stable host-edge export to the same bounded model.
