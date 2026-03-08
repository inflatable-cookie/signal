# 2026-03-09 01:00:00 UTC: Describe Export Introspection Mode

Status: completed
Owner: core-product

## Summary

Added a host-free `--describe-export` mode to `signal-supervisor-tools` so
other tooling can discover the frozen supervisor export schema and payload-only
debug policy without booting a local or server runtime host.

## Changes

- added `CliMode::DescribeExport` to `crates/signal-supervisor-tools`
- added text and JSON export-description renderers for schema version, default
  host-summary sections, and supported debug sections
- added argument parsing and render tests for the new mode
- updated the README and supervisor export contract to point at
  `--describe-export` as the canonical introspection path

## Validation

- `cargo fmt --all`
- `cargo check -p signal-supervisor-tools`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`

## Notes

- this keeps export-policy discovery separate from runtime host execution
- runtime execution is still not used as the validation gate in this
  environment because fresh Rust binaries can intermittently stall after launch

## Next Task

Move away from supervisor-export policy work and pick the next central engine
slice, most likely runtime/host control-path hardening or the next real
plugin-sandbox lifecycle increment.
