# 2026-04-08 - g09.003 VST3 metadata-driven descriptor hydration tranche

## Summary

Completed the second `g09.003` VST3 discovery tranche by moving descriptor and
default I/O hydration out of scaffold discovered plugin records and into the
bundle-local module metadata path.

## Work Completed

- updated `/crates/signal-plugin-vst3/src/vst3_host_adapter/introspection.rs`
  - expanded `signal-vst3-module.txt` parsing to include vendor, display name,
    version, default audio and MIDI layout, and feature classification
  - added metadata-driven `PluginDescriptor` and `PluginIoLayout` builders
- updated `/crates/signal-plugin-vst3/src/vst3_host_adapter/discovery.rs`
  - real bundle discovery now constructs `Vst3DiscoveredPluginType` directly
    from parsed bundle-local metadata instead of routing back through
    scaffold-discovered plugin records
- updated `/crates/signal-plugin-vst3/src/vst3_host_adapter/scaffold.rs`
  - expanded the remaining scaffold metadata records so test and fallback
    discovery helpers can emit the richer bundle-local metadata file
- updated `/crates/signal-plugin-vst3/src/lib.rs`
  - adapter tests now write richer VST3 metadata through the scaffold metadata
    content helper
- updated the VST3 temp bundle helpers in:
  - `/crates/signal-host-local/tests/support/public_host_edge_plugins.rs`
  - `/crates/signal-host-server/tests/support/public_host_edge_plugins.rs`
  - `/crates/signal-host-local/src/host_test_support/setup/scan_roots.rs`
  - `/crates/signal-host-server/src/host_test_support/setup.rs`
  - all VST3 proof bundles now carry the richer metadata file expected by the
    production discovery path
- updated `/docs/roadmaps/g09/003-real-vst3-discovery-instantiation-and-lifecycle-proof.md`
  - added `Batch 3.1 Tranche 2 Outcome`
  - advanced the next task deeper into real VST3 module and factory
    realization

## Validation

- `cargo check -p signal-plugin-vst3`
- `cargo test -p signal-plugin-vst3 --lib`
- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_vst3`
- `cargo test -p signal-host-server --test public_host_edge_vst3`
- `effigy health`

## Outcome

The real VST3 discovery path no longer depends on scaffold discovered plugin
records after bundle identity has been established. The remaining `g09.003`
work is now more clearly about real module/class-factory realization and
runtime lifecycle depth, not descriptor hydration.
