---
title: orphan lingering session sweep before overlap recovery
status: completed
owner: codex
updated: 2026-03-09
tags: [signal, runtime, transport, recovery]
---

## Summary

Hardened degraded recovery so orphan lingering sessions are swept before a
fresh overlap attach, and fixed replacement rollback so teardown failures stay
visible as runtime-owned lingering state instead of being dropped early.

## What Changed

- Added runtime-host logic in both local and server hosts to sweep orphan
  lingering sessions for the same sandbox before opening a new
  `RecoveryOverlap` transport attach.
- Reused the same sweep path from the existing lingering-origin cleanup flow so
  origin cleanup and orphan cleanup now follow one host-level policy instead of
  separate ad hoc branches.
- Fixed replacement rollback teardown semantics so a rollback only ends the
  replacement transport session after a clean detach; failed rollback teardown
  now leaves that replacement session visible as lingering runtime-owned state.
- Added focused local/server tests for:
  - successful orphan lingering cleanup before overlap recovery
  - deterministic recovery abort when orphan cleanup lacks valid transport
    metadata
  - existing lingering cleanup and interleaved recovery regressions
- Updated the architecture and export contract docs to make the new orphan
  cleanup boundary explicit.

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-host-local lingering -- --nocapture`
- `cargo test -p signal-host-server lingering -- --nocapture`
- `cargo test -p signal-host-local interleaved_recovery_failures -- --nocapture`
- `cargo test -p signal-host-server interleaved_recovery_failures -- --nocapture`
- `cargo test -p signal-host-local competing_recovery_attach_is_rejected -- --nocapture`
- `cargo test -p signal-host-server competing_recovery_attach_is_rejected -- --nocapture`

## Notes

- Orphan cleanup now depends on runtime-owned transport metadata
  (`backing_path`, `total_bytes`) carried in `transport_concurrency_snapshot`.
- Recovery intentionally aborts when that metadata is missing, because skipping
  the lingering session would leave admission and broker state out of sync.

## Next Task

Harden multi-session lingering-session churn further, especially around more
than one orphan lingering candidate or cleanup races between a fresh
replacement attach and a late origin teardown completion.
