# 2026-03-17 - g07.006 runtime expanded spatial receipts tranche

## Summary

Completed Batch 6.2 of `g07.006` by materializing the first bounded richer
spatial receipt layer on top of the closed spatial baseline.

The widened runtime path now carries explicit surround-bed class, object-role
placeholder, object count, mix policy, render scope, and expanded fallback
meaning through execution, observation, plugin-chain, offline-render preview,
and shared host-report surfaces.

## Key changes

- widened `RuntimeSpatialExecutionSummary` so richer spatial meaning stays on
  the existing shared runtime seam instead of creating a parallel immersive
  model
- added runtime-owned surround-bed, object-aware, and expanded-fallback counts
  to execution-topology and offline-render dependency preview summaries
- kept the current bounded behavior explicit:
  - stereo `StereoBalance` stages realize `StereoBed`, `BedOnly`, and
    `BedRender`
  - canonical surround stages surface `CanonicalSurroundBed` plus
    `CollapseToBaselineSpatial` as an expanded fallback instead of silent
    non-stereo bypass
- aligned local and server shared host reports to the same widened receipt
  model without introducing host-local or renderer-local spatial ownership

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_observation_and_render_preview_surface_spatial_execution_receipts -- --nocapture`
- `cargo test -p signal-host-local local_host_shared_report_surfaces_runtime_spatial_execution_baseline -- --nocapture`
- `cargo test -p signal-host-server server_host_shared_report_surfaces_runtime_spatial_execution_baseline -- --nocapture`
- `git diff --check`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy qa:docs --repo .`

## Residual risk

This tranche makes richer spatial receipts real, but the path is still
deliberately narrow. Object-aware execution remains explicit-but-zero, and the
public runtime, supervisor, and stable host-edge proof seam still belongs to
Batch 6.3.

## Next Task

Continue `g07.006` with Batch 6.3 by adding focused downstream-style proof
that the widened surround-bed, object-role, mix-policy, render-scope, and
expanded-fallback receipts remain consumable through shared runtime,
supervisor, and stable host-edge surfaces without host-local or renderer-local
spatial reinterpretation.
