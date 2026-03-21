# 2026-03-19 18:27:15 - g08.004 LV2 extension boundary closure and g08.005 handoff

## Summary

Closed `g08.004` by widening the existing LV2 consumer boundary to the new
runtime-owned worker, URID, patch, and extension-negotiation seam.

## What changed

- widened `signal-supervisor-tools` `lv2-boundary` output so it now points at
  `docs/contracts/055-lv2-worker-urid-patch-and-extension-negotiation-contract.md`
  instead of the older baseline-only LV2 contract
- updated the repo-owned `acceptance:lv2-boundary` lane to require:
  - public runtime LV2 extension proof
  - stable local host-edge LV2 extension proof
  - stable server host-edge LV2 extension proof
  - machine-readable supervisor boundary output
- closed `g08.004` in the roadmap and contract trail
- opened `g08.005` as the next active milestone for complex plugin pin-matrix
  and dynamic bus-negotiation depth

## Validation

- `cargo fmt --all`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_lv2_boundary_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools lv2_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-lv2-boundary --format=json`
- `effigy acceptance:lv2-boundary`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Residual risk

This closes the bounded LV2 extension consumer seam, not full atom-schema,
custom extension, UI, or worker-execution depth. Those remain later Linux
plugin and workflow work.

## Next Task

Continue `g08.005` with Batch 5.1 by freezing the first runtime-owned complex
plugin pin-matrix and dynamic bus-negotiation contract on top of the closed
LV2 extension, Linux parity, and live backend seams.
