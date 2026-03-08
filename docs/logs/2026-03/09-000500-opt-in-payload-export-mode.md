# 2026-03-09 00:05:00 UTC: Opt-In Payload Export Mode

Status: completed
Owner: core-product

## Summary

Added an explicit `--include-payload` mode to `signal-supervisor-tools` so
payload detail is available for debugging and soak inspection without widening
the default host export contract.

## Changes

- added `--include-payload` parsing to `crates/signal-supervisor-tools`
- restored grouped payload detail only when that flag is present in text and
  JSON output
- kept the default export shape unchanged for schema version 1
- pinned the flag behavior with supervisor-tool tests
- updated the README, export contract, package map, and roadmap notes

## Validation

- `cargo fmt --all`
- `cargo check -p signal-supervisor-tools`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`

## Notes

- this batch does not change the host binaries; they remain on the narrow
  default surface
- runtime execution remains a weaker validation signal in this environment
  because fresh Rust binaries can intermittently stall at `_dyld_start`
- a full `cargo test -p signal-supervisor-tools` launch was attempted, but the
  test binary stalled after startup, so the validation gate for this batch
  remains compile-only plus test-build coverage

## Next Task

Decide whether `signal-supervisor-tools` should grow broader debug export
profiles beyond payload, or keep the opt-in surface narrow and add future
detail only through targeted flags.
