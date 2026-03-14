# 2026-03-14 17:09:46 - g06.003 plugin continuity proof closure and g06.004 handoff

## What changed

- added focused runtime proofs for shared-sandbox blast radius, recovery, and
  terminal continuity across several member plugin instances
- added focused runtime proofs for allowlist, denylist, and by-format
  placement policy behavior on runtime-owned lifecycle and chain receipts
- added a downstream-style public runtime proof so shared-boundary continuity
  and placement truth stay consumable without private helpers
- added shared host-edge proofs for both local and server hosts so
  `supervisor_report()` preserves placement, grouping, and terminal continuity
  meaning without host-local reconstruction
- added the machine-readable
  `signal.runtime.plugin-continuity-boundary` descriptor and the repo-owned
  `effigy acceptance:plugin-continuity` task
- closed `g06.003`, marked contract `014` complete, and moved the active queue
  to `g06.004`

## Evidence

- `crates/signal-runtime/src/runtime.rs`
- `crates/signal-runtime/tests/public_contract_boundary.rs`
- `crates/signal-host-local/tests/public_host_edge_boundary.rs`
- `crates/signal-host-server/tests/public_host_edge_boundary.rs`
- `crates/signal-supervisor-tools/src/main.rs`
- `effigy.toml`
- `docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md`
- `docs/roadmaps/g06/003-plugin-transport-rebind-and-shared-sandbox-continuity-depth.md`

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_shared_sandbox_blast_radius_stays_boundary_local_across_recovery_and_terminal_states -- --nocapture`
- `cargo test -p signal-runtime runtime_plugin_placement_policy_exports_allowlist_denylist_and_by_format_receipts -- --nocapture`
- `cargo test -p signal-runtime public_runtime_plugin_continuity_boundary_reports_shared_boundary_and_policy_truth -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_plugin_placement_and_shared_boundary_continuity_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_plugin_placement_and_shared_boundary_continuity_truth -- --nocapture`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_plugin_continuity_boundary_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools plugin_continuity_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `effigy acceptance:plugin-continuity --repo .`

## Deferred

- dedicated blast-radius DTOs are still deferred beyond the current lifecycle
  and chain receipt family
- the exercised proof path is still sandbox-first, so deeper in-process parity
  and broader adapter transport tuning remain later plugin-format work
- offline render recovery and resumability depth still need the next milestone
  before the broader runtime recovery lane is complete

## Next Task

Continue `g06.004` with Batch 4.1 by freezing resumable, restartable,
recoverable, and terminal offline-render session outcomes, then align render
checkpoint survival semantics with the shared interruption taxonomy before
runtime session-depth work begins.
