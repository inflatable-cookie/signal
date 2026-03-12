# g03.006 - Routed Compensation And Refresh Proof Tranche

Date: 2026-03-12
Status: completed tranche
Roadmap: `docs/roadmaps/g03/006-plugin-device-chain-execution-delay-compensation-and-state-recall.md`

## Summary

Extended the `g03.006` plugin-chain contract into routed runtime/export
surfaces. This tranche makes realized plugin-chain latency and compensation
readiness visible on routed topology summaries, adds explicit pending-render
tracking for cold-start cases, and proves that rebinding and graph refresh
clear stale realized plugin state without erasing the reusable chain contract.

## Shipped

- added explicit `pending_render_stage_count` to runtime plugin-chain summaries
- added `RuntimeRoutedPluginChainSummary` and attached it to routed execution
  topology summaries for:
  - track lanes
  - bus groups
  - console groups
  - send/returns
- enriched `RuntimeExecutionNodeSummary` with plugin recall state,
  compensation state, realized latency, and tail export
- updated compact, multiline, and JSON execution-topology exports so routed
  plugin-chain state is visible through existing supervisor/observation reports
- added focused runtime proofs for:
  - cold-start routed plugin-chain readiness on projected topology
  - stale realized-latency clearing on rebinding
  - plugin-chain removal on graph refresh

## Deferred

- typed plugin state-recall payload/status surfaces are still missing; current
  work exports recall state, but not yet a reusable recall payload contract
- later offline render/freeze work still needs a stronger handoff between the
  runtime-owned chain contract and recall payload ownership

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime`

## Next Task

Continue `g03.006` by defining typed plugin state-recall payload/status
surfaces, then prove recovered, quarantined, and unavailable recall export
without pushing recall ownership into host-local bookkeeping.
