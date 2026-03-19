# 2026-03-19 - g08.001 Batch 1.2 Runtime Linux Live Ownership Baseline

## Summary

Materialized the first runtime-owned live Linux backend ownership receipt
family for `g08.001`.

## Delivered

- added `RuntimeLinuxBackendSessionSnapshot` and the first typed ownership,
  lifecycle, device-claim, session-role, and ownership-fallback enums in
  `crates/signal-runtime/src/interfaces.rs`
- threaded the new snapshot through runtime observation and supervisor export
- aligned `signal-host-local` to emit an explicit `NotLinux` answer on the
  same shared seam
- aligned `signal-host-server` to emit a bounded simulated PipeWire
  backend-managed live-session baseline using the existing shared hardware
  contract instead of host-private Linux session taxonomy
- updated the active roadmap, contract outcome, and architecture reference

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_linux_backend_session_snapshot_classifies_live_ownership_baselines -- --nocapture`
- `cargo test -p signal-host-local local_host_shared_report_surfaces_runtime_linux_backend_session_as_not_linux -- --nocapture`
- `cargo test -p signal-host-server server_host_shared_report_surfaces_runtime_linux_backend_session_baseline -- --nocapture`

## Next Task

Continue `g08.001` with Batch 1.3 by proving the widened live Linux backend
ownership, session-lifecycle, and device-claim seam through shared runtime,
supervisor, and stable host-edge consumer surfaces.
