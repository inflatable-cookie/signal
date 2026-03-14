# g03.006 - Runtime Plugin Recall Payload Export Tranche

Date: 2026-03-12
Status: completed tranche
Roadmap: `docs/roadmaps/g03/006-plugin-device-chain-execution-delay-compensation-and-state-recall.md`

## Summary

Completed the typed recall-payload/export portion of `g03.006` inside
`signal-runtime`. This tranche keeps recall ownership in runtime-owned plugin
lifecycle data, adds a typed recall snapshot alongside the coarse recall enum,
and proves recovered, unavailable, and quarantined recall export through the
existing plugin-chain and execution-topology surfaces.

## Shipped

- added `RuntimePluginRecallSnapshot` to `signal-runtime` and attached it to:
  - `RuntimePluginChainStageSnapshot`
  - `RuntimeExecutionNodeSummary`
- derived recall payload once from runtime-owned plugin lifecycle snapshots,
  preserving:
  - recall status
  - sandbox identity
  - lifecycle and transport stages
  - readiness state
  - recovery, restart, and fault counters
  - typed restart/stop/fault details
  - degraded reasons
- exported the typed recall payload through supervisor and observation
  multiline/JSON surfaces without adding host-local recall bookkeeping
- extended focused runtime proofs for:
  - recovered recall payload/status
  - unavailable recall payload/status
  - quarantined recall payload/status
  - execution-topology export after rebinding and graph refresh churn

## Deferred

- later offline render/freeze work still needs an explicit runtime-owned recall
  handoff boundary so those flows consume authoritative recall payload instead
  of rebuilding it in host-local request assembly
- `g03.007` implementation remains out of scope; only the ownership boundary
  should be defined next

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime`
- `git diff --check`
- `effigy health`
- `effigy test`
- `effigy validate`

## Next Task

Continue `g03.006` by defining the runtime-owned recall handoff boundary for
later offline render/freeze entry points, making authoritative versus
export-only recall fields explicit before `g03.007` opens.
