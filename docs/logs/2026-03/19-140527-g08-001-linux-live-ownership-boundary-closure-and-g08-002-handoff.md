# g08.001 Batch 1.3 - Linux Live Ownership Boundary Closure And g08.002 Handoff

Date: 2026-03-19
Milestone: `g08.001`
Batch: `1.3`
Status: complete

## Summary

Closed the live Linux backend ownership consumer seam through public runtime,
both stable host edges, and a machine-readable supervisor-tools descriptor.
`RuntimeLinuxBackendSessionSnapshot` is now proven consumable as one shared
runtime-owned seam for ownership, lifecycle, device-claim, role, and guarded
fallback truth.

## Shipped

- added the public runtime proof for live Linux backend ownership, lifecycle,
  device-claim, and guarded fallback
- added stable local and server host-edge proofs for explicit `NotLinux` and
  bounded PipeWire-style live-session export
- added `signal.runtime.linux-live-ownership-boundary` to
  `signal-supervisor-tools`
- added `acceptance:linux-live-ownership-boundary` to Effigy
- closed `g08.001` and activated `g08.002`

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime public_runtime_linux_live_ownership_boundary_reports_runtime_owned_session_truth -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_runtime_linux_live_ownership_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_linux_live_ownership_truth -- --nocapture`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_linux_live_ownership_boundary_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools linux_live_ownership_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-linux-live-ownership-boundary --format=json`
- `effigy acceptance:linux-live-ownership-boundary --repo .`
- `effigy test --plan --repo .`
- `git diff --check`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy qa:docs --repo .`

## Residual Risk

This closes the bounded proof seam, not real ALSA, JACK, or PipeWire daemon
coordination depth, transport integration, or backend-native recovery policy.

## Next Task

Continue `g08.002` with Batch 2.1 by freezing the runtime-owned JACK transport,
graph, and backend-native coordination contract on top of the now-closed live
Linux backend ownership seam.
