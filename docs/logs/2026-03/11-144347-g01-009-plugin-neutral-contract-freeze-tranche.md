# 2026-03-11 14:43:47 GMT - g01.009 plugin-neutral contract freeze tranche

## Summary

Opened `g01.009` with a broad `009.1` contract batch in `signal-plugin` and
updated the first CLAP consumer to use that surface.

The main goal of this tranche was to stop treating `signal-plugin` as only a
shared-memory/block transport crate and instead make it the canonical owner of
plugin-neutral descriptor, state, lifecycle, readiness, and fault semantics.

## What changed

- expanded `crates/signal-plugin/src/lib.rs` with plugin-neutral contract types:
  - descriptor enrichment via version, features, audio buses, parameters, state
    contract, processing contract, and lifecycle contract
  - explicit audio bus and parameter descriptor types
  - minimal lifecycle and readiness taxonomy for plugin instances
  - typed plugin fault kinds and severities aligned with runtime-style
    readiness and failure semantics
  - plugin process configuration and instance snapshot types for future runtime
    and sandbox projection
- added conversion from existing `PluginSandboxError` values into typed plugin
  faults and readiness outcomes, so the sandbox/control seam now has a shared
  vocabulary instead of ad hoc error interpretation
- updated `crates/signal-plugin-clap/src/lib.rs` so the CLAP fixture protocol
  publishes the richer neutral descriptor instead of bypassing the new surface
- added tests in both `signal-plugin` and `signal-plugin-clap` that pin the new
  descriptor/lifecycle/fault contract as a real consumer-facing API

## Validation

- `cargo test -p signal-plugin`
- `cargo test -p signal-plugin-clap`
- `cargo check -p signal-plugin-sandbox`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-host-server --no-run`

This tranche intentionally stopped at the Rust/plugin contract seam. It did not
yet run repo-wide Effigy validation because the behavioral change is still
scoped to the plugin-neutral contract layer and its immediate consumers.

## Ownership notes

- `signal-plugin` is now the canonical owner of plugin-neutral metadata and
  lifecycle/fault/readiness semantics
- `signal-plugin-clap` remains the owner of CLAP-specific extension and message
  details, but now projects them through the shared neutral contract
- runtime and host crates remain consumers of plugin-neutral state rather than
  owners of plugin-format interpretation

## Follow-on

The next batch should move into `009.2`: real CLAP/sandbox lifecycle control
and typed processing-state transport on top of this frozen neutral contract.
