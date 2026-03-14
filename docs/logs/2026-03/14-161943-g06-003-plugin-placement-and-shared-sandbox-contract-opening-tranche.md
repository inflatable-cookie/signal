# 2026-03-14 16:19:43 g06.003 Plugin Placement And Shared-Sandbox Contract Opening Tranche

## Summary

- opened `g06.003` Batch 3.1 with contract `014`
- froze shared plugin placement-rule, placement-policy, sandbox-grouping, and
  shared-boundary continuity vocabulary
- kept rebind, degradation, and terminal sandbox meaning explicitly
  runtime-owned ahead of deeper runtime receipt work

## Delivered

- added `docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md`
- marked Batch 3.1 complete in
  `docs/roadmaps/g06/003-plugin-transport-rebind-and-shared-sandbox-continuity-depth.md`
- updated active roadmap and reference indexes so Batch 3.2 is the only next
  queue

## Key decisions

- placement rules and grouping keys are runtime-owned interpretation, not
  product-local sandbox heuristics
- `in-process`, `shared-sandbox`, and `isolated-sandbox` are the first stable
  isolation outcomes
- multi-instance continuity is evaluated per shared sandbox boundary first,
  then projected onto member plugin instances or chains
- rebind composes with contract `012` instead of creating a second plugin-only
  recovery taxonomy

## Deferred

- runtime-owned placement-policy evaluation receipts
- explicit shared-boundary blast-radius export on runtime snapshots
- focused multi-instance recovery and allowlist or denylist or by-format proof
  fixtures
- backend-specific transport tuning or process-model detail

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Next Task

Continue `g06.003` with Batch 3.2 by implementing runtime-owned placement
evaluation, sandbox-assignment meaning, and richer shared-boundary rebind
receipts before the multi-instance proof batch closes the milestone.
