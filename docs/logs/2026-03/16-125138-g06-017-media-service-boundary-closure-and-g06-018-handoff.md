# 2026-03-16 12:51:38 UTC - g06.017 Media-Service Boundary Closure And g06.018 Handoff

## Summary

Closed `g06.017` by proving the widened runtime-owned media indexing,
waveform readiness, preview state, and invalidation receipt family stays
consumable through public runtime surfaces, both stable host edges, and a
machine-readable `signal-supervisor-tools` boundary descriptor. The active
queue now moves to `g06.018`.

## Work completed

- added the downstream-style runtime proof:
  - `crates/signal-runtime/tests/public_contract_boundary.rs`
- added stable host-edge proofs for:
  - local shared media-service truth
  - server shared media-service truth
- added the machine-readable `signal.runtime.media-service-boundary`
  descriptor and supporting CLI/tests in:
  - `crates/signal-supervisor-tools/src/main.rs`
- added the repo-owned acceptance task:
  - `effigy acceptance:media-service-boundary --repo .`
- closed the `g06.017` roadmap and contract trail and activated `g06.018`

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime public_runtime_media_service_boundary_reports_runtime_owned_readiness_and_invalidation_truth -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_runtime_media_service_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_media_service_truth -- --nocapture`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_media_service_boundary_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools media_service_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-media-service-boundary --format=json`
- `effigy acceptance:media-service-boundary --repo .`

## Deferred scope

- richer metadata extraction and broader library-service depth remain outside
  the closed `g06.017` boundary and now belong to `g06.018`
- this closes the bounded shared media-service seam, not product-local browser,
  collection, tagging, or editorial media-management workflows

## Next Task

Continue `g06.018` with Batch 18.1 by freezing the first reusable
analysis-metadata and library-service descriptor family on top of the closed
media-service boundary.
