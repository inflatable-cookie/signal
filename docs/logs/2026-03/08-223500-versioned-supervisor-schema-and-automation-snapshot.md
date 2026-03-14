# 2026-03-08 22:35:00 - Versioned Supervisor Schema And Automation Snapshot

Status: complete
Owner: core-product

## Summary

Stabilized the supervisor export shape further by versioning the tool output and
adding shared automation snapshot support to the runtime report surface.

This batch adds:

- `RuntimeAutomationSnapshot` in `signal-runtime`,
- shared report attachment helpers so `RuntimeObservationReport` and
  `RuntimeSupervisorReport` can carry automation continuity alongside runtime
  timeline continuity,
- richer shared report rendering that includes automation continuity in text and
  JSON,
- summary-to-automation conversion helpers on local/server host summaries,
- versioned `signal.supervisor.export` JSON output in `signal-supervisor-tools`.

## Files

- `crates/signal-runtime/src/interfaces.rs`
- `crates/signal-runtime/src/lib.rs`
- `crates/signal-runtime/src/runtime.rs`
- `crates/signal-host-local/src/host.rs`
- `crates/signal-host-server/src/host.rs`
- `crates/signal-supervisor-tools/src/main.rs`
- `README.md`
- `docs/architecture/package-map.md`
- `docs/roadmaps/g01/004-trust-edge-package-shell-expansion.md`

## Validation

- `cargo fmt --all`
- `git diff --check`
- `cargo test -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo run -p signal-supervisor-tools -- --format=json local soak`
- `cargo run -p signal-supervisor-tools -- --format=json server mixed`
- `effigy validate`

## Validation Notes

- The JSON export is now explicitly versioned with `schema` and
  `schema_version`, which gives downstream automation a stable contract to pin
  against.
- Automation continuity is now part of the shared report surface, even though
  the data is still attached by host assemblies rather than owned by runtime
  state.

## Notes

- The remaining architectural question is whether automation continuity should
  stay an attached host-derived supplement or move deeper into runtime-owned
  state the way block-sequence continuity already did.

## Next Task

Document and stabilize the supervisor export schema as a reusable contract, then
decide which remaining host-derived continuity fields should move into
runtime-owned state instead of being attached at report assembly time.
