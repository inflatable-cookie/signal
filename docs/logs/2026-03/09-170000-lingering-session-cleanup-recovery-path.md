---
title: lingering session cleanup recovery path
status: completed
owner: codex
updated: 2026-03-09
tags: [signal, runtime, transport, recovery]
---

## Summary

Added a real cleanup-and-recover path for runtime-owned lingering transport
sessions.

## What Changed

- Added a new local/server recovery scenario where an initial degraded recovery
  attempt fails with deferred old-session teardown, leaving a runtime-owned
  lingering `DetachFaulted` session.
- On the next recovery attempt, hosts now inspect
  `transport_concurrency_snapshot` and, when the current origin session is
  lingering, explicitly:
  - retry broker-region destroy
  - retry lifecycle transport teardown
  - record `Detached`
  - end the lingering session in runtime transport concurrency
  - restart the sandbox
  - boot a fresh lifecycle on the next processing epoch
- Preserved the earlier interleaved failure path where repeated degraded
  recovery can still fail and unwind cleanly; cleanup is now an additional
  success path, not a replacement for the previous failure hardening.
- Documented the control meaning: lingering state is not only observable in
  runtime admission, it is now actionable in later recovery attempts.

## Validation

- `cargo check -p signal-host-local -p signal-host-server`
- `cargo test -p signal-host-local local_host_recovers_after_lingering_deferred_teardown_cleanup`
- `cargo test -p signal-host-server server_host_recovers_after_lingering_deferred_teardown_cleanup`
- `cargo test -p signal-host-local local_host_handles_interleaved_recovery_failures_across_retries`
- `cargo test -p signal-host-server server_host_handles_interleaved_recovery_failures_across_retries`

## Notes

- The successful cleanup path currently re-enters from runtime-owned lingering
  control state and restarts from a fresh transport attach.
- Peak lingering count is `2` in the successful cleanup scenario because the
  first failed recovery episode briefly includes both the faulted origin and
  the rollback path for the replacement lifecycle before cleanup succeeds on
  the next attempt.

## Next Task

Harden lingering-session cleanup around repeated cleanup failures and mixed
origin/replacement teardown churn, especially when a previously faulted
session fails cleanup once more before finally freeing steady-state capacity.
