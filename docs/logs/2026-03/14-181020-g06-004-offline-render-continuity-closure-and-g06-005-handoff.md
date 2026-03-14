# 2026-03-14 18:10:20 - g06.004 offline render continuity closure and g06.005 handoff

## What changed

- added explicit restartable and failed-terminal offline render session meaning
  to the runtime-owned session snapshot family instead of leaving those paths
  implicit in stop/restart behavior or raw delivery errors
- proved resumable, restartable, and terminal offline render continuity
  through focused runtime tests plus downstream-style runtime and host-edge
  proofs
- added the machine-readable
  `signal.runtime.offline-render-continuity-boundary` descriptor and the
  repo-owned `effigy acceptance:offline-render-continuity` task
- closed `g06.004`, marked contract `015` complete, and moved the active queue
  to `g06.005`

## Evidence

- `crates/signal-runtime/src/runtime.rs`
- `crates/signal-runtime/src/interfaces.rs`
- `crates/signal-runtime/tests/public_contract_boundary.rs`
- `crates/signal-host-local/tests/public_host_edge_boundary.rs`
- `crates/signal-host-server/tests/public_host_edge_boundary.rs`
- `crates/signal-supervisor-tools/src/main.rs`
- `effigy.toml`
- `docs/contracts/015-offline-render-recovery-and-resumability-contract.md`
- `docs/roadmaps/g06/004-offline-render-execution-recovery-and-resumability-depth.md`
- `docs/roadmaps/g06/005-runtime-fault-cause-attribution-and-diagnostic-receipts.md`

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime offline_render_session_snapshot_reports_`
- `cargo test -p signal-runtime public_runtime_offline_render_continuity_boundary_reports_resumable_restartable_and_terminal_states`
- `cargo test -p signal-host-local local_shared_host_edge_exports_resumable_offline_render_session_truth`
- `cargo test -p signal-host-server server_shared_host_edge_exports_restartable_and_terminal_offline_render_session_truth`
- `cargo test -p signal-supervisor-tools offline_render_continuity_boundary_json_reports_runtime_and_host_edge_proofs`
- `effigy acceptance:offline-render-continuity --repo .`

## Deferred

- restart-survival across full process restart is still deeper than the current
  stop/restart proof and belongs to later recovery or orchestration work
- durable distributed queue ownership and remote offline render job execution
  remain out of scope for this continuity milestone
- `g06.005` still needs the next causal-diagnostics contract before profiling
  and pressure receipts can widen coherently

## Next Task

Continue `g06.005` with Batch 5.1 by defining the runtime-owned fault-cause
attribution and diagnostic receipt contract so later profiling, pressure, and
soak work can cite typed causal evidence instead of counter-only summaries.
