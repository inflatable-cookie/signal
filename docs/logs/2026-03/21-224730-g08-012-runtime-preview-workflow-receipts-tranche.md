# 2026-03-21 - g08.012 runtime preview workflow receipts tranche

## Summary

Completed `g08.012` Batch 12.2 by materializing the first runtime-owned
preview-browser queue, media audition orchestration, and transform-scheduling
receipts on the existing preview-transform seam.

## Work completed

- widened `RuntimePreviewTransformServiceSnapshot` with a typed
  `preview_workflow` summary that captures bounded queue posture, audition
  orchestration, and transform-scheduling truth from runtime-owned preview and
  media-service state
- re-exported the new preview-workflow receipt family from `signal-runtime`
- widened focused public runtime and stable local or server host-edge proofs
  so they all consume the same bounded preview-workflow truth without
  browser-local queue or host-local audition scheduler reconstruction
- recorded Batch 12.2 across the contract, roadmap, and feature-reference
  trail

## Validation

- `effigy test --plan`
- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_preview_transform_snapshot_derives_from_stretch_and_artifact_baselines -- --nocapture`
- `cargo test -p signal-runtime public_runtime_preview_transform_boundary_reports_runtime_owned_preview_truth -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_runtime_preview_transform_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_preview_transform_truth -- --nocapture`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Next task

Continue `g08.012` with Batch 12.3 by proving the widened preview-workflow
seam through shared runtime, supervisor, and stable host-edge surfaces
without introducing a browser-local queue or host-local audition scheduler
shell.
