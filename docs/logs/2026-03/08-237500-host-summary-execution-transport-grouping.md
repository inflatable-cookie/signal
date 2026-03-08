# 08 237500 Host Summary Execution Transport Grouping

Status: completed
Owner: core-product
Updated: 2026-03-08

## Summary

Grouped the remaining flat host-local execution fields into compact execution,
transport, and fault blocks for the local and server host summaries. This keeps
`host_summary` readable as an assembly-local report instead of another flat
compatibility surface.

## Work Completed

1. Added grouped host-local blocks in:
   - `crates/signal-host-local/src/host.rs`
   - `crates/signal-host-server/src/host.rs`
   using execution, transport, fault, and payload summaries.
2. Updated host tests to assert through the grouped blocks instead of the old
   flat execution fields.
3. Updated:
   - `crates/signal-host-local/src/main.rs`
   - `crates/signal-host-server/src/main.rs`
   - `crates/signal-supervisor-tools/src/main.rs`
   so their text and JSON surfaces follow the grouped host-summary shape.
4. Tightened the supervisor export contract to prefer grouped host-local blocks
   over flat host-summary fields.

## Validation

- `cargo check -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `git diff --check`

Runtime execution was not used as the validation gate for this batch because
fresh Rust binaries in this environment can intermittently stall at `_dyld_start`
before reaching project code.

## Next Task

Decide whether the compact host-local payload summary should remain alongside
the grouped execution/transport/fault blocks, or whether the host layer should
collapse further toward only control-path, transport, and fault execution
details.
