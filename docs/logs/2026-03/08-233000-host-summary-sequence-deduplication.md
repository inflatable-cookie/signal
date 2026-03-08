# 08 233000 Host Summary Sequence Deduplication

Status: completed
Owner: core-product
Updated: 2026-03-08

## Summary

Removed the remaining runtime-owned block-sequence continuity mirrors from the
local and server host summaries. Sequence continuity now flows through
`RuntimeSupervisorReport` only, matching the same canonical boundary already
used for automation continuity.

## Work Completed

1. Dropped sequence continuity fields from:
   - `crates/signal-host-local/src/host.rs`
   - `crates/signal-host-server/src/host.rs`
2. Updated host tests to assert sequence continuity through
   `host.supervisor_report().observation.timeline_snapshot.block_sequence_continuity`.
3. Removed duplicated sequence continuity output from:
   - `crates/signal-host-local/src/main.rs`
   - `crates/signal-host-server/src/main.rs`
   - `crates/signal-supervisor-tools/src/main.rs`
4. Tightened the supervisor export contract so `host_summary` no longer mirrors
   runtime-owned sequence continuity either.

## Validation

- `cargo test -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo run -p signal-supervisor-tools -- --format=json local soak`
- `cargo run -p signal-supervisor-tools -- --format=json server mixed`
- `git diff --check`

## Next Task

Decide whether the remaining host summary counters should stay as convenience
views, or whether more of that assembly-local output should collapse into
`RuntimeSupervisorReport` so the host layer gets even thinner.
