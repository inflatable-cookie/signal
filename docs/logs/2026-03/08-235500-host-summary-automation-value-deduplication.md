# 08 235500 Host Summary Automation Value Deduplication

Status: completed
Owner: core-product
Updated: 2026-03-08

## Summary

Removed the remaining runtime-owned automation value and counter mirrors from
the local and server host summaries. Automation now lives entirely under
`RuntimeSupervisorReport.observation.automation_snapshot`, leaving `host_summary`
focused on host-local execution and payload outcomes.

## Work Completed

1. Dropped automation value/counter fields from:
   - `crates/signal-host-local/src/host.rs`
   - `crates/signal-host-server/src/host.rs`
2. Updated host tests to assert automation counts and value snapshots through
   `host.supervisor_report().observation.automation_snapshot`.
3. Removed duplicated automation output from:
   - `crates/signal-host-local/src/main.rs`
   - `crates/signal-host-server/src/main.rs`
   - `crates/signal-supervisor-tools/src/main.rs`
4. Tightened the supervisor export contract so `host_summary` no longer mirrors
   runtime-owned automation counters or value snapshots.

## Validation

- `cargo check -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `git diff --check`

Runtime execution was not used as the validation gate for this batch because
fresh Rust binaries in this environment can intermittently stall at `_dyld_start`
before reaching project code.

## Next Task

Decide whether the remaining payload event counters should stay as host-summary
convenience views, or whether the host layer should collapse further toward only
irreducibly assembly-local execution details.
