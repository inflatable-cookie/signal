# 2026-03-14 15:36:50 - g06.001 public interruption boundary proof closure

## What changed

- completed `g06.001` Batch 1.3 and closed the milestone
- added focused downstream-style runtime proofs for restartable runtime
  interruption export and resumable deferred-work continuity on public
  reexports
- added a machine-readable interruption-boundary descriptor to
  `signal-supervisor-tools`
- added the repo-owned `acceptance:interruption-boundary` task so the proof is
  runnable without private app scripts
- moved the active queue to `g06.002`

## Evidence

- `crates/signal-runtime/tests/public_contract_boundary.rs`
- `crates/signal-supervisor-tools/src/main.rs`
- `effigy.toml`
- `docs/roadmaps/g06/001-runtime-interruption-taxonomy-and-resumability-contract.md`
- `docs/roadmaps/g06/002-recording-continuity-midi-capture-and-checkpoint-truth.md`

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime public_runtime_interruption_boundary_reports_restartable_runtime_state`
- `cargo test -p signal-runtime public_runtime_interruption_boundary_reports_resumable_deferred_state`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_interruption_boundary_mode`
- `cargo test -p signal-supervisor-tools interruption_boundary_json_reports_runtime_and_host_edge_proofs`
- `cargo test -p signal-host-local local_shared_host_edge_is_consumable_without_private_helpers`
- `cargo test -p signal-host-server server_shared_host_edge_is_consumable_without_private_helpers`
- `cargo run -p signal-supervisor-tools -- --describe-interruption-boundary --format=json`
- `effigy acceptance:interruption-boundary`

## Deferred

- device-loss-specific interruption truth still needs later `g06` hardware and
  supervision depth to become more explicit on runtime-owned surfaces
- recording continuity, plugin transport continuity, and offline render
  recovery depth remain queued in later `g06` milestones rather than being
  stretched into this closeout

## Next Task

Continue `g06.002` with Batch 2.1 by defining the recording continuity and
checkpoint contract for audio and MIDI capture, then freeze what evidence
survives interruption, restart, and failed runtime boundaries.
