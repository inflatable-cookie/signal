# 2026-03-17 - g07.007 Batch 7.3 - LV2 Boundary Closure And g07.008 Handoff

## Summary

Batch 7.3 closes the shared Linux-native LV2 consumer seam across public
runtime, the stable server host edge, and `signal-supervisor-tools`.

## Completed work

- added the public runtime proof for LV2 discovery, lifecycle, transport, and
  Linux-only platform scope
- added the stable server host-edge proof for LV2 discovery and lifecycle
  truth on supervisor export
- added the machine-readable `signal.runtime.lv2-boundary` descriptor and the
  repo-owned `acceptance:lv2-boundary` Effigy lane
- closed `g07.007` and activated `g07.008`

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime public_runtime_lv2_boundary_reports_runtime_owned_discovery_and_lifecycle_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_lv2_baseline_truth -- --nocapture`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_lv2_boundary_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools lv2_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-lv2-boundary --format=json`
- `effigy acceptance:lv2-boundary --repo .`
- `git diff --check`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy qa:docs --repo .`

## Residual risk

`g07.007` closes the bounded LV2 baseline, not richer LV2 worker, UI, patch,
URID, or extension depth. Linux cross-adapter parity and sandbox policy remain
the next active queue in `g07.008`.
