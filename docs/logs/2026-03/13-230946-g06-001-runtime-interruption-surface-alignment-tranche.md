# 2026-03-13 23:09:46 - g06.001 runtime interruption surface alignment tranche

## What changed

- completed `g06.001` Batch 1.2
- added runtime-owned `fault_status` and `interruption_summary` export to
  `RuntimeObservationReport` and `RuntimeSupervisorReport`
- aligned deferred-work and offline-render continuity receipts to the same
  `RuntimeInterruptionClass` vocabulary instead of leaving resumability implied
- kept local and server host-edge consumers on the same meaning by proving the
  shared `supervisor_report()` surface carries the new interruption fields

## Evidence

- `crates/signal-runtime/src/interfaces.rs`
- `crates/signal-runtime/src/runtime.rs`
- `crates/signal-runtime/tests/public_contract_boundary.rs`
- `crates/signal-host-local/tests/public_host_edge_boundary.rs`
- `crates/signal-host-server/tests/public_host_edge_boundary.rs`

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_observation_report_surfaces_restartable_interruption_summary`
- `cargo test -p signal-runtime runtime_offline_render_execution_pauses_and_resumes_without_early_delivery`
- `cargo test -p signal-runtime runtime_offline_render_execution_becomes_recoverable_and_resumes_after_interrupt`
- `cargo test -p signal-runtime runtime_offline_render_queue_throttles_when_runtime_is_running`
- `cargo test -p signal-runtime runtime_offline_render_queue_defers_and_resumes_after_safe_mode_clears`
- `cargo test -p signal-runtime public_runtime_contract_boundary_is_consumable_from_reexports`
- `cargo test -p signal-host-local local_shared_host_edge_is_consumable_without_private_helpers`
- `cargo test -p signal-host-server server_shared_host_edge_is_consumable_without_private_helpers`

## Deferred

- device-loss-specific fault truth still enters host-facing reports through
  broader host I/O surfaces rather than a runtime-owned device-loss snapshot
- Batch 1.3 still needs a tighter downstream-style proof and likely an Effigy
  acceptance task so interruption inspection is not only covered by focused
  crate tests

## Next Task

Continue `g06.001` with Batch 1.3 by adding the focused downstream-style proof
for interruption and resumability inspection across shared runtime and
host-edge surfaces, then record any remaining deferred subsystem-specific
interruption classes explicitly before opening `g06.002`.
