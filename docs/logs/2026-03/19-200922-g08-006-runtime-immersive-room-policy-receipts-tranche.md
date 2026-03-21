# 2026-03-19 - g08.006 runtime immersive room-policy receipts tranche

## Summary

Closed Batch 6.2 of `g08.006` by materializing the first runtime-owned
immersive object-rendering and room-policy receipts.

## Changes

- widened `signal-runtime` richer-spatial execution receipts so one bounded
  `immersive_room_policy` summary now composes through execution topology,
  plugin-chain stages, and offline-render dependency preview
- added aggregate immersive room-policy counts for runtime topology and
  offline-render preview instead of leaving room-policy truth buried in
  renderer-private fallback behavior
- widened the focused runtime, public runtime, and stable host-edge proofs so
  the same fallback-room-policy receipt is visible without host-local
  reclassification

## Validation

- `cargo fmt --all`
- `effigy test --plan`
- `cargo test -p signal-runtime runtime_observation_and_render_preview_surface_spatial_execution_receipts -- --nocapture`
- `cargo test -p signal-runtime public_runtime_spatial_boundary_reports_runtime_owned_execution_truth -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_runtime_spatial_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_spatial_truth -- --nocapture`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Residual Risk

This tranche closes the first runtime-owned immersive room-policy receipt seam,
not the consumer-facing acceptance boundary. `signal-supervisor-tools` still
describes the earlier richer-spatial contract, and true renderer-backed
immersive execution, monitoring-scene depth, and export packaging remain later
`g08` work.

## Next Task

Continue `g08.006` with Batch 6.3 by proving the widened immersive seam through
shared runtime, supervisor, and stable host-edge surfaces without introducing a
renderer-private room-policy shell.
