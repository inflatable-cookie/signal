# 2026-03-14 16:10:10 - g06.002 recording continuity proof closure and g06.003 handoff

## What changed

- made active recording checkpoints explicitly `Resumable` when runtime stays on
  the same capture identity through degraded safe-mode state
- promoted terminal recording commit failures into typed failed checkpoints
  instead of leaving them as error text only
- added focused runtime proofs for resumable, restartable, and terminal capture
  continuity
- added downstream-style public runtime and shared host-edge proofs so
  consumers can distinguish those outcomes without private helpers or log
  parsing
- added the machine-readable
  `signal.runtime.recording-continuity-boundary` descriptor and the repo-owned
  `effigy acceptance:recording-continuity` task
- closed `g06.002`, marked contract `013` complete, and moved the active queue
  to `g06.003`

## Evidence

- `crates/signal-runtime/src/interfaces.rs`
- `crates/signal-runtime/src/lib.rs`
- `crates/signal-runtime/src/runtime.rs`
- `crates/signal-runtime/tests/public_contract_boundary.rs`
- `crates/signal-host-local/tests/public_host_edge_boundary.rs`
- `crates/signal-host-server/tests/public_host_edge_boundary.rs`
- `crates/signal-supervisor-tools/src/main.rs`
- `effigy.toml`
- `docs/contracts/013-recording-continuity-midi-capture-and-checkpoint-contract.md`
- `docs/roadmaps/g06/002-recording-continuity-midi-capture-and-checkpoint-truth.md`

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_recording_capture_resumes_same_identity_after_safe_mode_clears -- --nocapture`
- `cargo test -p signal-runtime runtime_recording_capture_preserves_restartable_checkpoint_across_stop_and_reconfigure -- --nocapture`
- `cargo test -p signal-runtime runtime_recording_capture_reports_terminal_checkpoint_on_commit_failure -- --nocapture`
- `cargo test -p signal-runtime public_runtime_recording_continuity_boundary_reports_resumable_restartable_and_terminal_states -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_resumable_recording_checkpoint_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_restartable_and_terminal_recording_checkpoint_truth -- --nocapture`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_recording_continuity_boundary_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools recording_continuity_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `effigy acceptance:recording-continuity --repo .`

## Deferred

- concrete MIDI event capture and commit receipts are still deferred, so the
  continuity family is complete semantically but not yet format-complete
- resumable capture is currently proven through safe-mode degradation on the
  same active identity, not through a richer dedicated capture pause or resume
  API
- shared plugin rebind and shared-sandbox continuity still need the next
  milestone before broader recovery policy can reuse the same vocabulary

## Next Task

Continue `g06.003` with Batch 3.1 by freezing placement-rule vocabulary,
sandbox grouping keys, and shared plugin rebind or terminal continuity
semantics before deeper runtime policy evaluation and multi-instance proof
work.
