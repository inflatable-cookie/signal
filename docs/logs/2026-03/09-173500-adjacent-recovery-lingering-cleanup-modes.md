---
title: Adjacent Recovery Lingering Cleanup Modes
date: 2026-03-09
status: closed
---

## Summary

Hardened lingering cleanup around adjacent recovery episodes by making cleanup
mode explicit and proving that prior lingering state is either swept before the
next recovery or blocks it cleanly.

## Changes

- local and server hosts now distinguish strict pre-attach lingering cleanup
  from best-effort post-start reconciliation
- lingering cleanup order is now deterministic across intent/state/identity so
  mixed stale sessions are processed consistently
- added adjacent-episode local/server regressions that prove:
  - a prior lingering origin can be swept before the next recovery from a newer
    lingering replacement
  - invalid metadata on that prior lingering origin aborts the next recovery
    cleanly instead of creating a third transport session

## Validation

- `cargo fmt --all`
- `cargo check -p signal-host-local -p signal-host-server`
- `cargo test -p signal-host-local adjacent_overlap -- --nocapture`
- `cargo test -p signal-host-server adjacent_overlap -- --nocapture`
- `cargo test -p signal-host-local late_lingering -- --nocapture`
- `cargo test -p signal-host-server late_lingering -- --nocapture`

## Next

Push lingering-session provenance and cleanup scheduling further into runtime
state, especially if the engine needs to distinguish older origin lingerers
from newer replacement lingerers without relying on host-local cleanup policy
alone.
