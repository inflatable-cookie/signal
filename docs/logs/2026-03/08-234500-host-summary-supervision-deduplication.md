# 08 234500 Host Summary Supervision Deduplication

Status: completed
Owner: core-product
Updated: 2026-03-08

## Summary

Removed the remaining runtime-owned readiness, diagnostics, supervision, and
event-stream mirrors from the local and server host summaries. Those concepts
now flow through `RuntimeSupervisorReport` only, leaving `host_summary` focused
on assembly-local execution and payload outcomes.

## Work Completed

1. Dropped duplicated runtime state from:
   - `crates/signal-host-local/src/host.rs`
   - `crates/signal-host-server/src/host.rs`
2. Updated host tests to assert degraded readiness, watchdog restart counts,
   safe-mode state, and event-stream size through `host.supervisor_report()`.
3. Removed duplicated readiness/supervision output from:
   - `crates/signal-host-local/src/main.rs`
   - `crates/signal-host-server/src/main.rs`
   - `crates/signal-supervisor-tools/src/main.rs`
4. Tightened the supervisor export contract so `host_summary` no longer mirrors
   runtime-owned readiness, diagnostics, or supervision state.

## Validation

- `cargo check -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `git diff --check`

Rust test/tool execution was attempted but not trusted as a clean signal in this
environment. Freshly launched `signal_host_local` and `signal-supervisor-tools`
processes intermittently stalled at `_dyld_start` before reaching project code,
so this batch closes with compile-only validation plus the prior green runtime
validation from the immediately preceding host-summary cleanup batches.

## Next Task

Decide whether the remaining automation and payload counters should stay as
host-summary convenience views, or whether the host layer should collapse
further toward only irreducibly assembly-local execution details.
