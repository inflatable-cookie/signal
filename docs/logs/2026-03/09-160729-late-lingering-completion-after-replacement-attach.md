---
title: Late Lingering Completion After Replacement Attach
date: 2026-03-09
status: closed
---

## Summary

Added an explicit post-start reconciliation path for lingering transport
sessions after a replacement session is already active.

## Changes

- local and server hosts now route post-start lingering cleanup through a
  dedicated `reconcile_late_lingering_sessions_after_start(...)` helper instead
  of leaving that behavior as an inline orphan-sweep call
- added local and server regressions that prove:
  - a late lingering origin session can be detached and removed after the
    replacement session is already running
  - a failed late lingering cleanup leaves the active replacement session
    running while the lingering session remains visible in runtime-owned
    transport admission state
- updated Signal docs to make the post-start reconciliation rule explicit in
  the runtime/host control model

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-host-local late_lingering -- --nocapture`
- `cargo test -p signal-host-server late_lingering -- --nocapture`
- `cargo test -p signal-host-local orphan_lingering -- --nocapture`
- `cargo test -p signal-host-server orphan_lingering -- --nocapture`
- `git diff --check`

## Next

Harden mixed lingering-session churn further, especially when a late origin
detach completion and a fresh overlap attach race with each other or when more
than one lingering candidate for the same sandbox can resolve across adjacent
recovery episodes.
