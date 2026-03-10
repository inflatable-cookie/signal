---
title: Overlap Admission Contention And Cleanup Hardening
date: 2026-03-09
status: closed
---

## Summary

Hardened Signal's runtime-owned overlap admission control so contended recovery
attaches fail cleanly instead of leaving half-admitted state behind. The
runtime now rejects a second concurrent `RecoveryOverlap` attach while an
existing overlap session is still attached, and both hosts now prove that
contended overlap recovery rolls all transport state back to zero attached
sessions.

## Changes

- tightened `signal-runtime` transport admission so `RecoveryOverlap` is capped
  to one concurrent overlap session at a time, with explicit rejection reasons
  rather than only total attached-session counting
- extended the runtime admission test to prove repeated overlap attempts stay
  rejected until the previous overlap session ends, then become admissible
  again
- hardened local and server `run_lifecycle` setup so admission-rejected overlap
  prepares tear down their broker region and lifecycle state immediately
- added host-level overlap-contention recovery paths that simulate a competing
  replacement attach before old-session detach completes
- added local and server regression tests asserting overlap contention aborts
  recovery cleanly and leaves zero attached transport sessions after rollback
- updated Signal docs to freeze the single-overlap admission rule and rejected
  prepare cleanup semantics

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server`
- `cargo test -p signal-runtime admission_policy`
- `cargo test -p signal-host-local competing_recovery_attach_is_rejected`
- `cargo test -p signal-host-server competing_recovery_attach_is_rejected`

## Next Task

Harden multi-step recovery orchestration around detach latency and repeated
failed recovery episodes, especially where old-session teardown faults and
overlap admission rejection interleave across more than one attempt.
