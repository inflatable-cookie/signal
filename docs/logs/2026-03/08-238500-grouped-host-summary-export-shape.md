# 08 238500 Grouped Host Summary Export Shape

Status: completed
Owner: core-product
Updated: 2026-03-08

## Summary

Aligned the machine-readable and text export surfaces with the grouped
host-summary structure. `signal-supervisor-tools` now emits grouped
`execution`, `transport`, `faults`, and `payload` sections in `host_summary`
instead of flattening those fields back out.

## Work Completed

1. Updated `crates/signal-supervisor-tools/src/main.rs` so:
   - text output reports grouped execution/transport/fault/payload sections
   - JSON output nests grouped host-local blocks instead of flattening them
2. Froze the grouped `host_summary` export preference in
   `docs/contracts/002-supervisor-export-schema-and-report-boundary.md`.

## Validation

- `cargo check -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `git diff --check`

Runtime execution was not used as the validation gate for this batch because
fresh Rust binaries in this environment can intermittently stall at `_dyld_start`
before reaching project code.

## Next Task

Decide whether the compact host-local payload summary should remain in the
grouped `host_summary`, or whether the host export should narrow further around
only `execution`, `transport`, and `faults`.
