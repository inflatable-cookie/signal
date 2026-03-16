# 2026-03-16 09:15:46 UTC - g06.016 External-I/O Boundary Closure And g06.017 Handoff

## Summary

Closed `g06.016` by proving the widened runtime-owned external-I/O,
monitoring, tap-point, and loopback receipt family stays consumable through
public runtime surfaces, the stable local host edge, the stable server host
edge, and a machine-readable `signal-supervisor-tools` boundary descriptor.
The active queue now moves to `g06.017`.

## Work completed

- added the downstream-style runtime proof:
  - `crates/signal-runtime/tests/public_contract_boundary.rs`
- added stable host-edge proofs for:
  - direct and explicit faulted local-host external-I/O truth
  - explicit unavailable server-host external-I/O truth
- added the machine-readable `signal.runtime.external-io-boundary` descriptor
  and supporting CLI/tests in:
  - `crates/signal-supervisor-tools/src/main.rs`
- added the repo-owned acceptance task:
  - `effigy acceptance:external-io-boundary --repo .`
- closed the `g06.016` roadmap and contract trail and activated `g06.017`

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime public_runtime_external_io_boundary_reports_runtime_owned_monitor_and_loopback_truth -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_runtime_external_io_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_external_io_truth -- --nocapture`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_external_io_boundary_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools external_io_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-external-io-boundary --format=json`
- `effigy acceptance:external-io-boundary --repo .`

## Deferred scope

- richer measurement-session, calibration, waveform, and preview-service depth
  remain outside the closed `g06.016` boundary
- the stable server host edge still proves explicit unavailable external-I/O
  state rather than a live server-host hardware seam

## Next Task

Continue `g06.018` with Batch 18.1 by freezing the first reusable
analysis-metadata and library-service descriptor family on top of the closed
media-service boundary.
