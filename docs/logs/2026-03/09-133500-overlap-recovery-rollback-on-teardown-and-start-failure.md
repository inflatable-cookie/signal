---
title: Overlap Recovery Rollback On Teardown And Start Failure
date: 2026-03-09
status: closed
---

## Summary

Hardened Signal's overlapping broker-session recovery path so failed handoffs
do not leak the replacement session. Local and server hosts now roll the
replacement lifecycle back if old transport teardown fails after overlap
attach or if replacement startup fails before runtime returns to `Ready`.

## Changes

- added injected recovery failure paths in both host assemblies for old
  transport teardown failure and replacement start failure
- extended overlap recovery so replacement lifecycle and broker transport are
  torn back down when teardown, restart, or startup fails after overlap
  admission
- reset runtime-visible active sandbox counts to zero on failed overlap
  recovery instead of leaving the replacement session attached
- added focused local and server rollback regression tests asserting that
  current attached sessions return to zero while peak attached sessions still
  prove real overlap during the failed handoff
- updated Signal architecture, contract, and roadmap docs to mark overlap
  rollback as implemented and move the next step to repeated/concurrent
  overlap admission hardening

## Validation

- `cargo fmt --all`
- `cargo check -p signal-host-local -p signal-host-server`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-host-server --no-run`
- `cargo test -p signal-host-local recovery_teardown_fails`
- `cargo test -p signal-host-server recovery_start_fails`
- `git diff --check`

## Next Task

Stress the runtime-owned transport admission policy under repeated overlap
recovery attempts, especially concurrent replacement attaches competing with
detach completion and rollback after partial handoff progress.
