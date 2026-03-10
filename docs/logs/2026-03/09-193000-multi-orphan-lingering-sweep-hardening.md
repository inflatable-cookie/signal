---
title: multi orphan lingering sweep hardening
status: completed
owner: codex
updated: 2026-03-09
tags: [signal, runtime, transport, recovery]
---

## Summary

Hardened lingering-session cleanup so recovery can sweep more than one orphan
lingering transport for the same sandbox and records metadata-gap failures as
real transport-side faults instead of failing silently.

## What Changed

- Extended orphan lingering cleanup in both local and server hosts to support
  two cleanup modes:
  - fail-fast for pre-attach recovery cleanup
  - best-effort for non-fatal follow-on cleanup paths
- Added explicit broker-failure and `DetachFault` recording when an orphan
  lingering session cannot be reconstructed from runtime-owned metadata
  (`backing_path`, `total_bytes`).
- Hardened the strict pre-attach recovery sweep so it now clears multiple
  orphan lingering sessions for the same sandbox instead of assuming a single
  stale replacement-side transport.
- Kept the post-start best-effort cleanup hook in place for future late-detach
  race handling while leaving the validated behavior focused on the reachable
  strict recovery paths.
- Added focused local/server tests for:
  - multiple orphan lingering sessions cleaned in one sweep
  - orphan cleanup abort when runtime-owned metadata is missing
  - existing lingering cleanup and deferred-teardown recovery regressions

## Validation

- `cargo fmt --all`
- `cargo check -p signal-host-local -p signal-host-server -p signal-runtime -p signal-supervisor-tools`
- `cargo test -p signal-host-local orphan_lingering -- --nocapture`
- `cargo test -p signal-host-server orphan_lingering -- --nocapture`
- `cargo test -p signal-host-local lingering -- --nocapture`
- `cargo test -p signal-host-server lingering -- --nocapture`

## Notes

- The currently validated multi-orphan path is the strict recovery cleanup
  phase before a fresh overlap attach.
- A best-effort post-start sweep hook now exists in host control flow, but the
  current harness does not yet synthesize the exact late-detach race needed to
  validate that path end-to-end without bypassing the runtime admission limits
  under test.

## Next Task

Harden lingering-session race handling around late detach completion,
especially when a previously faulted origin teardown resolves after a fresh
replacement attach and the host needs to fold that completion back into
runtime admission without disturbing the active replacement session.
