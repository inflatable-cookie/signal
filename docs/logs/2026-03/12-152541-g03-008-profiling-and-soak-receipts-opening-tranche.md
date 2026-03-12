# g03.008 - Profiling And Soak Receipts Opening Tranche

Date: 2026-03-12
Status: completed tranche
Roadmap: `docs/roadmaps/g03/008-engine-profiling-soak-harnesses-and-runtime-fault-hardening.md`

## Summary

Opened `g03.008` and completed Batch 8.1. Signal now has runtime-owned
profiling and soak receipts that derive from the existing supervisor and
host-observation surfaces, plus one concrete supervisor-tool export path and
host soak proof that consume those receipts directly.

## Shipped

- added typed `RuntimeProfilingReceipt` and `RuntimeSoakReceipt` surfaces in
  `signal-runtime`, derived from existing supervisor and host-observation
  reports rather than new host-local benchmark state
- exposed receipt builders on `RuntimeSupervisorReport` and
  `RuntimeHostSupervisorReport` so runtime-only and host-backed harnesses can
  consume the same profiling/soak contract
- added focused runtime coverage proving the new receipts capture block,
  latency, xrun, and recovery counters from the runtime-owned report surface
- extended `signal-host-local` soak coverage so mixed watchdog runs assert the
  host-backed profiling and soak receipts directly
- extended `signal-supervisor-tools` JSON/text export so routed soak scenarios
  carry typed profiling and soak receipts alongside the existing supervisor
  report instead of making callers mine counters out of free-form output
- aligned `signal-host-server` with the current runtime supervisor trait so
  the supervisor-tool crate can compile against the same runtime-owned API

## Deferred

- Batch 8.1 stops at receipt/export shape and one concrete routed soak
  consumer; it does not yet harden degraded/fault behavior across the offline
  render or plugin-chain-heavy paths
- the new receipts summarize current counters, but they do not yet impose
  threshold policy or fail gates for future hardening runs

## Validation

- `cargo test -p signal-runtime`
- `cargo test -p signal-host-local`
- `cargo test -p signal-supervisor-tools`
- `cargo fmt --all`
- `git diff --check`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`

## Next Task

Continue `g03.008` with Batch 8.2 by threading the new profiling/soak receipts
through routing, plugin-chain, and offline-render fault cases, then pin the
degraded/recovery acceptance boundary before closing `g03`.
