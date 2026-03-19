## g07.003 Batch 3.2 - Runtime Multi-Bus Topology Receipts

- widened `signal-runtime` with runtime-owned `RuntimeBusConnectionSummary`
  and `RuntimeAuxiliaryPathSummary` so multi-bus connection identity,
  auxiliary-path grouping, bus role, attachment class, and fallback outcome are
  explicit typed execution receipts instead of contract-only terms
- aligned live execution, offline render dependency preview, and diagnostic
  metering on the same receipt family by widening
  `RuntimeExecutionTopologySummary`, `RuntimeOfflineRenderChainDependencyPreview`,
  and `RuntimeMeteringSnapshot`
- extended the focused runtime proofs so send-return and submix topology now
  assert concrete connection ids, auxiliary-path ids, and supervisor JSON
  export instead of only group counts
- rolled the roadmap and contract trail forward for Batch 3.3 consumer-boundary
  proof work

### Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_execution_topology_summarizes_send_return_routes_explicitly -- --nocapture`
- `cargo test -p signal-runtime runtime_offline_render_contract_preview_carries_sidechain_dependency_receipts -- --nocapture`
