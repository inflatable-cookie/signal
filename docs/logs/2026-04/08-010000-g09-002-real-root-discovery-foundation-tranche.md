# g09.002 Real-Root Discovery Foundation Tranche

Status: recorded
Owner: core-product
Date: 2026-04-08
Related roadmap: `docs/roadmaps/g09/002-shared-plugin-hosting-substrate-and-hardened-sandbox-execution.md`
Related contract: `docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution-contract.md`

## Summary

Replaced the old fixture-id scan shortcut in the non-CLAP adapter discovery
path with real filesystem traversal while keeping the existing runtime discovery
receipt boundary stable.

## What Landed

- `signal-plugin-vst3`
  - `discover_plugins_for_roots(...)` now scans actual `.vst3` entries under
    requested roots and records real module paths
- `signal-plugin-au`
  - `discover_plugins_for_roots(...)` now scans actual `.component` entries
    under requested roots and records real bundle paths
- `signal-plugin-lv2`
  - `discover_plugins_for_roots(...)` now scans actual `.lv2` bundles under
    requested roots and records real bundle and manifest paths
- the adapter unit tests now create temporary plugin-root directories instead of
  relying on magical system-root fixture behavior

## Still Open In This Batch

- `discover_plugin_type(...)` remains fixture-backed for now and still supports
  the current sandbox ensure path
- host proof tests still need migration to explicit temporary scan roots
- the synthetic sandbox binary remains untouched for the later Batch 2.3 work

## Validation

- `cargo test -p signal-plugin-vst3 --lib`
- `cargo test -p signal-plugin-au --lib`
- `cargo test -p signal-plugin-lv2 --lib`
- `cargo check -p signal-plugin-vst3`
- `cargo check -p signal-plugin-au`
- `cargo check -p signal-plugin-lv2`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `effigy health`

## Next Task

Finish the rest of `g09.002` Batch 2.2 by migrating the affected host proof
tests and shared host scan surfaces onto explicit temporary plugin roots, then
remove the remaining fixture-backed discovery dependence from the sandbox ensure
path before moving on to the hardened sandbox-process tranche.
