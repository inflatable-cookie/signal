# 2026-03-19 - g07.018 runtime preview service baseline tranche

## Summary

Materialized the first runtime-owned low-latency audition, scrub, and
preview-transform service receipt family on top of the closed stretch,
marker-analysis, and transform-artifact seams.

## Delivered

- added `RuntimePreviewTransformServiceSnapshot` and per-clip preview receipts
  to `signal-runtime`
- threaded the same preview service meaning through runtime observation,
  supervisor export, clip-render results, offline render contract preview, and
  both stable host-edge JSON paths
- updated the active `g07.018` roadmap, contract, and shared next pointers to
  Batch 18.3

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_preview_transform_snapshot_derives_from_stretch_and_artifact_baselines -- --nocapture`
- `cargo test -p signal-host-local local_host_shared_report_surfaces_runtime_preview_transform_baseline -- --nocapture`
- `cargo test -p signal-host-server server_host_shared_report_surfaces_runtime_preview_transform_baseline -- --nocapture`
- `effigy test --plan --repo .`
- `git diff --check`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy qa:docs --repo .`

## Deferred

- machine-readable preview boundary and acceptance lane
- downstream-style public runtime and host-edge proof seam
- fuller low-latency preview execution, device routing, and browser workflow depth

## Next Task

Continue `g07.018` with Batch 18.3 by adding focused downstream-style proof
that low-latency audition, scrub, preview-transform service, readiness,
degraded-state, and fallback receipts remain consumable through shared runtime,
supervisor, preview, and stable host-edge surfaces without host-local preview
playback reconstruction.
