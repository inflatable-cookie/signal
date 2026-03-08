# 2026-03-08 23:59:00 UTC: Default Host Export Payload Narrowing

Status: completed
Owner: core-product

## Summary

Narrowed the default host-facing export surface so payload detail stays inside
the host implementations for testing/debugging, but no longer appears in the
default local/server binary output or the schema-versioned
`signal-supervisor-tools` export.

## Changes

- removed payload fields from the default stdout summaries in
  `crates/signal-host-local/src/main.rs` and
  `crates/signal-host-server/src/main.rs`
- removed payload sections from the text renderer in
  `crates/signal-supervisor-tools/src/main.rs`
- removed the `payload` object from the default JSON `host_summary` renderer in
  `crates/signal-supervisor-tools/src/main.rs`
- updated the supervisor export contract to freeze payload as internal-only for
  schema version 1 default exports
- advanced the package map and active roadmap notes to the next boundary
  decision around optional verbose/debug export modes

## Validation

- `cargo fmt --all`
- `cargo check -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `git diff --check`

## Notes

- `last_payload` remains available inside host summary types for internal tests
  and debugging; this batch only narrows the default exported/view layer.
- runtime execution was not used as the validation gate for this batch because
  fresh Rust binaries in this environment can still intermittently stall at
  `_dyld_start` before reaching project code.

## Next Task

Decide whether host-local payload detail should get an explicit opt-in
verbose/debug export mode, or remain internal-only while the default contract
stays focused on execution, transport, faults, and shared runtime supervision.
