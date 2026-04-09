# 2026-04-09 - g09.005 - LV2 discovery diagnostics tranche

## Summary

Batch 5.1 Tranche 2 made malformed and unsupported LV2 bundle outcomes
runtime-owned instead of silently skipped.

## What changed

- added typed LV2 discovery diagnostics for malformed manifests and unsupported
  required features in `signal-plugin-lv2`
- threaded LV2 diagnostics into `signal-runtime` plugin scan receipts as
  runtime-owned discovery diagnostics
- updated the server host scan path to export those diagnostics through stable
  observation and supervisor reports
- widened the runtime and server-host public LV2 proofs to assert the exported
  diagnostic truth

## Validation

- `cargo test -p signal-plugin-lv2 --lib`
- `cargo test -p signal-runtime --test public_contract_boundary_lv2 -- --exact public_runtime_lv2_boundary_reports_runtime_owned_discovery_and_lifecycle_truth --nocapture --test-threads=1`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-server --test public_host_edge_lv2 -- --exact server_shared_host_edge_exports_runtime_lv2_baseline_truth --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_lv2_extension -- --exact server_shared_host_edge_exports_runtime_lv2_extension_truth --nocapture --test-threads=1`
- `cargo check -p signal-runtime`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Next Task

Continue `g09.005` with one meaningful extension-baseline batch: turn the
metadata-backed LV2 extension summary into a bounded adapter-owned negotiation
and lifecycle path, starting with explicit URID, worker, and patch posture
records during sandbox preparation, then prove that through the stable
server-host LV2 and LV2-extension public lanes.
