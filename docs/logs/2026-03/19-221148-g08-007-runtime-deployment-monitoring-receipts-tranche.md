# 2026-03-19 - g08.007 runtime deployment and monitoring receipts tranche

## Summary

Completed `g08.007` Batch 7.2 by promoting speaker deployment, fold-down, and
monitoring-scene posture into the shared runtime spatial seam instead of
leaving that meaning as contract-only prose beneath immersive room-policy.

## Changes

- re-exported the new deployment and monitoring receipt family from
  `crates/signal-runtime/src/lib.rs` so downstream consumers can import the
  bounded seam directly
- widened the focused runtime spatial proof in
  `crates/signal-runtime/src/runtime.rs` to assert deployment class, fold-down
  policy, monitoring-scene class and authority, monitoring outcome, and the new
  topology or offline-preview counters
- widened the downstream-style public runtime proof in
  `crates/signal-runtime/tests/public_contract_boundary.rs`
- widened the stable local and server host-edge proofs in
  `crates/signal-host-local/tests/public_host_edge_boundary.rs` and
  `crates/signal-host-server/tests/public_host_edge_boundary.rs`
- updated the `g08.007` contract, roadmap, and shared reference indexes to
  record Batch 7.2 as complete and move the queue to Batch 7.3

## Validation

- `effigy test --plan`
- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_observation_and_render_preview_surface_spatial_execution_receipts -- --nocapture`
- `cargo test -p signal-runtime public_runtime_spatial_boundary_reports_runtime_owned_execution_truth -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_runtime_spatial_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_spatial_truth -- --nocapture`

## Next Task

Continue `g08.007` with Batch 7.3 by proving the widened deployment and
monitoring seam through shared runtime, supervisor, and stable host-edge
surfaces without introducing a renderer-private monitoring shell.
