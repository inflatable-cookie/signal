# 2026-03-17 - g07.008 runtime Linux parity receipts tranche

## Summary

Completed Batch 8.2 of `g07.008` by materializing runtime-owned Linux
cross-adapter plugin parity and sandbox-policy receipts on top of the bounded
contract frozen in Batch 8.1.

This tranche turns the Linux plugin story into real typed runtime evidence
instead of leaving Linux parity implicit in the broader cross-platform parity
record.

## Key changes

- widened `RuntimePluginFormatPlatformCoverageRecord` and
  `RuntimePluginFormatParityRecord` so runtime-owned discovery and lifecycle
  receipts now carry:
  - Linux-specific parity band
  - Linux support state
  - preferred sandbox outcome
  - strict-sandbox default
  - render-capable type counts
  - in-process, shared, isolated, restarting, rebindable, and failure counts
- threaded that widened parity family through:
  - `RuntimePluginScanReceipt`
  - `RuntimePluginDiscoverySnapshot`
  - `RuntimePluginLifecycleSnapshot`
  - observation and supervisor JSON export
- aligned Linux host coverage so server-host VST3 and LV2 scan or sandbox paths
  now feed the same widened runtime-owned Linux parity model
- added focused runtime and server-host tests that prove Linux placement,
  render, restart, rebindability, and failure posture are now visible through
  one shared record family

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_plugin_discovery_snapshot_and_reports_surface_typed_scan_filters -- --nocapture`
- `cargo test -p signal-runtime runtime_linux_plugin_parity_coverage_tracks_policy_render_failure_and_restart_receipts -- --nocapture`
- `cargo test -p signal-host-server server_host_vst3_scan_and_sandbox_surface_linux_runtime_owned_receipts -- --nocapture`
- `cargo test -p signal-host-server server_host_lv2_scan_and_sandbox_surface_linux_runtime_owned_receipts -- --nocapture`
- `cargo test -p signal-runtime --lib --no-run`
- `cargo test -p signal-host-server --lib --no-run`
- `cargo test -p signal-supervisor-tools --no-run`

## Residual risk

This tranche closes the runtime receipt depth, not the public proof seam. Batch
8.3 still needs downstream-style runtime, supervisor, and stable host-edge
proof that the widened Linux parity and sandbox-policy receipts remain
consumable without host-local Linux portability matrices.

## Next Task

Continue `g07.008` with Batch 8.3 by adding focused proofs that the widened
Linux plugin parity and sandbox-policy receipts remain consumable through
shared runtime, supervisor, and stable host-edge surfaces without host-local
Linux portability matrices.
