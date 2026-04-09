# 2026-04-09 - g09.005 LV2 manifest discovery tranche

## Summary

Started `g09.005` and replaced the production LV2 bundle-name shortcut with a
real manifest-backed discovery path.

## Delivered

- added a real LV2 manifest parser in
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-plugin-lv2/src/lv2_host_adapter/introspection.rs`
- rewired production discovery in
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-plugin-lv2/src/lv2_host_adapter/discovery.rs`
  so `.lv2` bundle traversal parses `manifest.ttl` metadata instead of matching
  bundle names back into scaffold records
- updated adapter tests in
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-plugin-lv2/src/lib.rs`
  to use real manifest-backed temp bundles
- updated server internal and public LV2 proof roots in
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-host-server/src/host_test_support/setup.rs`
  and
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-host-server/tests/support/public_host_edge_plugins.rs`
  to emit the same manifest contract
- promoted `/Users/betterthanclay/Dev/projects/signal/docs/roadmaps/g09/005-real-lv2-discovery-extension-negotiation-and-linux-proof.md`
  from `draft` to `active`

## Validation

- `cargo test -p signal-plugin-lv2 --lib`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-server --test public_host_edge_lv2 -- --exact server_shared_host_edge_exports_runtime_lv2_baseline_truth --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_lv2_extension -- --exact server_shared_host_edge_exports_runtime_lv2_extension_truth --nocapture --test-threads=1`

## Notes

- this tranche intentionally leaves malformed LV2 bundle typing and deeper
  worker or lifecycle realization for the next batch
- the scaffold `discover_plugin_type(...)` helper still exists, but production
  root scanning no longer depends on it

## Next Task

Continue `g09.005` with one meaningful discovery-hardening batch: add explicit
malformed and missing-feature LV2 manifest outcomes to the discovery path and
prove them through runtime and server-host receipts, then reassess whether the
next tranche should stay in discovery hardening or move into bounded extension
and lifecycle realization.
