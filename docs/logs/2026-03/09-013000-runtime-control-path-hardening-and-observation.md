# 2026-03-09 01:30:00 UTC: Runtime Control-Path Hardening And Observation

Status: completed
Owner: core-product

## Summary

Hardened the core Signal runtime control path by requiring a real handshake
before configure/start, requiring configuration before start, and exposing the
resulting lifecycle state through a shared runtime control snapshot.

## Changes

- added `RuntimeControlSnapshot` to `crates/signal-runtime/src/interfaces.rs`
- threaded runtime lifecycle control state through `SignalRuntime`, including
  handshake/configure/start/stop/restart counters and last-request details
- enforced handshake-before-configure/start and configure-before-start in
  `crates/signal-runtime/src/runtime.rs`
- exposed the control snapshot through `RuntimeObservationReport` and
  `RuntimeSupervisorReport` renderers
- updated the runtime example and runtime tests to follow the hardened control
  sequence
- documented the new control snapshot surface in the Signal README and package
  map

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`

## Notes

- hosts already followed the handshake/configure/start ordering, so the main
  impact of this batch is on runtime tests, examples, and any future direct
  runtime consumers
- runtime execution is still not used as the validation gate in this
  environment because fresh Rust binaries can intermittently stall after launch

## Next Task

Use the new control snapshot to harden host/sandbox recovery paths, most likely
by threading restart intent and last stop reason into local/server host summary
surfaces or by enforcing stricter restart sequencing around plugin-sandbox
recovery.
