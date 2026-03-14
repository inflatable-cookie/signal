# g03.006 - Runtime Plugin Recall Handoff Boundary Tranche

Date: 2026-03-12
Status: completed tranche
Roadmap: `docs/roadmaps/g03/006-plugin-device-chain-execution-delay-compensation-and-state-recall.md`

## Summary

Defined the runtime-owned recall handoff boundary for later offline
render/freeze work without implementing those flows. This tranche separates the
authoritative recall payload from export-only summary decoration and exposes a
dedicated `RuntimePluginRecallHandoffSnapshot` so future consumers can stay on
runtime contracts instead of mining supervisor export.

## Shipped

- added `RuntimePluginRecallPayload` as the authoritative recall payload owned
  by `signal-runtime`
- changed `RuntimePluginRecallSnapshot` so report-oriented recall export wraps:
  - recall status
  - authoritative payload
  - summary decoration
- added `RuntimePluginRecallHandoffStage` and
  `RuntimePluginRecallHandoffSnapshot` as the sanctioned runtime handoff
  surface for later offline render/freeze entry points
- added `RuntimeObservationApi::get_plugin_recall_handoff_snapshot()` and wired
  `SignalRuntime` to derive it from the existing runtime-owned plugin-chain
  source of truth
- extended focused runtime proofs so recovered, unavailable, quarantined, and
  churned topology cases also verify the handoff boundary rather than only the
  export snapshot

## Deferred

- later offline render/freeze request assembly still needs an explicit identity
  contract for selecting handoff stages without copying recall fields
- no `g03.007` implementation was started; this tranche only defines the
  boundary the next phase must consume

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime`
- `git diff --check`
- `effigy health`
- `effigy test`
- `effigy validate`

## Next Task

Continue `g03.006` by defining how later offline render/freeze request
assembly references `RuntimePluginRecallHandoffSnapshot` stages by stable
identity, then add one API-local proof that consumers do not need
supervisor/export parsing before `g03.007`.
