# 2026-03-12 20:58:23 UTC - g04.005 plugin scan receipts and delegated format identity tranche

## Summary

Moved `g04.005` Batch 5.2 forward by promoting the first plugin discovery and
backend-identity depth into Signal-owned runtime surfaces rather than leaving
scan intent and delegated stage identity in host-local bookkeeping.

## What changed

- added typed runtime-owned plugin scan/discovery receipts in
  `crates/signal-runtime/src/interfaces.rs` and `crates/signal-runtime/src/runtime.rs`
- widened `PluginScanRequest` and `PluginSandboxSpec` to carry typed
  `PluginFormat` rather than stringly adapter hints
- threaded typed plugin format and plugin type identity through runtime plugin
  sandbox snapshots, recall payload, offline plugin execution boundaries, and
  delegated execution stage requests
- updated `signal-host-local` and `signal-host-server` to record plugin scan
  and sandbox spec state into `signal-runtime` and restored the current offline
  execution supervisor delegations while touching the impls
- extended focused proofs so runtime and host-local reports expose the new
  runtime-owned discovery receipt and plugin-format identity

## Why it matters

This tranche closes one of the most obvious ownership leaks in the current
plugin path. Hosts still initiate scans and sandbox creation, but the
authoritative meaning of scan filters and delegated stage backend identity now
lives in typed Signal-owned runtime DTOs instead of adapter-local or
host-local inference.

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_plugin_discovery_snapshot_and_reports_surface_typed_scan_filters`
- `cargo test -p signal-runtime runtime_prepare_offline_plugin_execution_boundary_surfaces_runtime_owned_stage_contracts`
- `cargo test -p signal-host-local local_host_shared_report_surfaces_topology_aware_host_io`
- `cargo test -p signal-host-server --no-run`
- `git diff --check`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`

## Deferred

- discovered-plugin catalogs are still shallow: runtime now owns roots/filter
  receipts, but it does not yet export richer backend-neutral discovered-plugin
  result records
- backend-neutral capability projection beyond current CLAP-first format
  identity is still deferred
- broader conformance proof that downstream consumers can stay off host-local
  plugin reconstruction still belongs to later `g04.005` work

## Next

Continue `g04.005` Batch 5.2 by promoting discovered-plugin catalog and
capability detail into the new runtime-owned receipt family, then prove one
broader consumer path still avoids host-local reconstruction.
