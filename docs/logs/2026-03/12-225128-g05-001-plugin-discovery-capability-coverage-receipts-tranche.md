# 2026-03-12 - g05.001 plugin discovery capability coverage receipts tranche

## Summary

Completed `g05.001` Batch 1.2 by widening runtime-owned plugin discovery
receipts with aggregated format coverage and backend-neutral capability
coverage, then proving those widened receipts through runtime, public-boundary,
and supervisor-export fixtures.

## Completed Work

- widened `RuntimePluginScanReceipt` and `RuntimePluginDiscoverySnapshot` with:
  - discovered format counts
  - `RuntimePluginFormatCoverageRecord`
  - `RuntimePluginCapabilityCoverageSummary`
- kept those aggregates runtime-owned so consumers do not need to recount raw
  discovered-plugin records or inspect adapter-private scan types
- updated runtime JSON and multiline export paths so the widened receipt family
  is visible through `RuntimeObservationReport` and `RuntimeSupervisorReport`
- widened the focused proof set in `signal-runtime` tests, the downstream-style
  public contract boundary test, and `signal-supervisor-tools` export proof

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_plugin_discovery_snapshot_and_reports_surface_typed_scan_filters`
- `cargo test -p signal-runtime public_runtime_contract_boundary_is_consumable_from_reexports`
- `cargo test -p signal-supervisor-tools export_json_carries_runtime_owned_plugin_discovery_catalog`
- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Residual Risk

The widened receipt family now exposes backend-neutral breadth, but Batch 1.3
still needs to pin the conformance story more strongly across the public runtime
and supervisor export boundaries before later host-edge or packaging milestones
rely on those new aggregates.

## Next Task

Continue `g05.001` with Batch 1.3 by adding focused conformance proofs for the
widened backend-neutral discovery and capability receipt family across public
runtime and supervisor export surfaces.
