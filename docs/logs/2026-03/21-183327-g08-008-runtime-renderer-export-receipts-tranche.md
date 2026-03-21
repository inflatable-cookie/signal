# 2026-03-21 - g08.008 runtime renderer capability and immersive export receipts tranche

## Summary

Completed `g08.008` Batch 8.2 by promoting renderer-capability negotiation and
immersive export posture into the shared runtime spatial seam instead of
leaving that meaning as contract-only prose beneath immersive room-policy and
deployment-monitoring.

## Changes

- widened `crates/signal-runtime/src/interfaces.rs` with the new
  `RuntimeRendererImmersiveExportSummary` family and threaded it through the
  shared spatial execution surface
- added renderer-capability and immersive-export aggregate counts to execution
  topology and offline render dependency preview
- re-exported the new renderer/export receipt family from
  `crates/signal-runtime/src/lib.rs`
- widened the focused runtime proof in `crates/signal-runtime/src/runtime.rs`
- widened the downstream-style public runtime and stable host-edge proofs in
  `crates/signal-runtime/tests/public_contract_boundary.rs`,
  `crates/signal-host-local/tests/public_host_edge_boundary.rs`, and
  `crates/signal-host-server/tests/public_host_edge_boundary.rs`
- updated the `g08.008` contract, roadmap, and shared reference indexes to
  record Batch 8.2 as complete and move the queue to Batch 8.3

## Validation

- `effigy test --plan`
- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_observation_and_render_preview_surface_spatial_execution_receipts -- --nocapture`
- `cargo test -p signal-runtime public_runtime_spatial_boundary_reports_runtime_owned_execution_truth -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_runtime_spatial_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_spatial_truth -- --nocapture`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Residual Risk

This closes the first runtime-owned renderer/export receipt seam, not the
consumer-facing supervisor acceptance boundary. The current proof still
reflects the bounded fallback surround path, not deeper renderer-backed export
or packaging depth.

## Next Task

Continue `g08.008` with Batch 8.3 by proving the widened renderer-capability
and immersive export seam through shared runtime, supervisor, and stable
host-edge surfaces without introducing a renderer-private export shell.
