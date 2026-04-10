# 2026-04-10 - g09.014 Plugin Broker And IPC Readiness Verdict

## Summary

Closed `038-g09-014-plugin-broker-readiness-verdict.md` by classifying the
plugin, adapter, and transport family against the repaired release gate.
`signal-plugin`, `signal-plugin-clap`, `signal-plugin-vst3`,
`signal-plugin-au`, `signal-plugin-lv2`, and `signal-ipc` are now promoted to
`production-ready for role`. `signal-plugin-sandbox` remains blocked on one
explicit gap: there is still no repo-owned long-lived broker operational
verdict beyond the bounded lifecycle, receipt, and demo surfaces already in
place.

## Implementation

- used the repaired gate baseline (`effigy health`, `effigy validate`,
  `effigy demo:coverage-matrix`, docs checks) as the family-wide prerequisite
- verified the shared plugin abstraction and parity surface through focused
  plugin-discovery and cross-adapter portability proofs
- verified the real VST3, AU, LV2, and CLAP family surfaces through focused
  runtime and host-edge proofs
- verified the transport and broker side through `signal-ipc` tests, a focused
  broker receipt proof, and the live sandbox-lifecycle demo
- updated the reopened `g09.014` inventory to promote the adapter and transport
  crates while keeping `signal-plugin-sandbox` explicitly blocked
- promoted the next bounded batch for the remaining
  runtime/host/hardware/broker operational family

## Validation

- `effigy health`
- `effigy validate`
- `effigy demo:coverage-matrix`
- `cargo test -p signal-runtime --test public_contract_boundary public_runtime_plugin_discovery_coverage_is_consumable_from_reexports -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-supervisor-tools --bin signal-supervisor-tools 'supervisor_main_tests::export_surface::export_json_carries_runtime_owned_plugin_discovery_capability_coverage' -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-runtime --test public_contract_boundary_cross_adapter_parity public_runtime_cross_adapter_parity_boundary_reports_runtime_owned_portability_truth -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_cross_adapter_parity local_shared_host_edge_exports_runtime_cross_adapter_parity_truth -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_cross_adapter_parity server_shared_host_edge_exports_runtime_cross_adapter_parity_truth -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-runtime --test public_contract_boundary_vst3 public_runtime_vst3_boundary_reports_runtime_owned_discovery_and_lifecycle_truth -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_vst3 local_shared_host_edge_exports_runtime_vst3_baseline_truth -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_vst3 server_shared_host_edge_exports_runtime_vst3_baseline_truth -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-runtime --test public_contract_boundary_au public_runtime_au_boundary_reports_runtime_owned_discovery_and_lifecycle_truth -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_au local_shared_host_edge_exports_runtime_au_baseline_truth -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_au server_shared_host_edge_exports_runtime_au_baseline_truth -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-runtime --test public_contract_boundary_lv2 public_runtime_lv2_boundary_reports_runtime_owned_discovery_and_lifecycle_truth -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_lv2_extension local_shared_host_edge_exports_runtime_lv2_extension_truth -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_lv2_extension server_shared_host_edge_exports_runtime_lv2_extension_truth -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_cross_adapter_parity local_shared_host_edge_exports_bounded_clap_sandbox_lifecycle_truth -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_cross_adapter_parity server_shared_host_edge_exports_bounded_clap_sandbox_lifecycle_truth -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-ipc`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_emits_vst3_flavored_receipts -- --exact --nocapture --test-threads=1`
- `effigy demo:sandbox-lifecycle`

## Notes

- `signal-plugin-sandbox` is the only crate left blocked in this family, and
  the blocker is now explicit and narrow
- the deferred `signal.demo.plugin.capability-browser` surface remains a demo
  gap, not a blocker for the promoted adapter crates

## Next Task

Continue the reopened strict `g09` lane from
`docs/specs/batch-cards/039-g09-014-runtime-host-hardware-broker-operational-verdict.md`.
