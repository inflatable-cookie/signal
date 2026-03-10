---
title: Runtime-Owned Transport Session Admission Policy
date: 2026-03-09
status: closed
---

## Summary

Moved the transport concurrency boundary from passive observation into
runtime-owned control by adding a transport-session admission policy for
steady-state versus recovery-overlap attach intent, then binding local/server
host lifecycle attach and detach paths to that runtime state.

## Changes

- added a runtime-owned transport concurrency snapshot with policy limits,
  current/peak attached session counts, recovery-overlap counts, active
  session identities, and last rejection details
- added runtime attach/detach admission methods so hosts ask `signal-runtime`
  before accepting a broker transport session
- wired local and server host lifecycle attach/detach paths into that runtime
  admission state
- pinned the policy in `signal-runtime` tests and added host/supervisor test
  assertions so the control path is exercised through real lifecycle runs
- updated Signal docs to freeze `transport_session_summary` as schema-version-1
  stable and to treat transport admission policy as the next control-layer
  boundary rather than another export-surface expansion

## Validation

- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-supervisor-tools --no-run`

## Next Task

Drive a real overlapping broker-session handoff path during restart/recovery so
the new runtime admission policy is exercised under actual overlap rather than
only guarded attach/detach sequencing.
