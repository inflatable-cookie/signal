# 2026-03-18 - g07.017 Transform-Artifact Boundary Closure And g07.018 Handoff

## Summary

Closed the bounded post-warp render, cache, and transform-artifact consumer
seam across public runtime, both stable host edges, and
`signal-supervisor-tools`, then advanced the active queue to `g07.018`.

## Work completed

- added focused downstream-style runtime proof for transform-artifact
  readiness, invalidation, cached-media readiness, and reuse in
  `crates/signal-runtime/tests/public_contract_boundary.rs`
- added matching stable host-edge proofs in
  `crates/signal-host-local/tests/public_host_edge_boundary.rs` and
  `crates/signal-host-server/tests/public_host_edge_boundary.rs`
- exposed the machine-readable
  `signal.runtime.transform-artifact-boundary` descriptor in
  `crates/signal-supervisor-tools/src/main.rs`
- wired the repo-owned rerun lane `acceptance:transform-artifact-boundary` in
  `effigy.toml`
- closed the `g07.017` roadmap and contract, activated `g07.018`, and rolled
  the shared roadmap and contract pointers forward

## Validation

- `cargo fmt --all`
- `effigy test --plan --repo .`
- `cargo test -p signal-runtime public_runtime_transform_artifact_boundary_reports_runtime_owned_artifact_truth -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_runtime_transform_artifact_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_transform_artifact_truth -- --nocapture`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_transform_artifact_boundary_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools transform_artifact_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-transform-artifact-boundary --format=json`
- `effigy acceptance:transform-artifact-boundary --repo .`
- `git diff --check`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy qa:docs --repo .`

## Deferred

- fuller cache-retention, storage-policy, and artifact reuse orchestration
- low-latency audition, scrub, and preview-transform service depth
- product-local cache browser or transform management workflow

## Next task

Continue `g07.018` with Batch 18.1 by freezing the low-latency audition,
scrub, and preview-transform service contract on top of the closed stretch and
transform-artifact boundaries before runtime preview-service depth widens.
