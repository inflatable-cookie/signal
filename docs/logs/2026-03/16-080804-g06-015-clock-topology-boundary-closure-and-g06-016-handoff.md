# 2026-03-16 08:08:04 UTC - g06.015 Clock Topology Boundary Closure And g06.016 Handoff

## Summary

Closed `g06.015` by proving the runtime-owned clock-topology boundary across
public runtime DTOs, the stable local host edge, and a machine-readable
supervisor-tools descriptor. This batch makes drift, discontinuity, duplex
mismatch, and endpoint-topology meaning consumable without backend-private
clock heuristics or product-local topology reconstruction.

## Work completed

- added the downstream-style runtime proof for drift, duplex-mismatch, and
  endpoint-topology receipts on `RuntimeHostObservationReport` and
  `RuntimeHostSupervisorReport`
- added the stable local host-edge proof for steady and explicit faulted
  clock-topology export through `LocalRuntimeHost::host_supervisor_report()`
- exposed the machine-readable
  `signal.runtime.clock-topology-boundary` descriptor from
  `signal-supervisor-tools`
- wired the repo-owned acceptance task:
  - `effigy acceptance:clock-topology-boundary --repo .`
- marked `g06.015` complete and moved the active queue to `g06.016`

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime public_runtime_clock_topology_boundary_reports_drift_duplex_and_endpoint_receipts -- --nocapture`
- `cargo test -p signal-host-local --test public_host_edge_boundary local_shared_host_edge_exports_runtime_clock_topology_truth -- --nocapture`
- `cargo test -p signal-host-local local_host_shared_report_surfaces_duplex_ -- --nocapture`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_clock_topology_boundary_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools clock_topology_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-clock-topology-boundary --format=json`
- `effigy acceptance:clock-topology-boundary --repo .`

## Deferred scope

- broader external-I/O, monitoring tap-point, and loopback measurement depth
  now belongs to `g06.016`
- the stable server host edge still does not expose live host-I/O clocking
  receipts directly, so this focused boundary remains local-host centric on the
  host-edge side
- multi-member aggregate detail, deeper drift compensation policy, and broader
  backend-matrix hardware breadth remain later hardware work

## Next Task

Continue `g06.016` with Batch 16.1 by freezing the external-I/O, monitoring
tap-point, and loopback measurement contract on top of the closed `g06.015`
clock-domain and endpoint-topology boundary.
