# 2026-03-08 22:15:00 - Supervisor JSON Export And Timeline Reporting

Status: complete
Owner: core-product

## Summary

Added machine-readable soak reporting and moved more timeline continuity detail
into the shared runtime supervisor report surface.

This batch adds:

- explicit timeline fields in `RuntimeObservationReport::render_compact()`,
- explicit timeline fields in `RuntimeSupervisorReport::render_multiline()`,
- `RuntimeSupervisorReport::render_json()` for stable machine-readable export,
- `signal-supervisor-tools --format=json` / `--json` support,
- machine-readable host summary export in the supervisor tool so automation can
  inspect local/server restart and lease-rollover scenarios without scraping
  the human text layout,
- tests that pin the new runtime JSON/timeline output and CLI argument parsing.

## Files

- `crates/signal-runtime/src/interfaces.rs`
- `crates/signal-runtime/src/runtime.rs`
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

- The shared supervisor report now exposes timeline cursor and sequence
  continuity directly, so machine-readable consumers no longer need to infer
  that data only from host-specific summary text.
- The JSON export still uses a hand-rolled stable schema rather than `serde`.
  That keeps the tool dependency-light for now, but it also means schema
  evolution should stay deliberate.

## Notes

- The next obvious improvement is to stabilize this output as reusable fixtures
- or a versioned schema and to fold automation continuity detail into the same
  shared supervisor report surface.

## Next Task

Break the supervisor tool output into reusable fixtures or stable schemas that
external automation can consume without CLI scraping, then carry richer
automation-continuity detail into the shared supervisor report alongside the
new timeline fields.
