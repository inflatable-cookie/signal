---
title: retryable lingering session cleanup
status: completed
owner: codex
updated: 2026-03-09
tags: [signal, runtime, transport, recovery]
---

## Summary

Extended the lingering-session cleanup path so cleanup itself can fail once
more and still succeed on a later degraded recovery attempt.

## What Changed

- Added a second cleanup-oriented recovery scenario in both local and server
  hosts:
  - first degraded recovery fails with deferred old-session teardown
  - second degraded recovery attempts lingering cleanup and fails again
  - third degraded recovery retries cleanup, succeeds, and boots a fresh
    lifecycle
- Extended recovery failure injection with a cleanup-specific failure stage so
  the retry path is exercised by host control logic instead of only by test
  shims around final state.
- Kept runtime-owned lingering-session admission state intact across the extra
  cleanup failure, so the faulted session remains visible and capacity is only
  freed once explicit cleanup really succeeds.
- Preserved the earlier successful cleanup and interleaved-failure paths.

## Validation

- `cargo check -p signal-host-local -p signal-host-server`
- `cargo test -p signal-host-local local_host_recovers_after_lingering_cleanup_fails_once_more`
- `cargo test -p signal-host-server server_host_recovers_after_lingering_cleanup_fails_once_more`
- `cargo test -p signal-host-local local_host_recovers_after_lingering_deferred_teardown_cleanup`
- `cargo test -p signal-host-server server_host_recovers_after_lingering_deferred_teardown_cleanup`
- `cargo test -p signal-host-local local_host_handles_interleaved_recovery_failures_across_retries`
- `cargo test -p signal-host-server server_host_handles_interleaved_recovery_failures_across_retries`

## Notes

- The lingering session remains runtime-owned state across both the initial
  deferred teardown failure and the later cleanup retry failure.
- Recovery only frees steady-state capacity once the explicit lingering cleanup
  path records `Detached` and ends the session in runtime transport
  concurrency.

## Next Task

Harden mixed origin/replacement teardown churn further, especially around more
than one concurrent lingering session candidate or cleanup races between a
fresh replacement attach and a late origin teardown completion.
