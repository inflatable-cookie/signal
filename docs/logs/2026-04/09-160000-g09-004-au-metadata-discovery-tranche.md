# 2026-04-09 - g09.004 AU metadata discovery tranche

## Summary

Started `g09.004` by replacing scaffold-style AU bundle-name discovery with
bundle-local AU metadata discovery and aligning the host proof surfaces to the
same `.component` metadata contract.

## Work Completed

- added `/crates/signal-plugin-au/src/au_host_adapter/introspection.rs`
  - introduced bundle-local parsing for `signal-au-component.txt`
  - extracts AU identity, component type, subtype, manufacturer, vendor, name,
    version, bounded I/O layout, and feature tags from real `.component`
    bundles
- updated `/crates/signal-plugin-au/src/au_host_adapter/discovery.rs`
  - removed production dependence on bundle-name inference and scaffold lookup
  - now discovers AU plugin records directly from bundle-local metadata
  - deduplicates on `plugin_type_id` after metadata-driven construction
- updated `/crates/signal-plugin-au/src/lib.rs`
  - AU adapter tests now build metadata-backed temp `.component` bundles
  - session coverage now discovers the AU instrument from real scan roots
    instead of direct scaffold lookup
- updated `/crates/signal-host-local/src/host_test_support/setup/scan_roots.rs`
  and `/crates/signal-host-local/tests/support/public_host_edge_plugins.rs`
  - local host internal and public AU proof roots now materialize
    `signal-au-component.txt`
- updated `/crates/signal-host-server/src/host_test_support/setup.rs`
  and `/crates/signal-host-server/tests/support/public_host_edge_plugins.rs`
  - server host internal and public AU proof roots now materialize the same
    metadata-backed AU bundles
- updated `/crates/signal-plugin-au/src/au_host_adapter.rs`
  and `/crates/signal-plugin-au/src/au_host_adapter/scaffold.rs`
  - moved the old AU scaffold helper behind `#[cfg(test)]`
  - limited remaining scaffold helpers to explicit test-only usage
- updated `/docs/roadmaps/g09/004-real-au-discovery-coreaudio-backed-execution-and-macos-proof.md`
  - marked `g09.004` active
  - recorded `Batch 4.2 Tranche 1 Outcome`
  - checked the evidence items actually completed in this tranche

## Validation

- `cargo check -p signal-plugin-au`
- `cargo test -p signal-plugin-au --lib`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_au -- --exact local_shared_host_edge_exports_runtime_au_baseline_truth --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_au -- --exact server_shared_host_edge_exports_runtime_au_baseline_truth --nocapture --test-threads=1`
- `effigy health`

## Outcome

The first macOS plugin seam is now honest: AU discovery identity and descriptor
shape come from bundle-local metadata instead of scaffold name matching. The
remaining fake depth in `g09.004` is no longer AU scan-root truth; it is the
synthetic CoreAudio device layer and the still-bounded AU execution path.
