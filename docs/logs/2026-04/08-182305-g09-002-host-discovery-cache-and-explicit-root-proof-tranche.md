# 2026-04-08 18:23:05 - g09.002 host discovery cache and explicit-root proof tranche

## Summary

Finished the host-side half of `g09.002` Batch 2.2.

Both runtime hosts now cache discovered AU/VST3/LV2 adapter records from
`start_plugin_scan(...)` and reuse those cached discoveries during sandbox
ensure, instead of re-entering fixture-backed `discover_plugin_type(...)`
lookups at ensure time.

The affected host proof surfaces also moved to explicit temporary plugin roots:

- host-internal plugin scan coverage in `signal-host-local` and
  `signal-host-server`
- public VST3, AU, LV2, cross-adapter parity, generic event, and LV2 extension
  tests

## Files

- `crates/signal-host-local/src/host.rs`
- `crates/signal-host-local/src/host_api.rs`
- `crates/signal-host-local/src/host_support/discovery.rs`
- `crates/signal-host-local/src/host_support/sandbox_sessions.rs`
- `crates/signal-host-local/src/host_test_support.rs`
- `crates/signal-host-local/src/host_test_support/setup.rs`
- `crates/signal-host-local/src/host_test_support/setup/scan_roots.rs`
- `crates/signal-host-local/src/host_tests/reports/report_surfaces/plugin_receipts/plugin_scan.rs`
- `crates/signal-host-local/tests/public_host_edge_vst3.rs`
- `crates/signal-host-local/tests/public_host_edge_au.rs`
- `crates/signal-host-local/tests/public_host_edge_cross_adapter_parity.rs`
- `crates/signal-host-local/tests/public_host_edge_plugin_events.rs`
- `crates/signal-host-local/tests/support/public_host_edge_plugins.rs`
- `crates/signal-host-server/src/host.rs`
- `crates/signal-host-server/src/host_support/discovery.rs`
- `crates/signal-host-server/src/host_support/sandbox_sessions.rs`
- `crates/signal-host-server/src/host_test_support.rs`
- `crates/signal-host-server/src/host_test_support/setup.rs`
- `crates/signal-host-server/src/host_tests/plugin_scan/au.rs`
- `crates/signal-host-server/src/host_tests/plugin_scan/lv2.rs`
- `crates/signal-host-server/src/host_tests/plugin_scan/vst3.rs`
- `crates/signal-host-server/tests/public_host_edge_vst3.rs`
- `crates/signal-host-server/tests/public_host_edge_au.rs`
- `crates/signal-host-server/tests/public_host_edge_lv2.rs`
- `crates/signal-host-server/tests/public_host_edge_cross_adapter_parity.rs`
- `crates/signal-host-server/tests/public_host_edge_plugin_events.rs`
- `crates/signal-host-server/tests/public_host_edge_lv2_extension.rs`
- `crates/signal-host-server/tests/support/public_host_edge_plugins.rs`
- `docs/roadmaps/g09/002-shared-plugin-hosting-substrate-and-hardened-sandbox-execution.md`
- `docs/roadmaps/g09/README.md`

## Validation

- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `effigy health`
- `cargo test -p signal-host-local --test public_host_edge_vst3`
- `cargo test -p signal-host-local --test public_host_edge_au`
- `cargo test -p signal-host-local --test public_host_edge_cross_adapter_parity`
- `cargo test -p signal-host-local --test public_host_edge_plugin_events`
- `cargo test -p signal-host-server --test public_host_edge_vst3`
- `cargo test -p signal-host-server --test public_host_edge_au`
- `cargo test -p signal-host-server --test public_host_edge_lv2`
- `cargo test -p signal-host-server --test public_host_edge_cross_adapter_parity`
- `cargo test -p signal-host-server --test public_host_edge_plugin_events`
- `cargo test -p signal-host-server --test public_host_edge_lv2_extension`

## Notes

`cargo test -p signal-host-local --lib --tests --no-run` and
`cargo test -p signal-host-server --lib --tests --no-run` still fail because of
pre-existing unresolved split-test module paths in the host lib-test trees.
That breakage is outside this tranche and was left untouched.

## Next Task

Finish `g09.002` Batch 2.2 by removing the remaining fixture-metadata
dependency inside AU, VST3, and LV2 adapter discovery so production discovery
no longer relies on test fixture modules after matching real filesystem
entries.
