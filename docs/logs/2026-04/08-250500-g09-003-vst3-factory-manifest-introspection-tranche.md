# 2026-04-08 - g09.003 VST3 factory manifest introspection tranche

## Summary

Completed the next `g09.003` VST3 discovery tranche by replacing
scaffold-backed processor/controller pairing checks with a bundle-local
class-factory manifest.

## Work Completed

- updated `/crates/signal-plugin-vst3/src/vst3_host_adapter/introspection.rs`
  - added parsing for `signal-vst3-factory.txt`
  - introduced bundle-local factory class records for component/controller
    exports
- updated `/crates/signal-plugin-vst3/src/vst3_host_adapter/discovery.rs`
  - real VST3 discovery now validates component/controller pairing, category,
    and display name against the parsed bundle-local factory manifest instead
    of scaffold metadata
- updated `/crates/signal-plugin-vst3/src/vst3_host_adapter/scaffold.rs`
  - added test-only factory metadata generation so adapter tests can emit the
    same class-factory contract that production discovery now expects
- updated `/crates/signal-plugin-vst3/src/lib.rs`
  - adapter test bundles now write both `signal-vst3-module.txt` and
    `signal-vst3-factory.txt`
- updated VST3 temp bundle helpers in:
  - `/crates/signal-host-local/tests/support/public_host_edge_plugins.rs`
  - `/crates/signal-host-server/tests/support/public_host_edge_plugins.rs`
  - `/crates/signal-host-local/src/host_test_support/setup/scan_roots.rs`
  - `/crates/signal-host-server/src/host_test_support/setup.rs`
  - all VST3 proof bundles now emit the factory manifest alongside the richer
    module metadata
- updated `/docs/roadmaps/g09/003-real-vst3-discovery-instantiation-and-lifecycle-proof.md`
  - added `Batch 3.1 Tranche 3 Outcome`
  - advanced the next task from discovery realization into instantiation and
    lifecycle depth

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

The real VST3 discovery path now depends on bundle-local identity,
descriptor-shape, and class-factory export manifests rather than scaffold
records. The remaining `g09.003` work is now centered on real component
loading, instantiation, and lifecycle/state execution behind that discovery
contract.
