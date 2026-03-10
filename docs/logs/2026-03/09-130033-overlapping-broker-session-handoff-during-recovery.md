---
title: Overlapping Broker Session Handoff During Recovery
date: 2026-03-09
status: closed
---

## Summary

Implemented the first real overlapping broker-session handoff path for Signal
recovery. Local and server hosts now prepare and attach the replacement broker
transport before tearing down the old transport, so runtime-owned concurrency
policy is exercised by actual recovery control flow instead of only by guarded
attach/detach sequencing.

## Changes

- rewired local and server recovery flows so recovery returns the replacement
  lifecycle run directly instead of tearing down and then re-running lifecycle
  in a separate phase
- attached replacement broker sessions under `RecoveryOverlap` intent before
  old transport teardown, letting runtime concurrency peak at two attached
  sessions during handoff
- added a runtime-owned transport concurrency snapshot to the shared
  observation/supervisor surface and export path
- pinned runtime admission policy with a dedicated `signal-runtime` test and
  updated host assertions so timeout recovery now proves real overlap via
  peak attached sessions and recovery-overlap counts
- updated Signal architecture/contract docs to mark the overlap handoff as
  implemented and to move the next step toward rollback hardening

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime --no-run`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-host-server --no-run`
- `git diff --check`

## Next Task

Harden the overlapping broker-session handoff path, especially failure
rollback when replacement lifecycle preparation succeeds but old transport
teardown or replacement startup fails.
