# 2026-03-08 23:05:00 - Runtime Owned Automation Continuity

Status: complete
Owner: core-product

## Summary

Moved automation continuity ownership into `signal-runtime` instead of keeping
it as a host-attached supplement.

This batch adds:

- runtime-owned automation tracking and reset behavior in `signal-runtime`,
- `RuntimeObservationApi::get_automation_snapshot()` so shared reports capture
  automation continuity directly from runtime state,
- host summary surfaces that now read automation continuity from runtime-owned
  snapshots instead of maintaining their own recovery-side copies,
- updated soak expectations reflecting the more accurate earliest contributing
  automation epoch,
- contract and architecture docs updated to treat automation continuity as
  runtime-owned rather than transitional.

## Files

- `crates/signal-runtime/src/interfaces.rs`
- `crates/signal-runtime/src/runtime.rs`
- `crates/signal-host-local/src/host.rs`
- `crates/signal-host-server/src/host.rs`
- `crates/signal-supervisor-tools/src/main.rs`
- `docs/contracts/002-supervisor-export-schema-and-report-boundary.md`
- `docs/architecture/system-architecture.md`
- `docs/architecture/package-map.md`
- `docs/roadmaps/g01/004-trust-edge-package-shell-expansion.md`

## Validation

- `cargo fmt --all`
- `git diff --check`
- `cargo test -p signal-runtime -p signal-host-local -p signal-host-server -p signal-supervisor-tools`
- `cargo run -p signal-supervisor-tools -- --format=json local soak`
- `cargo run -p signal-supervisor-tools -- --format=json server mixed`

## Validation Notes

- The soak runs now report `automation.first_epoch = 2` rather than `3`, which
  reflects the true earliest contributing epoch once continuity is sourced from
  runtime-owned state instead of host-local merge history.
- This batch intentionally did not rerun the heavier Effigy loop after the
  earlier green `validate` pass because the change stayed inside the Rust
  workspace and focused validation is already green.

## Notes

- Host summaries still mirror automation fields for convenience. That is now a
  presentation choice, not the source of truth.

## Next Task

Trim or justify the remaining duplicate automation fields in host-specific
summary surfaces now that automation continuity is runtime-owned, then tighten
the export/schema docs around which fields are canonical versus convenience-only.
