# 2026-04-10 - g09.015 VST3 Moduleinfo And Browser Reactivation

## Summary

Closed the remaining VST3 discovery shim seam by replacing Signal-specific
`.txt` metadata with official `moduleinfo.json` parsing plus real
module/class-factory fallback, then reactivated the plugin capability browser
batch now that CLAP, AU, and VST3 installed-plugin discovery are honest enough
for execution.

## Work Completed

- replaced the VST3 production discovery path in
  `crates/signal-plugin-vst3/src/vst3_host_adapter/introspection.rs` so it now:
  - reads official `moduleinfo.json` when present
  - falls back to real module loading and `GetPluginFactory` class inspection
    when moduleinfo is absent
  - derives bounded plugin identity from real bundle, module, and class data
    instead of `signal-vst3-module.txt` / `signal-vst3-factory.txt`
- widened the internal VST3 model and session validation so discovery is
  bundle-wide and can carry real component category through instantiation
- migrated repo-owned VST3 fixture and proof roots in:
  - `crates/signal-plugin-vst3/src/lib.rs`
  - `crates/signal-host-local/src/host_test_support/setup/scan_roots.rs`
  - `crates/signal-host-server/src/host_test_support/setup.rs`
  - `crates/signal-host-local/tests/support/public_host_edge_plugins.rs`
  - `crates/signal-host-server/tests/support/public_host_edge_plugins.rs`
  - `crates/signal-host-local/src/host_support/demo.rs`
  - `crates/signal-host-server/src/host_support/demo.rs`
  onto real `Contents/Info.plist` plus official `moduleinfo.json`
- confirmed the active Signal VST3 paths no longer contain the old `.txt` shim
- closed `046` and reactivated `043`

## Validation Run

- `cargo check -p signal-plugin-vst3`
- `cargo test -p signal-plugin-vst3 --lib`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_vst3 -- --exact local_shared_host_edge_exports_runtime_vst3_baseline_truth --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_vst3 -- --exact server_shared_host_edge_exports_runtime_vst3_baseline_truth --nocapture --test-threads=1`
- `effigy qa:docs`
- `effigy qa:northstar`

## Notes

The VST3 adapter now prefers official moduleinfo for rich class/category truth.
The real factory fallback remains in place for installed bundles that do not
ship moduleinfo, but the repo-owned proof roots intentionally moved onto the
official bundle surfaces rather than private metadata files.

## Next Task

Continue the reopened strict `g09` lane from
`docs/roadmaps/g09/batch-cards/043-g09-015-plugin-capability-browser-bootstrap.md`.
