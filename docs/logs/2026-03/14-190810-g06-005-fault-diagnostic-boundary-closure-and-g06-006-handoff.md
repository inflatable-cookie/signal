# 2026-03-14 19:08:10 - g06.005 fault-diagnostic boundary closure and g06.006 handoff

## What changed

- added the first dedicated consumer-facing fault-diagnostic proof boundary
  through `signal-supervisor-tools --describe-fault-diagnostic-boundary`
  and `effigy acceptance:fault-diagnostic-boundary --repo .`
- added focused downstream-style runtime proof coverage for canonical
  `primary_family` export on public runtime surfaces
- added stable local and server host-edge proofs showing
  `supervisor_report()` forwards the same runtime-owned fault-diagnostic
  receipt without host-local causal reclassification
- restored the current `RuntimeSupervisorApi` surface in both host crates by
  delegating `start_media_preview()` and `stop_media_preview()` into
  `signal-runtime`
- closed `g06.005` and moved the active queue to `g06.006`

## Evidence

- `crates/signal-runtime/tests/public_contract_boundary.rs`
- `crates/signal-host-local/tests/public_host_edge_boundary.rs`
- `crates/signal-host-server/tests/public_host_edge_boundary.rs`
- `crates/signal-host-local/src/host.rs`
- `crates/signal-host-server/src/host.rs`
- `crates/signal-supervisor-tools/src/main.rs`
- `effigy.toml`
- `docs/contracts/016-runtime-fault-cause-attribution-and-diagnostic-receipt-contract.md`
- `docs/roadmaps/g06/005-runtime-fault-cause-attribution-and-diagnostic-receipts.md`
- `docs/roadmaps/g06/006-per-block-execution-timing-and-pressure-snapshots.md`
- `docs/roadmaps/g06/README.md`
- `docs/roadmaps/README.md`
- `docs/roadmaps/generation-index.md`
- `docs/contracts/README.md`
- `docs/architecture/graph-runtime-feature-reference.md`

## Validation

- `cargo test -p signal-runtime public_runtime_fault_diagnostic_boundary_reports_canonical_runtime_receipts -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_runtime_fault_diagnostic_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_fault_diagnostic_truth -- --nocapture`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_fault_diagnostic_boundary_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools fault_diagnostic_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-fault-diagnostic-boundary --format=json`
- `effigy acceptance:fault-diagnostic-boundary --repo .`

## Deferred

- callback pressure remains advisory host evidence rather than a stronger
  canonical runtime family
- per-event traces, remote diagnostics pipelines, and product-specific
  diagnostic UX remain out of scope for `g06.005`
- `g06.006` still needs to define bounded timing and pressure measurements
  before scheduler or hot-path optimization work can rely on them

## Next Task

Continue `g06.006` with Batch 6.1 by defining the first per-block execution
timing and pressure snapshot contract so the newly closed fault-diagnostic
boundary can feed into bounded profiling work instead of counter-only
performance anecdotes.
