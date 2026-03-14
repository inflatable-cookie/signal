# 2026-03-14 18:45:41 - g06.005 runtime fault diagnostic receipt depth tranche

## What changed

- added the first typed `g06.005` runtime diagnostic receipt family in
  `signal-runtime`: `RuntimeFaultDiagnosticReceipt`,
  `RuntimeFaultContributionReceipt`, `RuntimeFaultDiagnosticFamily`, and
  `RuntimeFaultDiagnosticAuthority`
- threaded canonical fault-cause and contribution export through
  `RuntimeObservationReport`, `RuntimeSupervisorReport`, and
  `RuntimeProfilingReceipt` so runtime, supervisor JSON, and profiling surfaces
  now expose one shared primary-family story instead of counter-only summaries
- kept callback and backend host evidence additive by marking it
  `HostAdvisory`, while runtime-owned posture remains the only source of
  canonical `primary_family`
- aligned the downstream-style runtime proof and the stable local/server
  host-edge proofs to the same receipt family without host-local causal
  reclassification
- marked Batch 5.2 complete and moved `g06.005` to the focused consumer-proof
  pass in Batch 5.3

## Evidence

- `crates/signal-runtime/src/interfaces.rs`
- `crates/signal-runtime/src/lib.rs`
- `crates/signal-runtime/src/runtime.rs`
- `crates/signal-runtime/tests/public_contract_boundary.rs`
- `crates/signal-host-local/src/host.rs`
- `crates/signal-host-local/tests/public_host_edge_boundary.rs`
- `crates/signal-host-server/tests/public_host_edge_boundary.rs`
- `docs/contracts/016-runtime-fault-cause-attribution-and-diagnostic-receipt-contract.md`
- `docs/roadmaps/g06/005-runtime-fault-cause-attribution-and-diagnostic-receipts.md`
- `docs/roadmaps/g06/README.md`
- `docs/architecture/graph-runtime-feature-reference.md`

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_fault_diagnostic_receipt_ -- --nocapture`
- `cargo test -p signal-runtime public_runtime_contract_boundary_is_consumable_from_reexports -- --nocapture`
- `cargo test -p signal-host-local local_host_shared_report_derives_profiling_and_soak_receipts -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_is_consumable_without_private_helpers -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_is_consumable_without_private_helpers -- --nocapture`
- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Deferred

- there is still no dedicated consumer-facing fault-diagnostic descriptor or
  repo-owned acceptance seam; Batch 5.3 needs to prove the new receipt family
  through one focused public boundary
- callback pressure remains advisory host evidence rather than a stronger
  canonical runtime family
- per-event traces, remote diagnostics pipelines, and product-specific
  diagnostic UX remain out of scope for `g06.005`

## Next Task

Continue `g06.005` with Batch 5.3 by adding a focused consumer-facing proof and
repo-owned acceptance or descriptor seam for the typed fault-cause receipt
family so downstream tooling can consume canonical primary-family and
contributing-evidence meaning without private host helpers.
