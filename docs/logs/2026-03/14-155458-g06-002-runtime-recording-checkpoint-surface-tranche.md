# 2026-03-14 15:54:58 - g06.002 runtime recording checkpoint surface tranche

## What changed

- widened `RuntimeRecordingCaptureSnapshot` with typed capture kind,
  `active_checkpoint`, `last_checkpoint`, and explicit buffered event count so
  recording continuity no longer stops at enum-level state
- widened `RuntimeRecordingCaptureCommitReceipt` with runtime-owned capture
  kind and committed checkpoint evidence
- preserved restartable buffered checkpoint truth across runtime stop and
  reconfigure instead of dropping active capture state silently
- threaded `recording_capture_snapshot` into `RuntimeObservationReport` and
  `RuntimeSupervisorReport` JSON or multiline surfaces so shared host edges
  observe the same runtime-owned capture truth
- extended focused runtime and downstream-style public proofs to cover the new
  recording checkpoint export surface

## Evidence

- `crates/signal-runtime/src/interfaces.rs`
- `crates/signal-runtime/src/runtime.rs`
- `crates/signal-runtime/tests/public_contract_boundary.rs`
- `crates/signal-host-local/tests/public_host_edge_boundary.rs`
- `crates/signal-host-server/tests/public_host_edge_boundary.rs`
- `docs/contracts/013-recording-continuity-midi-capture-and-checkpoint-contract.md`

## Validation

- `cargo test -p signal-runtime runtime_recording_capture -- --nocapture`
- `cargo test -p signal-runtime public_runtime_contract_boundary_is_consumable_from_reexports -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_is_consumable_without_private_helpers -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_is_consumable_without_private_helpers -- --nocapture`

## Deferred

- concrete MIDI event capture and commit DTOs still remain deferred
- same-identity resumable capture after a mid-stream interruption is not yet
  proven; this tranche mainly establishes typed steady, committed, and
  restartable checkpoint truth
- terminal capture proof still needs a focused public/runtime case rather than
  only the typed DTO path

## Next Task

Continue `g06.002` with Batch 2.3 by proving resumed, restarted, and terminal
capture outcomes through shared runtime and host-edge surfaces so downstream
consumers can distinguish them without log parsing or host-local
reclassification.
