# 2026-03-19 - g07.018 Preview-Transform Boundary Closure And g07.019 Handoff

## Summary

Closed the bounded low-latency audition, scrub, and preview-transform consumer
seam across public runtime, both stable host edges, and
`signal-supervisor-tools`, then advanced the active queue to `g07.019`.

## Work completed

- added focused downstream-style runtime proof for preview-transform service
  class, readiness, degraded-state, fallback, active audition, and
  scrub-supported truth in
  `crates/signal-runtime/tests/public_contract_boundary.rs`
- added matching stable host-edge proofs in
  `crates/signal-host-local/tests/public_host_edge_boundary.rs` and
  `crates/signal-host-server/tests/public_host_edge_boundary.rs`
- exposed the machine-readable
  `signal.runtime.preview-transform-boundary` descriptor in
  `crates/signal-supervisor-tools/src/main.rs`
- wired the repo-owned rerun lane `acceptance:preview-transform-boundary` in
  `effigy.toml`
- closed the `g07.018` roadmap and contract, activated `g07.019`, and rolled
  the shared roadmap and contract pointers forward

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime public_runtime_preview_transform_boundary_reports_runtime_owned_preview_truth -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_runtime_preview_transform_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_preview_transform_truth -- --nocapture`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_preview_transform_boundary_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools preview_transform_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-preview-transform-boundary --format=json`
- `effigy acceptance:preview-transform-boundary --repo .`
- `effigy test --plan --repo .`
- `git diff --check`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy qa:docs --repo .`

## Deferred

- fuller low-latency preview execution and device-routing behavior
- browser or editor preview workflow depth
- richer preview retention, browsing, and cache policy

## Next task

Continue `g07.019` with Batch 19.1 by freezing the integrated acceptance
contract for the widened multichannel, Linux, time-stretch, and
control-surface surfaces before harness depth widens.
