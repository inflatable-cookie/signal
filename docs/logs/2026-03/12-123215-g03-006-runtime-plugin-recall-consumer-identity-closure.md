# g03.006 - Runtime Plugin Recall Consumer Identity Closure

Date: 2026-03-12
Status: completed tranche
Roadmap: `docs/roadmaps/g03/006-plugin-device-chain-execution-delay-compensation-and-state-recall.md`

## Summary

Closed `g03.006` by defining the stable identity contract future offline
render/freeze consumers must use when selecting runtime-owned recall handoff
stages. This tranche finishes the plugin-chain, compensation, recall payload,
handoff, and consumer-selection contract without opening `g03.007`.

## Shipped

- added stable `RuntimePluginRecallHandoffStageId` identities on runtime recall
  handoff stages
- added `RuntimePluginRecallHandoffSelection` plus handoff-snapshot resolution
  helpers so future consumer request assembly can reference runtime recall
  stages by identity instead of copied recall fields
- kept authoritative recall data in `RuntimePluginRecallPayload` and left
  report-only summary decoration in `RuntimePluginRecallSnapshot`
- added an API-local proof that a future recall consumer can:
  - collect stable handoff stage ids
  - resolve them back through `RuntimePluginRecallHandoffSnapshot`
  - consume authoritative payload directly
  - reject missing identities without parsing supervisor/export summaries

## Deferred

- `g03.007` implementation is still intentionally unopened; only the
  prerequisite runtime contract is complete
- later offline render/freeze work still needs its own execution and request
  types, but those should now consume the finished handoff identity contract
  rather than redefining recall ownership

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime`
- `git diff --check`
- `effigy health`
- `effigy test`
- `effigy validate`

## Completion

`g03.006` is complete. Hold the runtime recall payload and handoff identity
surfaces as the baseline until you intentionally open `g03.007`.
