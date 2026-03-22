# 2026-03-21 - g08.013 transform persistence contract opening tranche

## Summary

Opened `g08.013` Batch 13.1 by freezing the first runtime-owned asset/session
transform persistence, retention, and cache placement policy contract on top
of the closed preview-workflow seam.

## Work completed

- added
  `docs/contracts/064-asset-session-transform-persistence-retention-and-cache-placement-policy-contract.md`
  to freeze the authority line, shared vocabulary, rules, deferred scope, and
  Batch 13.1 outcome for transform persistence, retention, and cache
  placement policy
- updated
  `docs/roadmaps/g08/013-asset-session-transform-persistence-retention-and-cache-placement-policy.md`
  to mark Batch 13.1 complete and point the queue at Batch 13.2
- rolled the new contract and next-step references through the contract index,
  roadmap indexes, and graph-runtime feature reference

## Validation

- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Next task

Continue `g08.013` with Batch 13.2 by materializing the first runtime-owned
asset/session transform persistence, retention, and cache placement receipts,
then align stable host-edge export to the same bounded model.
