# 2026-04-09 - g09.005 - LV2 unavailable fault tranche

## Summary

Batch 5.2 Tranche 2 added a runtime-owned unavailable negotiation failure lane
for LV2 preparation.

## What changed

- added a bounded metadata-backed LV2 prepare-fault contract for unavailable
  worker, URID, or patch negotiation cases
- mapped that LV2 prepare fault into runtime-owned sandbox lifecycle, fault, and
  prepared-negotiation records on the server host path
- widened the stable public LV2-extension host proof to exercise a
  worker-unavailable lane and assert exported `Unavailable` negotiation truth

## Validation

- `cargo check -p signal-plugin-lv2`
- `cargo check -p signal-runtime`
- `cargo check -p signal-host-server`
- `cargo test -p signal-plugin-lv2 --lib`
- `cargo test -p signal-host-server --test public_host_edge_lv2_extension server_shared_host_edge_exports_runtime_lv2_unavailable_negotiation_truth -- --exact --nocapture --test-threads=1`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Next Task

Refocus `g09.005` into the next major seam instead of adding more tiny
fault-mode variants: replace the remaining demo broker path in the LV2 lane with
one bounded real LV2 broker-backed prepare/teardown flow, and carry the new
runtime-owned negotiation and failure records through that path before widening
to execution streaming.
