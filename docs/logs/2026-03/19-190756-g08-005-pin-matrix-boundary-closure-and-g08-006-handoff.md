# 2026-03-19 19:07:56 - g08.005 pin-matrix boundary closure and g08.006 handoff

## Summary

Closed `g08.005` by widening the existing complex plugin-I/O consumer boundary
to the new runtime-owned pin-matrix and dynamic bus-negotiation seam.

## What changed

- widened `signal-supervisor-tools` `complex-io-boundary` output so it now
  points at
  `docs/contracts/056-complex-plugin-pin-matrix-and-dynamic-bus-negotiation-contract.md`
  instead of the older baseline-only complex-I/O contract
- updated the machine-readable boundary to describe the new
  `plugin_pin_matrix_snapshot` surface alongside the existing complex-I/O
  discovery, plugin-chain, render-preview, and stable host-edge proof seam
- kept the repo-owned `acceptance:complex-io-boundary` lane reusable rather
  than inventing a second overlapping plugin-routing descriptor
- closed `g08.005` in the roadmap and contract trail
- opened `g08.006` as the next active milestone for immersive object rendering
  and room-policy substrate work

## Validation

- `effigy tasks`
- `cargo test -p signal-supervisor-tools complex_io_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-complex-io-boundary --format=json`
- `effigy acceptance:complex-io-boundary`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Residual risk

This closes the bounded plugin-routing consumer seam, not full format-specific
pin schemas, product-local pin-matrix workflows, or immersive renderer policy.
Those remain later `g08` work.

## Next Task

Continue `g08.006` with Batch 6.1 by freezing the first runtime-owned
immersive object rendering and room-policy contract on top of the closed
plugin-routing, LV2 extension, Linux parity, and live backend seams.
