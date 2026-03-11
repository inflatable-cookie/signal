---
title: g01.007 runtime stabilization closure
status: completed
owner: nucleus
updated: 2026-03-11
tags: [signal, runtime, roadmap, g01, g01.007, stabilization, tests]
---

## Summary

Closed the `g01.007` runtime stabilization pass for the current transport-authority tranche.

The main result is that `cargo test -p signal-runtime` is green again after reconciling the forecast/prework test surface with the newer runtime model.

## What changed

- Added test-only helpers in `crates/signal-runtime/src/runtime.rs` to separate manual prework tests from role-default forecast behavior.
- Reworked the pressure-policy tests so they seed explicit pending targets and validate the service policy directly instead of depending on whatever backlog survives `start()`.
- Updated snapshot and observation expectations to match the current runtime semantics:
  - consumed prework snapshots instead of admitted-only snapshots in current-block execution
  - transport/session summary counts and active block reporting
  - transport fault phase totals in observation rendering
- Relaxed a few brittle queue-depth assertions so the forecast runner tests validate the intended behavior:
  - bounded budget leaves pending work
  - zero budget starves the lane
  - pressure policies yield, throttle, or clear according to service semantics

## Validation

- Passed: `cargo test -p signal-runtime -- --nocapture`
- Passed: `effigy health --repo .`
- Passed: `effigy validate --repo .`
- Passed: `effigy test --repo .`
- Passed: touched-file `git diff --check`

## Notes

- I initially ran `effigy validate` and `effigy test` in parallel and hit the known workspace/CMake collision. Rerunning `effigy validate` serially passed cleanly.
- This closes the broad stabilization work that started after the transport-truth batch. The remaining runtime work can now move back to roadmap-forward implementation instead of test-surface cleanup.
