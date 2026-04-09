# 2026-04-09 - g09.005 - LV2 prepared negotiation tranche

## Summary

Batch 5.2 Tranche 1 moved LV2 extension posture from pure discovery inference
into adapter-owned preparation and runtime-owned sandbox lifecycle state.

## What changed

- added a bounded LV2 extension-preparation record to the LV2 adapter session
  plan for worker, URID, patch, and overall negotiation posture
- recorded that preparation truth on the runtime plugin lifecycle snapshot as a
  dedicated LV2 prepared-negotiation record during sandbox prepare
- taught the LV2 extension snapshot to prefer live sandbox-prepared posture when
  a matching LV2 sandbox exists, while retaining discovery fallback for
  scan-only cases
- widened the runtime and server-host public LV2 proof surfaces to assert the
  exported prepared-negotiation truth

## Validation

- `cargo test -p signal-plugin-lv2 --lib`
- `cargo test -p signal-runtime --test public_contract_boundary_lv2 -- --exact public_runtime_lv2_boundary_reports_runtime_owned_discovery_and_lifecycle_truth --nocapture --test-threads=1`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-server --test public_host_edge_lv2 -- --exact server_shared_host_edge_exports_runtime_lv2_baseline_truth --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_lv2_extension -- --exact server_shared_host_edge_exports_runtime_lv2_extension_truth --nocapture --test-threads=1`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Next Task

Continue `g09.005` with one meaningful lifecycle-and-fault batch: add explicit
LV2 preparation or activation failure mapping into runtime-owned lifecycle and
fault receipts, then prove one guarded or unavailable LV2 negotiation lane
through the stable server-host LV2-extension surface before widening into
broker-backed LV2 execution.
