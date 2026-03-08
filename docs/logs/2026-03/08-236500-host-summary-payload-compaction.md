# 08 236500 Host Summary Payload Compaction

Status: completed
Owner: core-product
Updated: 2026-03-08

## Summary

Collapsed the remaining wide payload counter spread in the local and server host
summaries into one compact host-local payload summary block. This keeps
`host_summary` focused on assembly-local execution results without scattering
payload counters across the summary surface.

## Work Completed

1. Replaced the flat payload counter/value fields in:
   - `crates/signal-host-local/src/host.rs`
   - `crates/signal-host-server/src/host.rs`
   with compact `LocalPayloadSummary` / `ServerPayloadSummary` blocks.
2. Updated host tests to assert payload results through `summary.last_payload`
   while keeping runtime-owned automation and supervision assertions on
   `host.supervisor_report()`.
3. Updated:
   - `crates/signal-host-local/src/main.rs`
   - `crates/signal-host-server/src/main.rs`
   - `crates/signal-supervisor-tools/src/main.rs`
   so the CLI surfaces report one compact payload block instead of a broad list
   of payload fields.
4. Tightened the export contract so host-local payload outcomes are treated as a
   compact execution-result surface rather than a broad compatibility dump.

## Validation

- `cargo check -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `git diff --check`

Runtime execution was not used as the validation gate for this batch because
fresh Rust binaries in this environment can intermittently stall at `_dyld_start`
before reaching project code.

## Next Task

Decide whether the compact host-local payload summary should remain in
`host_summary`, or whether the host layer should collapse further toward only
control-path and transport execution details.
