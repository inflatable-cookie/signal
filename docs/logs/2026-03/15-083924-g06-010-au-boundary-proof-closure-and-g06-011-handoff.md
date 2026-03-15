# 2026-03-15 08:39:24 UTC - g06.010 AU Boundary Proof Closure And g06.011 Handoff

## Summary

Closed `g06.010` by proving the new AU adapter baseline remains consumable
through shared runtime, supervisor, and stable host-edge surfaces without
adapter-local reconstruction, then moved the active queue to `g06.011`.

## Work completed

- added the machine-readable `signal.runtime.au-boundary` descriptor to
  `signal-supervisor-tools`
- added the repo-owned `effigy acceptance:au-boundary` task
- proved AU discovery and lifecycle truth through:
  - public `signal-runtime` reexports
  - `signal-host-local` stable host edge
  - `signal-host-server` stable host edge
- closed `g06.010` roadmap and contract surfaces
- activated `g06.011` and corrected its stale next-task pointer

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime public_runtime_au_boundary_reports_runtime_owned_discovery_and_lifecycle_truth -- --nocapture`
- `cargo test -p signal-host-local local_shared_host_edge_exports_runtime_au_baseline_truth -- --nocapture`
- `cargo test -p signal-host-server server_shared_host_edge_exports_runtime_au_baseline_truth -- --nocapture`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_au_boundary_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools au_boundary_json_reports_runtime_and_host_edge_proofs -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-au-boundary --format=json`
- `effigy acceptance:au-boundary --repo .`

## Deferred scope

- richer AU parameter-tree, preset, editor, and event-model depth remains later
  cross-adapter work
- this closes the first bounded AU consumer seam, not broader backend
  capability parity or cross-format portability claims

## Next Task

Continue `g06.011` with Batch 11.1 by freezing the backend capability parity,
Linux plugin-support, and cross-adapter conformance contract on top of the now
closed CLAP, VST3, and AU runtime-owned adapter boundaries.
