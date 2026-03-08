# 08 231500 Host Summary Automation Deduplication

Status: completed
Owner: core-product
Updated: 2026-03-08

## Summary

Removed the remaining runtime-owned automation continuity mirrors from
`signal-host-local` and `signal-host-server` summary structs.
Automation continuity now flows through `RuntimeSupervisorReport` only, while
host summaries retain automation event/value counters that remain useful as
assembly-local convenience fields.

## Work Completed

1. Dropped automation epoch/segment/lease-rollover fields from:
   - `crates/signal-host-local/src/host.rs`
   - `crates/signal-host-server/src/host.rs`
2. Updated host tests to assert automation continuity through
   `host.supervisor_report().observation.automation_snapshot` instead of host
   summary copies.
3. Removed duplicated automation continuity output from:
   - `crates/signal-host-local/src/main.rs`
   - `crates/signal-host-server/src/main.rs`
   - `crates/signal-supervisor-tools/src/main.rs`
4. Tightened the supervisor export contract so `host_summary` is explicitly not
   the default home for runtime-owned automation continuity.

## Validation

- `cargo test -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo run -p signal-supervisor-tools -- --format=json local soak`
- `cargo run -p signal-supervisor-tools -- --format=json server mixed`
- `git diff --check`

## Next Task

Decide whether block-sequence continuity should get the same cleanup as
automation continuity by removing more of that detail from host summaries or
explicitly keeping it as a convenience-only host projection.
