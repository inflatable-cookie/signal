# 2026-03-18 - g07.017 Runtime Transform-Artifact Baseline Tranche

## Summary

Materialized the first bounded runtime-owned post-warp render, cache, and
transform-artifact receipt family on top of the closed stretch and
marker-analysis seams.

## Work completed

- added `RuntimeTransformArtifactSnapshot` and per-clip
  `RuntimeTransformArtifactClipSnapshot` in
  `crates/signal-runtime/src/interfaces.rs`, including typed readiness,
  invalidation, reuse, cached-media readiness, and artifact identity
- derived the new artifact family from shared clip-processing, stretch,
  marker-analysis, and media-pipeline truth in
  `crates/signal-runtime/src/runtime.rs` instead of introducing host-local
  preview-cache ownership
- threaded the same runtime-owned transform-artifact surface through
  observation, supervisor export, clip-render results, offline-render preview,
  and stable local/server host-edge JSON in
  `crates/signal-runtime/src/interfaces.rs`,
  `crates/signal-runtime/src/runtime.rs`,
  `crates/signal-host-local/src/host.rs`, and
  `crates/signal-host-server/src/host.rs`
- recorded the Batch 17.2 outcome in the `g07.017` roadmap, contract, and
  runtime feature reference so Batch 17.3 is now the explicit next queue

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_transform_artifact_snapshot_derives_from_stretch_and_marker_analysis_baselines -- --nocapture`
- `cargo test -p signal-host-local local_host_shared_report_surfaces_runtime_transform_artifact_baseline -- --nocapture`
- `cargo test -p signal-host-server server_host_shared_report_surfaces_runtime_transform_artifact_baseline -- --nocapture`

## Deferred

- focused public runtime, supervisor-tools, and stable host-edge proof seam
- fuller cache-retention, artifact-reuse orchestration, and preview-cache
  policy depth
- low-latency audition, editor-grade transform reuse, and broader storage
  backend breadth

## Next task

Continue `g07.017` with Batch 17.3 by adding focused downstream-style proof
that the widened post-warp render, cache, transform-artifact readiness,
invalidation, and reuse receipts remain consumable through shared runtime,
supervisor, and stable host-edge surfaces without host-local preview-cache
reconstruction.
