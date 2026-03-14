---
title: Runtime-Owned Forecast Mode Switching
status: completed
updated: 2026-03-10
owner: codex
---

## Summary

- added `RuntimePreworkForecastMode` as a first-class runtime state with `Disabled`, `RuntimeRoleDefault`, `ExplicitProfile`, and `RawPolicyOverride`
- made `signal-runtime` derive the effective forecast mode during `configure`, expose it through the shared engine snapshot/export surface, and support explicit mode switching at runtime
- kept forecast profile/policy metadata separate from the effective mode so disabled runtime roles still report their default forecast profile while forecast priming remains off
- taught the runtime forecast path to apply current-block forecast state while skipping future-window priming when forecast mode is disabled
- tightened local and server host proofs so local asserts role-default forecast mode and server asserts disabled forecast mode

## Validation

- `cargo fmt --all`
- `cargo check -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo test -p signal-runtime forecast -- --nocapture`
- `cargo test -p signal-host-local local_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-host-server server_host_rolls_leases_forward_after_timeout -- --nocapture`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`
- `effigy validate`
- `effigy health`

## Notes

- no stale Effigy lock recurrence showed up in this batch
- current workspace validation should now fail only on external workspace/toolchain issues, not on this Signal runtime slice
