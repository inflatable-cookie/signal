---
title: Runtime-Owned Forecast Profile Selection
status: completed
updated: 2026-03-10
owner: codex
---

## Summary

- added runtime-known forecast profile selection in `signal-runtime`
- persisted selected profile metadata alongside the expanded stored forecast policy
- switched local and server hosts to select runtime-known profiles instead of constructing raw policy structs inline
- exposed the selected forecast profile and optional target-window override through the shared engine snapshot/export surface

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- focused runtime profile-selection tests
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-local --no-run`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`
- `effigy validate --repo .`
- `effigy health --repo .`

## Notes

- the raw forecast-policy setter remains available as a lower-level escape hatch for focused runtime tests and non-profile use cases
- no stale Effigy locks appeared during this batch
