---
title: Transport Fault Sequence Canonicalization
status: completed
owner: nucleus
updated: 2026-03-09
tags: [signal, runtime, supervisor, transport-faults]
---

# Summary

Added a canonical top-level `transport_fault_sequence` to the shared runtime
observation/report surface so supervision and soak tooling can inspect one
ordered transport-fault view with explicit source labels, while keeping the
existing broker-specific and sandbox-specific sequences as subordinate detail.

# Changes

- Wired `TransportFaultRecord`, `TransportFaultSource`, and
  `TransportFaultStage` through `signal-runtime` diagnostics, compact output,
  multiline output, and JSON export.
- Added `transport_fault_event_count()` and `last_transport_fault_event()` to
  the shared observation/supervisor surface.
- Derived the canonical ordered transport-fault sequence directly from the
  recorded runtime event stream so broker-visible and sandbox-operation
  failures preserve their real event order.
- Re-exported the shared transport-fault types from `signal-runtime`.
- Updated runtime, host, and supervisor-tool fixture coverage to pin the new
  aggregate `transport_fault_events`, `last_transport_fault`, and
  `transport_fault_sequence` shape.
- Updated the README, package map, supervisor export contract, and roadmap so
  `transport_fault_sequence` is explicitly the canonical top-level transport
  fault view, with `broker_failure_sequence` and
  `sandbox_operation_failure_sequence` retained as subordinate detail paths.

# Validation

- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-supervisor-tools --no-run`
- `cargo fmt --all`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-host-server --no-run`
- `git diff --check`

# Next Task

Decide whether the canonical `transport_fault_sequence` should gain richer
operation metadata from brokered block execution and lease teardown, or
whether that extra detail should stay only in the broker-specific and
sandbox-specific subordinate sequences.
