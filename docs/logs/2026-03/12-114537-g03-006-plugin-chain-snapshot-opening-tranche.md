# g03.006 - Plugin Chain Snapshot Opening Tranche

Date: 2026-03-12
Status: completed tranche
Roadmap: `docs/roadmaps/g03/006-plugin-device-chain-execution-delay-compensation-and-state-recall.md`

## Summary

Opened `g03.006` with a runtime-owned plugin-chain snapshot surface instead of
leaving plugin-backed execution as isolated lifecycle and render-batch details.
The tranche adds ordered chain/stage summaries, recall-state and compensation
state modeling, and observation/supervisor export for realized plugin-backed
execution.

## Shipped

- added `RuntimePluginChainSnapshot`, `RuntimePluginExecutionChainSummary`,
  `RuntimePluginChainStageSnapshot`, `RuntimePluginRecallState`, and
  `RuntimePluginCompensationState` to `signal-runtime`
- retained the latest realized plugin render state per plugin-backed node inside
  the runtime engine so observation/report code can summarize actual realized
  latency, tail, bypass, and binding state after block execution
- derived plugin-chain summaries from planned plugin-backed node order plus
  plugin sandbox lifecycle state, preserving routed ownership fields such as
  `track_lane_id`, `bus_group_id`, `console_group_id`, and `send_return_id`
- exported the chain snapshot through compact, multiline, and JSON
  supervisor/observation report surfaces
- added focused runtime tests for:
  - compensated plus bypassed chain stages with warm versus recovered recall
  - degraded and missing-binding stages without collapsing chain grouping

## Deferred

- routed delay-compensation follow-through still needs to consume the realized
  chain-latency totals rather than treating the new snapshot as observation-only
- graph refresh, rebinding, and cold-start cases still need explicit proof so
  recall-state continuity is covered across topology churn

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime`

## Next Task

Continue `g03.006` by propagating realized plugin-chain latency and
compensation readiness into routed runtime/export summaries, then prove graph
refresh, rebinding, and cold-start recall behavior on the same chain surface.
