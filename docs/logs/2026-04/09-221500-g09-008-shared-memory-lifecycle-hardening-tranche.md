# 2026-04-09 - g09.008 Shared-Memory Lifecycle Hardening Tranche

## Summary

Completed the strict `g09.008` shared-memory lifecycle hardening batch. The
shared-memory broker now writes a metadata sidecar for each region, validates
that metadata on attach and destroy, tightens broker root and file permissions
on Unix, and emits explicit lifecycle errors for missing metadata, missing
backing files, malformed sidecars, and size mismatch instead of silently
relying on best-effort temp-file cleanup.

## Implementation

- added typed shared-memory lifecycle errors and operations in
  `crates/signal-ipc/src/shared_memory.rs`
- added sidecar metadata write/read/validation for region identity and byte
  shape
- tightened Unix permission posture for broker root, backing files, and
  metadata sidecars
- made attach and destroy fail explicitly on stale or partially torn-down
  region state
- added focused stale/mismatch/missing cleanup tests in
  `crates/signal-ipc/src/tests.rs`
- updated the two recovery cleanup paths that convert broker destroy failures
  into runtime errors so they accept the new typed lifecycle error boundary

## Validation

- `cargo test -p signal-ipc`
- `cargo check -p signal-plugin-clap`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_drive_broker_backed_vst3_crash_recovery -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_drive_broker_backed_lv2_crash_recovery -- --exact --nocapture --test-threads=1`
- `effigy health`

## Reassessment

No further honest bounded seam remains inside `g09.008` without widening into
the next milestone. The strict lane should re-enter planning before creating
another ready card.

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether
`g09.008` closes here or hands off into `g09.009` before creating another
ready batch card.
