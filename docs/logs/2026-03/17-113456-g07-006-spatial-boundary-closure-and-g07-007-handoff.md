# 2026-03-17 - g07.006 spatial boundary closure and g07.007 handoff

## Summary

Closed `g07.006` by proving the widened richer-spatial receipt family through
public runtime, both stable host edges, and the existing machine-readable
spatial boundary descriptor.

This tranche turns the Batch 6.2 runtime work into a real consumer seam rather
than leaving surround-bed, mix-policy, render-scope, and expanded-fallback
meaning as runtime-only detail.

## Key changes

- widened the public runtime proof to assert richer-spatial counts and fields
  on execution-topology, plugin-chain, and offline-render preview surfaces
- widened both stable host-edge proofs to assert the same richer-spatial truth
  on supervisor export
- repointed `signal-supervisor-tools` spatial boundary metadata to the
  `g07.006` contract and updated its surface anchors and deferred-scope text to
  describe the richer substrate rather than the earlier baseline-only seam
- closed `g07.006` in roadmap/docs state and activated `g07.007`

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime public_runtime_spatial_boundary_reports_runtime_owned_execution_truth -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_runtime_spatial_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_spatial_truth -- --nocapture`
- `cargo test -p signal-supervisor-tools spatial_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-spatial-boundary --format=json`
- `effigy acceptance:spatial-boundary --repo .`

## Residual risk

`g07.006` is closed as a bounded substrate milestone, not as a full immersive
rendering program. True object rendering, richer renderer breadth, and room or
deployment policy remain later `g07` work.

## Next Task

Continue `g07.007` with Batch 7.1 by mapping LV2-specific discovery,
lifecycle, and Linux-native capability details onto the existing backend-neutral
plugin contract before runtime-owned LV2 realization widens.
