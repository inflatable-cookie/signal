# 2026-04-08 - g09.003 VST3 module metadata introspection tranche

## Summary

Started `g09.003` by replacing VST3 bundle-name reverse mapping with
bundle-local module metadata introspection while keeping the existing host-edge
VST3 proof surfaces green.

## Work Completed

- added `/crates/signal-plugin-vst3/src/vst3_host_adapter/introspection.rs`
  - introduced bundle-local metadata parsing for
    `signal-vst3-module.txt`
  - supports `plugin_type_id`, class id, optional controller class id, and
    category extraction from real `.vst3` bundles
- updated `/crates/signal-plugin-vst3/src/vst3_host_adapter/discovery.rs`
  - replaced bundle-name reverse mapping with bundle-local metadata reads
  - validates parsed metadata against the current scaffold-known pairing and
    category surface before hydrating the discovered plugin record
- updated `/crates/signal-plugin-vst3/src/vst3_host_adapter/scaffold.rs`
  - added scaffold module metadata records used to validate the new
    introspection path
  - removed the old name-only reverse mapping dependence from discovery
- updated `/crates/signal-plugin-vst3/src/lib.rs`
  - VST3 adapter tests now build temp `.vst3` bundles with explicit
    `signal-vst3-module.txt` metadata files
- updated `/crates/signal-host-local/src/host_test_support/setup/scan_roots.rs`
  and `/crates/signal-host-server/src/host_test_support/setup.rs`
  - host-internal VST3 scan roots now materialize bundle-local metadata files
- updated `/crates/signal-host-local/tests/support/public_host_edge_plugins.rs`
  and `/crates/signal-host-server/tests/support/public_host_edge_plugins.rs`
  - public host-edge VST3 proof bundles now include explicit VST3 module
    metadata instead of relying on bundle-name inference
- updated `/docs/roadmaps/g09/003-real-vst3-discovery-instantiation-and-lifecycle-proof.md`
  - marked `g09.003` active
  - recorded `Batch 3.1 Tranche 1 Outcome`
  - checked the evidence items actually completed in this tranche

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

The first real VST3 realization seam is now in place: discovered plugin
identity comes from bundle-local metadata rather than directory names. The
remaining obvious scaffold in `g09.003` is descriptor hydration and deeper
module/class-factory realization, which still route through scaffold plugin
records after identity has been established.
