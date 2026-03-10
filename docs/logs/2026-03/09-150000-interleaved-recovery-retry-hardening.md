---
title: Interleaved Recovery Retry Hardening
date: 2026-03-09
status: closed
---

## Summary

Hardened Signal's host recovery orchestration so degraded recovery can retry
while runtime is already stopped, and proved that interleaved teardown-fault
and overlap-admission failures unwind cleanly across more than one attempt.

## Changes

- added host-side recovery orchestration that treats `runtime.stop()` as a
  no-op when runtime is already stopped from a prior degraded attempt
- added a staged interleaved failure path in local and server hosts: first a
  deferred old-session teardown failure, then a competing overlap attach
  rejection on the next recovery attempt
- kept the original session alive through the deferred teardown fault so the
  next recovery attempt actually runs against a non-clean-slate transport state
- verified that the second attempt still rolls all broker-session state back to
  zero attached sessions after the overlap admission rejection
- updated Signal docs to describe staged retry semantics as implemented and
  pushed the next step toward real lingering-session state rather than purely
  host-local deferred failure injection

## Validation

- `cargo fmt --all`
- `cargo check -p signal-host-local -p signal-host-server`
- `cargo test -p signal-host-local interleaved_recovery_failures`
- `cargo test -p signal-host-server interleaved_recovery_failures`

## Next Task

Model real lingering-session state in runtime admission and recovery handling,
so detach latency and repeated failed recovery episodes are represented as
first-class engine state rather than host-local deferred failure injection.
