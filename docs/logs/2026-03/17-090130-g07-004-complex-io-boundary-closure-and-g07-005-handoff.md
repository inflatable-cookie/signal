# 2026-03-17 - g07.004 Batch 4.3 - Complex I/O Boundary Closure And g07.005 Handoff

## Summary

Batch 4.3 closed the bounded complex plugin-I/O consumer seam across public
runtime, both stable host edges, and `signal-supervisor-tools`.

## Completed work

- added public runtime proof for complex plugin-I/O discovery, plugin-chain,
  and offline render dependency preview receipts
- added stable local and server host-edge proofs for forwarded complex
  plugin-I/O, multi-output instrument, and bus-capable FX topology
- added the machine-readable `signal.runtime.complex-io-boundary` descriptor
  and wired `effigy acceptance:complex-io-boundary`
- closed `g07.004` and promoted `g07.005` as the next active queue

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime public_runtime_complex_io_boundary_reports_runtime_owned_topology_truth -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_runtime_complex_io_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_complex_io_truth -- --nocapture`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_complex_io_boundary_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools complex_io_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-complex-io-boundary --format=json`
- `effigy acceptance:complex-io-boundary --repo .`

## Residual risk

`g07.004` closes the bounded complex plugin-I/O seam, not broader spatial
routing, immersive bus behavior, or product-local pin-matrix policy. Those
remain later `g07` work beginning with spatial adapter execution in `g07.005`.
