# 12-231951 g05.002 Host-Edge Boundary Descriptor Tranche

Status: complete
Owner: core-product
Related roadmap: `docs/roadmaps/g05/002-shared-host-convenience-api-and-consumer-edge-contracts.md`

## Summary

Completed `g05.002` Batch 2.2 by making the shared-stable host edge
machine-readable through `signal-supervisor-tools` and a repo-owned acceptance
task.

## Work Completed

- added `--describe-host-edge-boundary` to `signal-supervisor-tools` so the
  stable versus intentionally unstable host-edge split is inspectable as
  Signal-owned JSON or text rather than only prose
- added `acceptance:host-edge-boundary` to `effigy.toml` so the descriptor is
  validated through one repo-owned task
- kept the descriptor aligned with the Batch 2.1 contract by exposing the
  stable shared host edge around host construction, `RuntimeSupervisorApi`, and
  `supervisor_report()`, while keeping host summaries, enriched host reports,
  scenario boot helpers, and local delegated executor helpers explicitly out of
  the stable tier
- rolled the contract, roadmap, and architecture/reference trail forward to
  Batch 2.3

## Validation

- `cargo fmt --all`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_host_edge_boundary_mode`
- `cargo test -p signal-supervisor-tools host_edge_boundary_json_reports_stable_and_unstable_edges`
- `cargo run -p signal-supervisor-tools -- --describe-host-edge-boundary --format=json`
- `effigy acceptance:host-edge-boundary`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Residual Risk

The host-edge boundary is now inspectable, but Batch 2.3 still has to prove a
consumer can rely on the stable shared host edge without reaching for private
host internals or the intentionally unstable summary/helper layer.

## Next Task

Continue `g05.002` with Batch 2.3 by adding a focused consumer-facing proof
that the stable shared host-edge surfaces remain usable without private host
internals or unstable summary helpers.
