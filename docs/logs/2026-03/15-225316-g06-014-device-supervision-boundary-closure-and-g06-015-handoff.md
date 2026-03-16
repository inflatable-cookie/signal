# 2026-03-15 22:53:16 UTC - g06.014 Device Supervision Boundary Closure And g06.015 Handoff

## Summary

Closed `g06.014` by proving the runtime-owned device supervision boundary across
public runtime, stable host edges, and a machine-readable supervisor-tools
descriptor. This batch makes recovered, exhausted, and explicit faulted device
outcomes consumable without host-local restart classification.

## Work completed

- added the downstream-style runtime proof for the public device-supervision
  seam
- added stable host-edge proofs for local and server supervisor reports
- exposed the `signal.runtime.device-supervision-boundary` descriptor from
  `signal-supervisor-tools`
- wired the repo-owned acceptance task:
  - `effigy acceptance:device-supervision-boundary --repo .`
- marked `g06.014` complete and moved the active queue to `g06.015`

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime public_runtime_device_supervision_boundary_reports_recovering_and_faulted_runtime_states -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_runtime_device_supervision_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_device_supervision_truth -- --nocapture`
- `cargo test -p signal-supervisor-tools device_supervision_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-device-supervision-boundary --format=json`
- `effigy acceptance:device-supervision-boundary --repo .`

## Deferred scope

- broader backend-matrix breadth, drift detail, duplex mismatch, and endpoint
  topology still belong to `g06.015`
- product-local recovery UX and remote or distributed hardware orchestration
  remain out of scope for this boundary

## Next Task

Continue `g06.015` with Batch 15.1 by freezing the runtime-owned clock-domain
drift, discontinuity, duplex mismatch, and endpoint-topology contract on top of
the closed `g06.014` supervision boundary.
