---
title: Runtime Role-Derived Forecast Profile Defaults
status: completed
updated: 2026-03-10
owner: codex
---

## Summary

- made `signal-runtime` derive its default prework forecast profile from `RuntimeConfig.profile` during configure
- tracked whether the active forecast profile comes from runtime-role defaulting, explicit profile selection, or raw policy override
- removed host-side local/server forecast profile selection from the local and server boot flows
- exposed the new forecast-profile source metadata through the shared engine snapshot/export surface

## Validation

- `cargo fmt --all`
- focused `cargo check` for `signal-runtime`, `signal-host-local`, `signal-host-server`, and `signal-supervisor-tools`
- focused runtime forecast-profile tests
- focused host recovery tests
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`
- `effigy validate`
- `effigy health`

## Notes

- workspace Effigy remains sensitive to overlapping invocations, but no stale lock recurrence appeared in this batch
- current workspace-level failures, if they recur, are outside this Signal slice rather than caused by the new runtime forecast behavior
